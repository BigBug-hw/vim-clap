use super::executable_searcher::LanguageRegexSearcher;
use crate::tools::rg::{Match, Word};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;

/// A map of the ripgrep language to a set of regular expressions.
///
/// Ref: https://github.com/jacktasia/dumb-jump/blob/master/dumb-jump.el.
static RG_PCRE2_REGEX_RULES: Lazy<HashMap<&str, DefinitionRules>> = Lazy::new(|| {
    serde_json::from_str(include_str!(
        "../../../../../../scripts/dumb_jump/rg_pcre2_regex.json"
    ))
    .expect("Wrong path for rg_pcre2_regex.json")
});

/// Type of match result of ripgrep.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum MatchKind {
    /// Results matched from the definition regexp.
    Definition(DefinitionKind),
    /// Occurrences with the definition items excluded.
    Reference,
    /// Pure text matching results on top of ripgrep.
    Occurrence,
}

impl Display for MatchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Definition(def_kind) => write!(f, "{}", def_kind.as_ref()),
            Self::Reference => write!(f, "refs"),
            Self::Occurrence => write!(f, "grep"),
        }
    }
}

impl From<DefinitionKind> for MatchKind {
    fn from(def_kind: DefinitionKind) -> Self {
        Self::Definition(def_kind)
    }
}

/// Unit type wrapper of the kind of definition.
///
/// Possibale values: variable, function, type, etc.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct DefinitionKind(String);

impl AsRef<str> for DefinitionKind {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Unit type wrapper of the regexp of a definition kind.
///
/// See more info in rg_pcre2_regex.json.
#[derive(Clone, Debug, Deserialize)]
pub struct DefinitionRegexp(Vec<String>);

impl DefinitionRegexp {
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }
}

/// Definition rules of a language.
#[derive(Clone, Debug, Deserialize)]
pub struct DefinitionRules(pub HashMap<DefinitionKind, DefinitionRegexp>);

impl DefinitionRules {
    fn kind_rules_for(&self, kind: &DefinitionKind) -> Option<impl Iterator<Item = &str>> {
        self.0.get(kind).map(|x| x.iter().map(|x| x.as_str()))
    }
}

/// Returns the definition rules given `lang`.
pub fn get_definition_rules(lang: &str) -> Option<&DefinitionRules> {
    /// A map of extension => ripgrep language.
    static EXTENSION_LANGUAGE_MAP: Lazy<HashMap<&str, &str>> =
        Lazy::new(|| [("js", "javascript")].iter().cloned().collect());

    match RG_PCRE2_REGEX_RULES.get(lang) {
        Some(rules) => Some(rules),
        None => EXTENSION_LANGUAGE_MAP
            .get(lang)
            .and_then(|l| RG_PCRE2_REGEX_RULES.get(l)),
    }
}

pub(super) fn build_full_regexp(lang: &str, kind: &DefinitionKind, word: &Word) -> Option<String> {
    let regexp = get_definition_rules(lang)?
        .kind_rules_for(kind)?
        .map(|x| x.replace("\\\\", "\\").replace("JJJ", &word.raw))
        .join("|");
    Some(regexp)
}

/// Returns true if the ripgrep match is a comment line.
#[inline]
pub(super) fn is_comment(mat: &Match, comments: &[&str]) -> bool {
    comments.iter().any(|c| mat.line_starts_with(c))
}

/// Search results of a specific definition kind.
#[derive(Debug, Clone)]
pub struct DefinitionSearchResult {
    pub kind: DefinitionKind,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone)]
pub struct Definitions {
    pub defs: Vec<DefinitionSearchResult>,
}

impl Definitions {
    pub fn flatten(&self) -> Vec<Match> {
        let defs_count = self.defs.iter().map(|def| def.matches.len()).sum();
        let mut defs = Vec::with_capacity(defs_count);
        for DefinitionSearchResult { matches, .. } in self.defs.iter() {
            defs.extend_from_slice(matches);
        }
        defs
    }

    #[allow(unused)]
    pub fn par_iter(&self) -> rayon::slice::Iter<'_, DefinitionSearchResult> {
        self.defs.par_iter()
    }

    pub fn into_par_iter(self) -> rayon::vec::IntoIter<DefinitionSearchResult> {
        self.defs.into_par_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Occurrences(pub Vec<Match>);

impl Occurrences {
    pub fn contains(&self, m: &Match) -> bool {
        self.0.contains(m)
    }

    #[allow(unused)]
    pub fn par_iter(&self) -> rayon::slice::Iter<'_, Match> {
        self.0.par_iter()
    }

    pub fn into_par_iter(self) -> rayon::vec::IntoIter<Match> {
        self.0.into_par_iter()
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Match) -> bool,
    {
        self.0.retain(f)
    }

    pub fn into_inner(self) -> Vec<Match> {
        self.0
    }
}

pub(super) fn definitions_and_references(
    lang_regex_searcher: LanguageRegexSearcher,
    comments: &[&str],
) -> std::io::Result<HashMap<MatchKind, Vec<Match>>> {
    let (definitions, mut occurrences) = lang_regex_searcher.all(comments);

    let defs = definitions.flatten();

    // There are some negative definitions we need to filter them out, e.g., the word
    // is a subtring in some identifer but we consider every word is a valid identifer.
    let positive_defs = defs
        .par_iter()
        .filter(|def| occurrences.contains(def))
        .collect::<Vec<_>>();

    let res: HashMap<MatchKind, Vec<Match>> = definitions
        .into_par_iter()
        .filter_map(|DefinitionSearchResult { kind, mut matches }| {
            matches.retain(|ref def| positive_defs.contains(def));
            if matches.is_empty() {
                None
            } else {
                Some((kind.into(), matches))
            }
        })
        .chain(rayon::iter::once((MatchKind::Reference, {
            occurrences.retain(|r| !defs.contains(r));
            occurrences.into_inner()
        })))
        .collect();

    if res.is_empty() {
        lang_regex_searcher
            .regexp_search(comments)
            .map(|results| std::iter::once((MatchKind::Occurrence, results)).collect())
    } else {
        Ok(res)
    }
}

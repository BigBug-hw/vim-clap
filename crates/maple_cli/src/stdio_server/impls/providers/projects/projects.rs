use std::cmp::Ordering;
use std::path::Path;
use crate::utils::{generate_data_file_path, load_json};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::prelude::*;
use filter::SourceItem;
use matcher::{Bonus, MatcherBuilder};
use serde::{Deserialize, Serialize};

use crate::utils::UtcTime;
use crate::recent_files::{FrecentEntry, SortPreference};

/// Maximum number of projects.
const MAX_ENTRIES: u64 = 100;

const PROJECTS_FILENAME: &str = "projects.json";

static PROJECTS_JSON_PATH: Lazy<Option<PathBuf>> =
    Lazy::new(|| generate_data_file_path(PROJECTS_FILENAME).ok());

pub static PROJECTS_IN_MEMORY: Lazy<Mutex<SortedProjects>> = Lazy::new(|| {
    let maybe_persistent = load_json(PROJECTS_JSON_PATH.as_deref())
        .map(|f: SortedProjects| f.remove_invalid_entries())
        .unwrap_or_default();
    Mutex::new(maybe_persistent)
});

fn store_projects(projects: &SortedProjects) -> std::io::Result<()> {
    crate::utils::write_json(projects, PROJECTS_JSON_PATH.as_ref())
}

/// In memory version of sorted recent files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SortedProjects {
    /// Maximum number of entries.
    pub max_entries: u64,
    /// Sort preference of entries.
    pub sort_preference: SortPreference,
    /// An ordered list of [`FrecentEntry`].
    pub entries: Vec<FrecentEntry>,
}

impl Default for SortedProjects {
    fn default() -> Self {
        Self {
            max_entries: MAX_ENTRIES,
            sort_preference: Default::default(),
            entries: Default::default(),
        }
    }
}

impl SortedProjects {
    /// Deletes the invalid ones from current entries.
    ///
    /// Used when loading from the disk.
    pub fn remove_invalid_entries(self) -> Self {
        Self {
            entries: self
                .entries
                .into_iter()
                .filter(|entry| {
                    let path = Path::new(&entry.fpath);
                    path.exists() && path.is_dir()
                })
                .collect(),
            ..self
        }
    }

    /// Returns the size of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Sort the entries by adding a bonus score given `cwd`.
    pub fn sort_by_cwd(&mut self, cwd: &str) {
        self.entries.sort_unstable_by(|a, b| {
            b.cwd_preferred_score(cwd)
                .partial_cmp(&a.cwd_preferred_score(cwd))
                .unwrap()
        });
    }

    pub fn filter_on_query(&self, query: &str, _cwd: String) -> Vec<filter::MatchedItem> {
        let source_items: Vec<SourceItem> = self
            .entries
            .iter()
            .map(|entry| entry.fpath.clone().into())
            .collect();

        let matcher = MatcherBuilder::default()
            .build(query.into());

        filter::par_filter(source_items, &matcher)
    }

    /// Updates or inserts a new entry in a sorted way.
    pub fn upsert(&mut self, path: String) {
        match self
            .entries
            .iter()
            .position(|entry| entry.fpath.as_str() == path.as_str())
        {
            Some(pos) => FrecentEntry::refresh_now(&mut self.entries[pos]),
            None => {
                let entry = FrecentEntry::new(path);
                self.entries.push(entry);
            }
        }

        self.entries
            .sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        if self.entries.len() > self.max_entries as usize {
            self.entries.truncate(self.max_entries as usize);
        }

        // Write back to the disk.
        if let Err(e) = store_projects(self) {
            tracing::error!(?e, "Failed to write the recent files to the disk");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_by_cwd() {
        let mut sorted_recent_files = SortedProjects::default();

        let entries = vec![
            "/usr/local/share/test1.txt",
            "/home/xlc/.vimrc",
            "/home/xlc/test.txt",
        ];

        for entry in entries.iter() {
            sorted_recent_files.upsert(entry.to_string());
        }

        sorted_recent_files.sort_by_cwd("/usr/local/share");

        assert_eq!(
            sorted_recent_files
                .entries
                .iter()
                .map(|entry| entry.fpath.as_str())
                .collect::<Vec<_>>(),
            vec![
                "/usr/local/share/test1.txt",
                "/home/xlc/test.txt",
                "/home/xlc/.vimrc",
            ]
        );
    }
}

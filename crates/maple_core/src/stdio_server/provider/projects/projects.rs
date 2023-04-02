use crate::dirs::PROJECT_DIRS;
use crate::stdio_server::handler::{CachedPreviewImpl, Preview, PreviewTarget};
use crate::stdio_server::provider::{ClapProvider, Context, SearcherControl};
use crate::stdio_server::vim::preview_syntax;
use anyhow::{anyhow, Result};
use filter::SourceItem;
use printer::Printer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use types::Query;

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    name: String,
    path: String,
}

#[derive(Debug)]
pub struct ProjectsProvider {
    projects: Vec<Project>,
    searcher_control: Option<SearcherControl>,
}

impl ProjectsProvider {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            searcher_control: None,
        }
    }

    fn projects_path() -> PathBuf {
        PROJECT_DIRS.data_dir().join("projects")
    }

    async fn initialize_provider(&mut self, ctx: &mut Context) -> Result<()> {
        self.projects.clear();
        // "~/.local/share/vimclap/projects"
        let path = Self::projects_path();
        let mut entries = tokio::fs::read_dir(&path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let f = std::io::BufReader::new(std::fs::File::open(&path)?);
            let project: Project = serde_json::from_reader(f)?;
            self.projects.push(project);
        }
        let entries: Vec<String> = self
            .projects
            .as_slice()
            .iter()
            .map(|v| format!("{}\t{}", v.name, v.path))
            .collect();
        let response = json!({ "entries": &entries, "total": entries.len() });
        ctx.vim
            .exec("clap#provider#projects#handle_on_initialize", response)?;

        Ok(())
    }

    fn process_query(&mut self, query: String, ctx: &Context) -> Result<()> {
        let matcher = ctx.matcher_builder().build(Query::from(&query));
        let source_items: Vec<SourceItem> = self
            .projects
            .iter()
            .map(|e| e.name.clone().into())
            .collect();
        let matched = filter::par_filter(source_items, &matcher);

        let printer = Printer::new(ctx.env.display_winwidth, icon::Icon::Null);
        let printer::DisplayLines {
            lines,
            indices,
            truncated_map,
            icon_added,
        } = printer.to_display_lines(matched.iter().take(200).cloned().collect());

        // The indices are empty on the empty query.
        let indices = indices
            .into_iter()
            .filter(|i| !i.is_empty())
            .collect::<Vec<_>>();

        let mut value = json!({
            "lines": lines,
            "indices": indices,
            "matched": matched.len(),
            "processed": self.projects.len(),
            "icon_added": icon_added,
            "preview": Option::<Value>::None,
        });

        if !truncated_map.is_empty() {
            value
                .as_object_mut()
                .expect("Value is constructed as an Object")
                .insert("truncated_map".into(), json!(truncated_map));
        }

        ctx.vim
            .exec("clap#state#process_response_on_typed", value)?;

        if !ctx.env.preview_enabled {
            return Ok(());
        }
        return Ok(());
    }

    async fn current_line(&self, ctx: &Context) -> Result<String> {
        let curline = ctx.vim.display_getcurline().await?;
        let curline: Vec<&str> = curline.splitn(2, '\t').collect();
        let curline = curline.get(1).ok_or(anyhow!("Failed to get curline"))?;
        Ok(curline.to_string())
    }

    async fn update_preview(&self, preview_target: PreviewTarget, ctx: &mut Context) -> Result<()> {
        let preview_height = ctx.preview_height().await?;

        let preview_impl = CachedPreviewImpl {
            ctx,
            preview_height,
            preview_target,
            cache_line: None,
        };

        match preview_impl.get_preview().await {
            Ok((preview_target, preview)) => {
                ctx.preview_manager.reset_scroll();
                ctx.render_preview(preview)?;

                let maybe_syntax = preview_target.path().and_then(|path| {
                    if path.is_dir() {
                        Some("clap_projects")
                    } else if path.is_file() {
                        preview_syntax(path)
                    } else {
                        None
                    }
                });

                if let Some(syntax) = maybe_syntax {
                    ctx.vim.set_preview_syntax(syntax)?;
                }

                ctx.preview_manager.set_preview_target(preview_target);

                Ok(())
            }
            Err(err) => ctx.render_preview(Preview::new(vec![err.to_string()])),
        }
    }

    async fn preview_current_entry(&self, ctx: &mut Context) -> Result<()> {
        let curline = self.current_line(ctx).await?;
        if curline.is_empty() {
            tracing::debug!("Skipping preview as curline is empty");
            ctx.vim.bare_exec("clap#state#clear_preview")?;
            return Ok(());
        }
        // curline.push_str(".json");
        let target_dir = Self::projects_path().join(curline);
        let preview_target = if target_dir.is_dir() {
            PreviewTarget::Directory(target_dir)
        } else {
            PreviewTarget::File(target_dir)
        };

        self.update_preview(preview_target, ctx).await
    }
}

#[async_trait::async_trait]
impl ClapProvider for ProjectsProvider {
    async fn on_initialize(&mut self, ctx: &mut Context) -> Result<()> {
        let query = ctx.vim.context_query_or_input().await?;
        if !query.is_empty() {
            self.process_query(query, ctx)?;
        } else {
            self.initialize_provider(ctx).await?;
        }
        Ok(())
    }

    async fn on_typed(&mut self, ctx: &mut Context) -> Result<()> {
        let query = ctx.vim.input_get().await?;
        if !query.is_empty() {
            self.process_query(query, ctx)?;
        } else {
            self.initialize_provider(ctx).await?;
        }
        Ok(())
    }

    async fn on_move(&mut self, ctx: &mut Context) -> Result<()> {
        if !ctx.env.preview_enabled {
            return Ok(());
        }
        self.preview_current_entry(ctx).await
    }

    fn on_terminate(&mut self, ctx: &mut Context, session_id: u64) {
        if let Some(control) = self.searcher_control.take() {
            // NOTE: The kill operation can not block current task.
            tokio::task::spawn_blocking(move || control.kill());
        }
        ctx.signify_terminated(session_id);
    }
}

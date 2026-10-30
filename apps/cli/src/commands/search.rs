use crate::args::SearchArgs;
use crate::output::OutputRenderer;
use devcontext_core::domain::RecordType;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::search::{SearchFilter, SearchService};
use devcontext_core::workspace::WorkspaceManager;
use std::path::PathBuf;
use std::str::FromStr;

pub fn handle_search(
    project_dir: Option<PathBuf>,
    args: SearchArgs,
    renderer: &OutputRenderer,
) -> Result<i32> {
    let start_dir = match project_dir {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let root = WorkspaceManager::find_project_root(&start_dir).ok_or_else(|| {
        CoreError::NotFound(format!(
            "No DevContext project found in '{}'. Run 'devcontext init' first.",
            start_dir.display()
        ))
    })?;

    let (project, storage) = WorkspaceManager::open_project(&root)?;

    if args.reindex {
        let count = SearchService::rebuild_index(&storage)?;
        renderer.success(&serde_json::json!({ "reindexed_count": count }), || {
            format!(
                "Successfully rebuilt search index ({} records indexed).",
                count
            )
        });
        return Ok(0);
    }

    let parsed_type = match args.record_type.as_deref() {
        Some(t) => {
            let normalized = match t.to_lowercase().as_str() {
                "note" => "note",
                "decision" => "decision",
                "milestone" => "milestone",
                "log" | "log_entry" => "log_entry",
                "command" | "saved_command" => "saved_command",
                _ => {
                    return Err(CoreError::Validation(format!(
                        "Invalid record type '{}'. Valid types: note, decision, milestone, log, command",
                        t
                    )))
                }
            };
            Some(RecordType::from_str(normalized)?)
        }
        None => None,
    };

    let filter = SearchFilter {
        record_type: parsed_type,
        limit: Some(args.limit),
    };

    let results = SearchService::search(&storage, project.id, &root, &args.query, &filter)?;

    renderer.success(&results, || {
        if results.is_empty() {
            return format!("No records found matching query \"{}\".", args.query);
        }

        let mut out = format!("Search Results ({} matches):\n\n", results.len());
        for res in &results {
            let type_label = res.record_type.to_string().to_uppercase();
            out.push_str(&format!(
                "[{}] {} (ID: {})\n",
                type_label, res.title, res.record_id
            ));

            if !res.snippet.is_empty() {
                out.push_str(&format!("  Snippet: {}\n", res.snippet.trim()));
            }

            if !res.tags.is_empty() {
                out.push_str(&format!("  Tags: {}\n", res.tags.join(", ")));
            }

            if !res.references.is_empty() {
                out.push_str(&format!("  References ({}):\n", res.references.len()));
                for r in &res.references {
                    out.push_str(&format!(
                        "    - [{}] {} ({})\n",
                        r.reference.ref_type, r.reference.target, r.status
                    ));
                }
            }

            out.push('\n');
        }

        out.trim_end().to_string()
    });

    Ok(0)
}

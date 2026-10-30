//! Markdown export generation.
//!
//! Produces a human-readable directory hierarchy with standard YAML frontmatter per ADR 0009.

use crate::error::Result;
use crate::export::model::ExportBundle;
use std::fs;
use std::path::Path;

/// Generates human-readable Markdown files in `output_dir` from an `ExportBundle` or project models.
pub fn generate_markdown_export(bundle: &ExportBundle, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    let notes_dir = output_dir.join("notes");
    let decisions_dir = output_dir.join("decisions");
    let milestones_dir = output_dir.join("milestones");
    let logs_dir = output_dir.join("logs");
    let commands_dir = output_dir.join("commands");

    fs::create_dir_all(&notes_dir)?;
    fs::create_dir_all(&decisions_dir)?;
    fs::create_dir_all(&milestones_dir)?;
    fs::create_dir_all(&logs_dir)?;
    fs::create_dir_all(&commands_dir)?;

    let mut toc_notes = Vec::new();
    let mut toc_decisions = Vec::new();
    let mut toc_milestones = Vec::new();
    let mut toc_logs = Vec::new();
    let mut toc_commands = Vec::new();

    for record in &bundle.records {
        let slug = slugify(&record.title, &record.id.to_string()[..8]);
        let filename = format!("{}.md", slug);

        match record.record_type.as_str() {
            "note" => {
                let note: crate::export::model::NotePayload = serde_json::from_value(
                    record.payload.clone(),
                )
                .unwrap_or(crate::export::model::NotePayload {
                    content_markdown: String::new(),
                    category: None,
                    is_pinned: false,
                    is_archived: false,
                });

                let mut frontmatter = format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nrecord_type: note\ncreated_at: \"{}\"\nupdated_at: \"{}\"\n",
                    record.id, escape_yaml(&record.title), record.created_at, record.updated_at
                );
                if let Some(cat) = &note.category {
                    frontmatter.push_str(&format!("category: \"{}\"\n", escape_yaml(cat)));
                }
                frontmatter.push_str(&format!(
                    "is_pinned: {}\nis_archived: {}\n",
                    note.is_pinned, note.is_archived
                ));
                frontmatter.push_str(&render_yaml_tags(&record.tags));
                frontmatter.push_str("---\n\n");

                let content = format!(
                    "{}# {}\n\n{}\n",
                    frontmatter, record.title, note.content_markdown
                );
                fs::write(notes_dir.join(&filename), content)?;
                toc_notes.push((record.title.clone(), format!("notes/{}", filename)));
            }

            "decision" => {
                let dec: crate::export::model::DecisionPayload = serde_json::from_value(
                    record.payload.clone(),
                )
                .unwrap_or(crate::export::model::DecisionPayload {
                    status: "proposed".to_string(),
                    context: String::new(),
                    decision: String::new(),
                    consequences: String::new(),
                    alternatives_considered: None,
                    superseded_by: None,
                });

                let mut frontmatter = format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nrecord_type: decision\nstatus: \"{}\"\ncreated_at: \"{}\"\nupdated_at: \"{}\"\n",
                    record.id, escape_yaml(&record.title), dec.status, record.created_at, record.updated_at
                );
                if let Some(sup) = dec.superseded_by {
                    frontmatter.push_str(&format!("superseded_by: \"{}\"\n", sup));
                }
                frontmatter.push_str(&render_yaml_tags(&record.tags));
                frontmatter.push_str("---\n\n");

                let mut body = format!("{}# {}\n\n## Status\n{}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}\n",
                    frontmatter, record.title, dec.status, dec.context, dec.decision, dec.consequences);

                if let Some(alt) = &dec.alternatives_considered {
                    if !alt.trim().is_empty() {
                        body.push_str(&format!("\n## Alternatives Considered\n{}\n", alt));
                    }
                }

                fs::write(decisions_dir.join(&filename), body)?;
                toc_decisions.push((record.title.clone(), format!("decisions/{}", filename)));
            }

            "milestone" => {
                let ms: crate::export::model::MilestonePayload = serde_json::from_value(
                    record.payload.clone(),
                )
                .unwrap_or(crate::export::model::MilestonePayload {
                    status: "planned".to_string(),
                    target_date: None,
                    completed_date: None,
                    description: None,
                });

                let mut frontmatter = format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nrecord_type: milestone\nstatus: \"{}\"\ncreated_at: \"{}\"\n",
                    record.id, escape_yaml(&record.title), ms.status, record.created_at
                );
                if let Some(td) = &ms.target_date {
                    frontmatter.push_str(&format!("target_date: \"{}\"\n", td));
                }
                if let Some(cd) = &ms.completed_date {
                    frontmatter.push_str(&format!("completed_date: \"{}\"\n", cd));
                }
                frontmatter.push_str(&render_yaml_tags(&record.tags));
                frontmatter.push_str("---\n\n");

                let desc = ms.description.unwrap_or_default();
                let content = format!(
                    "{}# {}\n\nStatus: {}\n\n{}\n",
                    frontmatter, record.title, ms.status, desc
                );
                fs::write(milestones_dir.join(&filename), content)?;
                toc_milestones.push((record.title.clone(), format!("milestones/{}", filename)));
            }

            "log_entry" => {
                let log: crate::export::model::LogEntryPayload = serde_json::from_value(
                    record.payload.clone(),
                )
                .unwrap_or(crate::export::model::LogEntryPayload {
                    event_date: record.created_at.clone(),
                    summary: record.title.clone(),
                    details: None,
                });

                let mut frontmatter =
                    format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nrecord_type: log_entry\nevent_date: \"{}\"\n",
                    record.id, escape_yaml(&record.title), log.event_date
                );
                frontmatter.push_str(&render_yaml_tags(&record.tags));
                frontmatter.push_str("---\n\n");

                let mut content = format!(
                    "{}# {}\n\n## Summary\n{}\n",
                    frontmatter, record.title, log.summary
                );
                if let Some(details) = &log.details {
                    content.push_str(&format!("\n## Details\n{}\n", details));
                }

                fs::write(logs_dir.join(&filename), content)?;
                toc_logs.push((record.title.clone(), format!("logs/{}", filename)));
            }

            "saved_command" => {
                let cmd: crate::export::model::SavedCommandPayload = serde_json::from_value(
                    record.payload.clone(),
                )
                .unwrap_or(crate::export::model::SavedCommandPayload {
                    command_text: String::new(),
                    explanation: None,
                    working_directory: None,
                });

                let mut frontmatter = format!(
                    "---\nid: \"{}\"\ntitle: \"{}\"\nrecord_type: saved_command\n",
                    record.id,
                    escape_yaml(&record.title)
                );
                if let Some(wd) = &cmd.working_directory {
                    frontmatter.push_str(&format!("working_directory: \"{}\"\n", escape_yaml(wd)));
                }
                frontmatter.push_str(&render_yaml_tags(&record.tags));
                frontmatter.push_str("---\n\n");

                let mut content = format!(
                    "{}# {}\n\n```bash\n{}\n```\n",
                    frontmatter, record.title, cmd.command_text
                );
                if let Some(exp) = &cmd.explanation {
                    content.push_str(&format!("\n{}\n", exp));
                }

                fs::write(commands_dir.join(&filename), content)?;
                toc_commands.push((record.title.clone(), format!("commands/{}", filename)));
            }

            _ => {}
        }
    }

    // Generate root README.md table of contents
    let mut readme = format!(
        "# DevContext Export: {}\n\n- **Project ID:** `{}`\n- **Exported At:** {}\n- **Schema Version:** {}\n\n",
        bundle.project.name, bundle.project.id, bundle.exported_at, bundle.schema_version
    );

    readme.push_str("## Architecture Decision Records (ADRs)\n\n");
    if toc_decisions.is_empty() {
        readme.push_str("_No decisions recorded._\n\n");
    } else {
        for (title, path) in &toc_decisions {
            readme.push_str(&format!("- [{}]({})\n", title, path));
        }
        readme.push('\n');
    }

    readme.push_str("## Developer Notes\n\n");
    if toc_notes.is_empty() {
        readme.push_str("_No notes recorded._\n\n");
    } else {
        for (title, path) in &toc_notes {
            readme.push_str(&format!("- [{}]({})\n", title, path));
        }
        readme.push('\n');
    }

    readme.push_str("## Milestones\n\n");
    if toc_milestones.is_empty() {
        readme.push_str("_No milestones recorded._\n\n");
    } else {
        for (title, path) in &toc_milestones {
            readme.push_str(&format!("- [{}]({})\n", title, path));
        }
        readme.push('\n');
    }

    readme.push_str("## Development Logs\n\n");
    if toc_logs.is_empty() {
        readme.push_str("_No log entries recorded._\n\n");
    } else {
        for (title, path) in &toc_logs {
            readme.push_str(&format!("- [{}]({})\n", title, path));
        }
        readme.push('\n');
    }

    readme.push_str("## Saved Commands\n\n");
    if toc_commands.is_empty() {
        readme.push_str("_No saved commands recorded._\n\n");
    } else {
        for (title, path) in &toc_commands {
            readme.push_str(&format!("- [{}]({})\n", title, path));
        }
        readme.push('\n');
    }

    fs::write(output_dir.join("README.md"), readme)?;

    Ok(())
}

fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

fn render_yaml_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return "tags: []\n".to_string();
    }
    let mut out = "tags:\n".to_string();
    for t in tags {
        out.push_str(&format!("  - \"{}\"\n", escape_yaml(t)));
    }
    out
}

fn slugify(title: &str, fallback_id: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if (ch == ' ' || ch == '-' || ch == '_') && !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        fallback_id.to_string()
    } else if trimmed.len() > 60 {
        trimmed[..60].trim_matches('-').to_string()
    } else {
        trimmed.to_string()
    }
}

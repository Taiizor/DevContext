use crate::args::NoteCommands;
use crate::output::OutputRenderer;
use devcontext_core::domain::Note;
use devcontext_core::error::{CoreError, Result};
use devcontext_core::workspace::WorkspaceManager;
use serde::Serialize;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&Note> for NoteSummary {
    fn from(n: &Note) -> Self {
        Self {
            id: n.record.id.to_string(),
            title: n.record.title.clone(),
            category: n.category.clone(),
            tags: n.record.tags.clone(),
            is_pinned: n.is_pinned,
            is_archived: n.is_archived,
            created_at: n.record.created_at.clone(),
            updated_at: n.record.updated_at.clone(),
        }
    }
}

pub fn handle_note(
    project_dir: Option<PathBuf>,
    cmd: NoteCommands,
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

    match cmd {
        NoteCommands::Add {
            title,
            content,
            category,
            tag,
        } => {
            let body = content.unwrap_or_else(|| format!("# {}\n", title));
            let note = Note::new(project.id, title, body, tag, category)?;
            storage.save_note(&note)?;

            let summary = NoteSummary::from(&note);
            renderer.success(&summary, || {
                format!(
                    "Created note [{}] \"{}\" in category '{}'",
                    summary.id,
                    summary.title,
                    summary.category.as_deref().unwrap_or("none")
                )
            });
            Ok(0)
        }

        NoteCommands::List { all, tag } => {
            let notes = storage.list_notes(project.id, all)?;
            let filtered: Vec<NoteSummary> = notes
                .iter()
                .filter(|n| match &tag {
                    Some(t) => n.record.tags.contains(&t.to_lowercase()),
                    None => true,
                })
                .map(NoteSummary::from)
                .collect();

            renderer.success(&filtered, || {
                if filtered.is_empty() {
                    return "No notes found.".to_string();
                }
                let mut out = format!("Notes ({}):\n", filtered.len());
                for n in &filtered {
                    let pin_mark = if n.is_pinned { " [PINNED]" } else { "" };
                    let arch_mark = if n.is_archived { " [ARCHIVED]" } else { "" };
                    out.push_str(&format!(
                        "- {} \"{}\"{}{}\n  Tags: {:?}\n",
                        n.id, n.title, pin_mark, arch_mark, n.tags
                    ));
                }
                out
            });
            Ok(0)
        }

        NoteCommands::Show { id } => {
            let note_id = Uuid::parse_str(&id)
                .map_err(|e| CoreError::Validation(format!("Invalid note UUID '{}': {}", id, e)))?;

            let note = storage
                .get_note(note_id)?
                .ok_or_else(|| CoreError::NotFound(format!("Note with id '{}' not found", id)))?;

            renderer.success(&note, || {
                format!(
                    "# {}\nID: {}\nCategory: {}\nTags: {:?}\nPinned: {} | Archived: {}\nCreated: {}\nUpdated: {}\n\n{}",
                    note.record.title,
                    note.record.id,
                    note.category.as_deref().unwrap_or("none"),
                    note.record.tags,
                    note.is_pinned,
                    note.is_archived,
                    note.record.created_at,
                    note.record.updated_at,
                    note.content_markdown
                )
            });
            Ok(0)
        }

        NoteCommands::Edit {
            id,
            title,
            content,
            pin,
            unpin,
            archive,
            unarchive,
        } => {
            let note_id = Uuid::parse_str(&id)
                .map_err(|e| CoreError::Validation(format!("Invalid note UUID '{}': {}", id, e)))?;

            let mut note = storage
                .get_note(note_id)?
                .ok_or_else(|| CoreError::NotFound(format!("Note with id '{}' not found", id)))?;

            if let Some(new_title) = title {
                note.record.update_title(new_title)?;
            }
            if let Some(new_content) = content {
                note.update_content(new_content)?;
            }
            if pin {
                note.set_pinned(true);
            } else if unpin {
                note.set_pinned(false);
            }
            if archive {
                note.set_archived(true);
            } else if unarchive {
                note.set_archived(false);
            }

            storage.save_note(&note)?;

            let summary = NoteSummary::from(&note);
            renderer.success(&summary, || {
                format!("Updated note [{}] \"{}\"", summary.id, summary.title)
            });
            Ok(0)
        }

        NoteCommands::Delete { id, hard } => {
            let note_id = Uuid::parse_str(&id)
                .map_err(|e| CoreError::Validation(format!("Invalid note UUID '{}': {}", id, e)))?;

            let deleted = if hard {
                storage.hard_delete_record(note_id)?
            } else {
                storage.soft_delete_record(note_id)?
            };

            if !deleted {
                return Err(CoreError::NotFound(format!(
                    "Note with id '{}' not found",
                    id
                )));
            }

            renderer.success(&id, || {
                format!(
                    "{} note [{}]",
                    if hard {
                        "Permanently deleted"
                    } else {
                        "Soft-deleted"
                    },
                    id
                )
            });
            Ok(0)
        }
    }
}

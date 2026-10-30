//! Export, import, inspection, and data safety orchestration services.

use crate::domain::{
    Decision, DecisionStatus, LogEntry, Milestone, MilestoneStatus, Note, Project, Record,
    RecordType, Reference, ReferenceType, SavedCommand,
};
use crate::error::{CoreError, Result};
use crate::export::archive::{create_zip_archive, read_bundle_from_zip};
use crate::export::markdown::generate_markdown_export;
use crate::export::model::{
    ConflictStrategy, DecisionPayload, DeleteResult, ExportBundle, ExportFormat, ExportOptions,
    ExportPreview, ExportProject, ExportRecord, ExportReference, ExportVcs, GeneratorInfo,
    ImportOptions, ImportResult, InspectionReport, LogEntryPayload, MilestonePayload, NotePayload,
    SavedCommandPayload, SCHEMA_VERSION_1,
};
use crate::export::sanitization::sanitize_target;
use crate::storage::backup::BackupService;
use crate::storage::sqlite_storage::SqliteStorage;
use crate::workspace::WorkspaceManager;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use uuid::Uuid;

pub struct ExportService;

impl ExportService {
    /// Builds an in-memory `ExportBundle` from the active records in the database.
    pub fn build_export_bundle(
        project: &Project,
        storage: &SqliteStorage,
        project_root: &Path,
        redact_paths: bool,
    ) -> Result<ExportBundle> {
        let mut export_records = Vec::new();

        // 1. Export Notes
        let notes = storage.list_notes(project.id, false)?;
        for note in notes {
            let refs = Self::fetch_and_sanitize_references(
                storage,
                note.record.id,
                project_root,
                redact_paths,
            )?;
            let payload = serde_json::to_value(NotePayload {
                content_markdown: note.content_markdown,
                category: note.category,
                is_pinned: note.is_pinned,
                is_archived: note.is_archived,
            })?;

            export_records.push(ExportRecord {
                id: note.record.id,
                record_type: "note".to_string(),
                title: note.record.title,
                created_at: note.record.created_at,
                updated_at: note.record.updated_at,
                tags: note.record.tags,
                references: refs,
                payload,
            });
        }

        // 2. Export Decisions (ADRs)
        let decisions = storage.list_decisions(project.id)?;
        for dec in decisions {
            let refs = Self::fetch_and_sanitize_references(
                storage,
                dec.record.id,
                project_root,
                redact_paths,
            )?;
            let payload = serde_json::to_value(DecisionPayload {
                status: dec.status.to_string(),
                context: dec.context,
                decision: dec.decision,
                consequences: dec.consequences,
                alternatives_considered: dec.alternatives_considered,
                superseded_by: dec.superseded_by,
            })?;

            export_records.push(ExportRecord {
                id: dec.record.id,
                record_type: "decision".to_string(),
                title: dec.record.title,
                created_at: dec.record.created_at,
                updated_at: dec.record.updated_at,
                tags: dec.record.tags,
                references: refs,
                payload,
            });
        }

        // 3. Export Milestones
        let milestones = storage.list_milestones(project.id)?;
        for ms in milestones {
            let refs = Self::fetch_and_sanitize_references(
                storage,
                ms.record.id,
                project_root,
                redact_paths,
            )?;
            let payload = serde_json::to_value(MilestonePayload {
                status: ms.status.to_string(),
                target_date: ms.target_date,
                completed_date: ms.completed_date,
                description: ms.description,
            })?;

            export_records.push(ExportRecord {
                id: ms.record.id,
                record_type: "milestone".to_string(),
                title: ms.record.title,
                created_at: ms.record.created_at,
                updated_at: ms.record.updated_at,
                tags: ms.record.tags,
                references: refs,
                payload,
            });
        }

        // 4. Export Log Entries
        let logs = storage.list_log_entries(project.id)?;
        for log in logs {
            let refs = Self::fetch_and_sanitize_references(
                storage,
                log.record.id,
                project_root,
                redact_paths,
            )?;
            let payload = serde_json::to_value(LogEntryPayload {
                event_date: log.event_date,
                summary: log.summary,
                details: log.details,
            })?;

            export_records.push(ExportRecord {
                id: log.record.id,
                record_type: "log_entry".to_string(),
                title: log.record.title,
                created_at: log.record.created_at,
                updated_at: log.record.updated_at,
                tags: log.record.tags,
                references: refs,
                payload,
            });
        }

        // 5. Export Saved Commands
        let commands = storage.list_commands(project.id)?;
        for cmd in commands {
            let refs = Self::fetch_and_sanitize_references(
                storage,
                cmd.record.id,
                project_root,
                redact_paths,
            )?;
            let working_dir = cmd
                .working_directory
                .as_deref()
                .map(|wd| sanitize_target(wd, project_root, redact_paths));

            let payload = serde_json::to_value(SavedCommandPayload {
                command_text: cmd.command_text,
                explanation: cmd.explanation,
                working_directory: working_dir,
            })?;

            export_records.push(ExportRecord {
                id: cmd.record.id,
                record_type: "saved_command".to_string(),
                title: cmd.record.title,
                created_at: cmd.record.created_at,
                updated_at: cmd.record.updated_at,
                tags: cmd.record.tags,
                references: refs,
                payload,
            });
        }

        // Stable chronological ordering
        export_records.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let vcs = project.vcs_identifier.as_ref().map(|v| ExportVcs {
            vcs_type: "git".to_string(),
            root_identifier: Some(v.clone()),
        });

        Ok(ExportBundle {
            schema_version: SCHEMA_VERSION_1.to_string(),
            exported_at: Utc::now().to_rfc3339(),
            generator: GeneratorInfo::default(),
            project: ExportProject {
                id: project.id,
                name: project.name.clone(),
                created_at: project.created_at.clone(),
                vcs,
            },
            records: export_records,
        })
    }

    fn fetch_and_sanitize_references(
        storage: &SqliteStorage,
        record_id: Uuid,
        project_root: &Path,
        redact_paths: bool,
    ) -> Result<Vec<ExportReference>> {
        let refs = storage.list_references_for_record(record_id)?;
        let mut out = Vec::with_capacity(refs.len());

        for r in refs {
            let sanitized_target = match r.ref_type {
                ReferenceType::File | ReferenceType::Folder => {
                    sanitize_target(&r.target, project_root, redact_paths)
                }
                _ => r.target,
            };

            out.push(ExportReference {
                id: r.id,
                ref_type: r.ref_type.to_string(),
                target: sanitized_target,
                label: r.label,
                metadata: r.metadata,
            });
        }

        Ok(out)
    }

    /// Exports project data into the specified format (JSON, Markdown, or ZIP).
    pub fn export_project(
        project_root: &Path,
        options: &ExportOptions,
    ) -> Result<(ExportBundle, PathBuf)> {
        let (project, storage) = WorkspaceManager::open_project(project_root)?;
        let bundle =
            Self::build_export_bundle(&project, &storage, project_root, options.redact_paths)?;

        let timestamp_slug = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let default_name = format!("{}-export-{}", project.name, timestamp_slug);

        let dest_path = match &options.output_path {
            Some(p) => p.clone(),
            None => match options.format {
                ExportFormat::Json => project_root.join(format!("{}.json", default_name)),
                ExportFormat::Markdown => project_root.join(default_name),
                ExportFormat::Zip => project_root.join(format!("{}.zip", default_name)),
            },
        };

        if let Some(parent) = dest_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        match options.format {
            ExportFormat::Json => {
                let json_str = serde_json::to_string_pretty(&bundle)
                    .map_err(|e| CoreError::Validation(e.to_string()))?;
                fs::write(&dest_path, json_str)?;
            }
            ExportFormat::Markdown => {
                generate_markdown_export(&bundle, &dest_path)?;
            }
            ExportFormat::Zip => {
                create_zip_archive(&bundle, &dest_path)?;
            }
        }

        Ok((bundle, dest_path))
    }

    /// Generates a preview summary of records that would be exported.
    pub fn preview_export(project_root: &Path, redact_paths: bool) -> Result<ExportPreview> {
        let (project, storage) = WorkspaceManager::open_project(project_root)?;

        let notes = storage.list_notes(project.id, false)?;
        let decisions = storage.list_decisions(project.id)?;
        let milestones = storage.list_milestones(project.id)?;
        let logs = storage.list_log_entries(project.id)?;
        let commands = storage.list_commands(project.id)?;

        let mut ref_count = 0;
        for n in &notes {
            ref_count += storage.list_references_for_record(n.record.id)?.len();
        }
        for d in &decisions {
            ref_count += storage.list_references_for_record(d.record.id)?.len();
        }
        for m in &milestones {
            ref_count += storage.list_references_for_record(m.record.id)?.len();
        }
        for l in &logs {
            ref_count += storage.list_references_for_record(l.record.id)?.len();
        }
        for c in &commands {
            ref_count += storage.list_references_for_record(c.record.id)?.len();
        }

        let total_records =
            notes.len() + decisions.len() + milestones.len() + logs.len() + commands.len();

        Ok(ExportPreview {
            project_name: project.name,
            notes_count: notes.len(),
            decisions_count: decisions.len(),
            milestones_count: milestones.len(),
            logs_count: logs.len(),
            commands_count: commands.len(),
            references_count: ref_count,
            total_records,
            redact_paths,
        })
    }

    /// Reads and inspects a source bundle file (JSON or ZIP archive) or local database.
    pub fn inspect_source(
        project_root: &Path,
        file_path: Option<&Path>,
    ) -> Result<InspectionReport> {
        let (project, storage) = WorkspaceManager::open_project(project_root)?;

        match file_path {
            None => {
                // Inspect current project database
                let notes = storage.list_notes(project.id, false)?;
                let decisions = storage.list_decisions(project.id)?;
                let milestones = storage.list_milestones(project.id)?;
                let logs = storage.list_log_entries(project.id)?;
                let commands = storage.list_commands(project.id)?;

                let mut ref_count = 0;
                for n in &notes {
                    ref_count += storage.list_references_for_record(n.record.id)?.len();
                }
                for d in &decisions {
                    ref_count += storage.list_references_for_record(d.record.id)?.len();
                }
                for m in &milestones {
                    ref_count += storage.list_references_for_record(m.record.id)?.len();
                }
                for l in &logs {
                    ref_count += storage.list_references_for_record(l.record.id)?.len();
                }
                for c in &commands {
                    ref_count += storage.list_references_for_record(c.record.id)?.len();
                }

                let total_records =
                    notes.len() + decisions.len() + milestones.len() + logs.len() + commands.len();

                Ok(InspectionReport {
                    source_type: "database".to_string(),
                    schema_version: Some(SCHEMA_VERSION_1.to_string()),
                    project_name: project.name,
                    project_id: project.id,
                    notes_count: notes.len(),
                    decisions_count: decisions.len(),
                    milestones_count: milestones.len(),
                    logs_count: logs.len(),
                    commands_count: commands.len(),
                    references_count: ref_count,
                    total_records,
                    conflicts_with_existing: 0,
                })
            }

            Some(path) => {
                let (bundle, source_type) = Self::load_bundle_from_path(path)?;

                let mut notes_count = 0;
                let mut decisions_count = 0;
                let mut milestones_count = 0;
                let mut logs_count = 0;
                let mut commands_count = 0;
                let mut ref_count = 0;
                let mut conflicts = 0;

                for rec in &bundle.records {
                    ref_count += rec.references.len();
                    match rec.record_type.as_str() {
                        "note" => notes_count += 1,
                        "decision" => decisions_count += 1,
                        "milestone" => milestones_count += 1,
                        "log_entry" => logs_count += 1,
                        "saved_command" => commands_count += 1,
                        _ => {}
                    }

                    if storage.get_record_envelope(rec.id)?.is_some() {
                        conflicts += 1;
                    }
                }

                Ok(InspectionReport {
                    source_type,
                    schema_version: Some(bundle.schema_version),
                    project_name: bundle.project.name,
                    project_id: bundle.project.id,
                    notes_count,
                    decisions_count,
                    milestones_count,
                    logs_count,
                    commands_count,
                    references_count: ref_count,
                    total_records: bundle.records.len(),
                    conflicts_with_existing: conflicts,
                })
            }
        }
    }

    /// Imports records from a JSON file or ZIP bundle into the project with data safety guarantees.
    pub fn import_bundle(
        project_root: &Path,
        bundle_path: &Path,
        options: &ImportOptions,
    ) -> Result<ImportResult> {
        let (bundle, _) = Self::load_bundle_from_path(bundle_path)?;

        // Schema version check
        if bundle.schema_version != SCHEMA_VERSION_1 {
            return Err(CoreError::Validation(format!(
                "Unsupported export schema version '{}'. Supported version: '{}'.",
                bundle.schema_version, SCHEMA_VERSION_1
            )));
        }

        let (project, storage) = WorkspaceManager::open_project(project_root)?;

        let mut conflicts = 0;
        for rec in &bundle.records {
            if storage.get_record_envelope(rec.id)?.is_some() {
                conflicts += 1;
            }
        }

        // Dry-run preview
        if options.dry_run {
            let mut imported = 0;
            let mut skipped = 0;
            let mut overwritten = 0;

            for rec in &bundle.records {
                let exists = storage.get_record_envelope(rec.id)?.is_some();
                if exists {
                    match options.on_conflict {
                        ConflictStrategy::Skip => skipped += 1,
                        ConflictStrategy::Overwrite => overwritten += 1,
                        ConflictStrategy::GenerateNewId => imported += 1,
                    }
                } else {
                    imported += 1;
                }
            }

            return Ok(ImportResult {
                dry_run: true,
                total_incoming: bundle.records.len(),
                imported_count: imported,
                skipped_count: skipped,
                overwritten_count: overwritten,
                conflicts_count: conflicts,
                backup_path: None,
            });
        }

        // Pre-import backup
        let backup_path_str = if options.create_backup {
            let backup_dir = project_root.join(".devcontext").join("backups");
            fs::create_dir_all(&backup_dir)?;
            let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
            let backup_file = backup_dir.join(format!(
                "pre-import-{}-{}.db",
                timestamp,
                &Uuid::new_v4().to_string()[..8]
            ));
            BackupService::backup_to_file(&storage, &backup_file)?;
            Some(backup_file.to_string_lossy().to_string())
        } else {
            None
        };

        let mut imported_count = 0;
        let mut skipped_count = 0;
        let mut overwritten_count = 0;
        let mut id_remap: HashMap<Uuid, Uuid> = HashMap::new();

        for rec in &bundle.records {
            let existing = storage.get_record_envelope(rec.id)?;
            let (target_id, is_overwrite) = if existing.is_some() {
                match options.on_conflict {
                    ConflictStrategy::Skip => {
                        skipped_count += 1;
                        continue;
                    }
                    ConflictStrategy::Overwrite => (rec.id, true),
                    ConflictStrategy::GenerateNewId => {
                        let new_id = Uuid::new_v4();
                        id_remap.insert(rec.id, new_id);
                        (new_id, false)
                    }
                }
            } else {
                (rec.id, false)
            };

            Self::persist_imported_record(&storage, project.id, target_id, rec, &id_remap)?;

            if is_overwrite {
                overwritten_count += 1;
            } else {
                imported_count += 1;
            }
        }

        // Rebuild search index to ensure imported content is instantly searchable
        let _ = storage.rebuild_search_index();

        Ok(ImportResult {
            dry_run: false,
            total_incoming: bundle.records.len(),
            imported_count,
            skipped_count,
            overwritten_count,
            conflicts_count: conflicts,
            backup_path: backup_path_str,
        })
    }

    fn persist_imported_record(
        storage: &SqliteStorage,
        project_id: Uuid,
        record_id: Uuid,
        export_rec: &ExportRecord,
        id_remap: &HashMap<Uuid, Uuid>,
    ) -> Result<()> {
        let record = Record::with_id(
            record_id,
            project_id,
            RecordType::from_str(&export_rec.record_type)?,
            export_rec.title.clone(),
            export_rec.created_at.clone(),
            export_rec.updated_at.clone(),
            export_rec.tags.clone(),
            false,
        )?;

        match export_rec.record_type.as_str() {
            "note" => {
                let p: NotePayload = serde_json::from_value(export_rec.payload.clone())
                    .map_err(|e| CoreError::Validation(format!("Invalid Note payload: {}", e)))?;
                let note = Note {
                    record,
                    content_markdown: p.content_markdown,
                    category: p.category,
                    is_pinned: p.is_pinned,
                    is_archived: p.is_archived,
                };
                storage.save_note(&note)?;
            }

            "decision" => {
                let p: DecisionPayload = serde_json::from_value(export_rec.payload.clone())
                    .map_err(|e| {
                        CoreError::Validation(format!("Invalid Decision payload: {}", e))
                    })?;
                let status = DecisionStatus::from_str(&p.status)?;
                let superseded_by = p
                    .superseded_by
                    .map(|s_id| *id_remap.get(&s_id).unwrap_or(&s_id));

                let decision = Decision {
                    record,
                    status,
                    context: p.context,
                    decision: p.decision,
                    consequences: p.consequences,
                    alternatives_considered: p.alternatives_considered,
                    superseded_by,
                };
                storage.save_decision(&decision)?;
            }

            "milestone" => {
                let p: MilestonePayload = serde_json::from_value(export_rec.payload.clone())
                    .map_err(|e| {
                        CoreError::Validation(format!("Invalid Milestone payload: {}", e))
                    })?;
                let status = MilestoneStatus::from_str(&p.status)?;

                let milestone = Milestone {
                    record,
                    status,
                    target_date: p.target_date,
                    completed_date: p.completed_date,
                    description: p.description,
                };
                storage.save_milestone(&milestone)?;
            }

            "log_entry" => {
                let p: LogEntryPayload = serde_json::from_value(export_rec.payload.clone())
                    .map_err(|e| {
                        CoreError::Validation(format!("Invalid LogEntry payload: {}", e))
                    })?;

                let log = LogEntry {
                    record,
                    event_date: p.event_date,
                    summary: p.summary,
                    details: p.details,
                };
                storage.save_log_entry(&log)?;
            }

            "saved_command" => {
                let p: SavedCommandPayload = serde_json::from_value(export_rec.payload.clone())
                    .map_err(|e| {
                        CoreError::Validation(format!("Invalid SavedCommand payload: {}", e))
                    })?;

                let cmd = SavedCommand {
                    record,
                    command_text: p.command_text,
                    explanation: p.explanation,
                    working_directory: p.working_directory,
                };
                storage.save_command(&cmd)?;
            }

            other => {
                return Err(CoreError::Validation(format!(
                    "Unknown record type '{}'",
                    other
                )));
            }
        }

        // Persist references
        for r in &export_rec.references {
            let ref_type = ReferenceType::from_str(&r.ref_type)?;
            let ref_id = Uuid::new_v4();
            let reference = Reference::with_id(
                ref_id,
                record_id,
                ref_type,
                r.target.clone(),
                r.label.clone(),
                r.metadata.clone(),
                export_rec.created_at.clone(),
            )?;
            storage.save_reference(&reference)?;
        }

        Ok(())
    }

    /// Deletes all project context data safely, creating a mandatory pre-deletion backup.
    pub fn delete_project_data(
        project_root: &Path,
        force: bool,
        create_backup: bool,
    ) -> Result<DeleteResult> {
        if !force {
            return Err(CoreError::Validation(
                "Data deletion requires explicit confirmation (--force).".to_string(),
            ));
        }

        let (project, storage) = WorkspaceManager::open_project(project_root)?;

        let backup_path = if create_backup {
            let backup_dir = project_root.join(".devcontext").join("backups");
            fs::create_dir_all(&backup_dir)?;
            let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
            let backup_file = backup_dir.join(format!(
                "pre-delete-{}-{}.db",
                timestamp,
                &Uuid::new_v4().to_string()[..8]
            ));
            BackupService::backup_to_file(&storage, &backup_file)?;
            Some(backup_file.to_string_lossy().to_string())
        } else {
            None
        };

        let deleted_count = storage.clear_project_data(project.id)?;

        Ok(DeleteResult {
            project_name: project.name,
            deleted_records_count: deleted_count,
            backup_path,
        })
    }

    fn load_bundle_from_path(bundle_path: &Path) -> Result<(ExportBundle, String)> {
        if !bundle_path.exists() {
            return Err(CoreError::NotFound(format!(
                "Bundle file not found: '{}'",
                bundle_path.display()
            )));
        }

        // Check if file is ZIP
        let is_zip = bundle_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);

        if is_zip {
            let bundle = read_bundle_from_zip(bundle_path)?;
            Ok((bundle, "zip_archive".to_string()))
        } else {
            let content = fs::read_to_string(bundle_path)?;
            let bundle: ExportBundle = serde_json::from_str(&content)
                .map_err(|e| CoreError::Validation(format!("Invalid export JSON: {}", e)))?;
            Ok((bundle, "json_bundle".to_string()))
        }
    }
}

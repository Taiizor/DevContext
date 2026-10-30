/**
 * Standard typed JSON response envelope received from devcontext CLI.
 */
export interface CliEnvelope<T> {
  success: boolean;
  data: T | null;
  error: CliError | null;
}

/**
 * Structured CLI error payload.
 */
export interface CliError {
  code: string;
  message: string;
  details?: unknown;
}

/**
 * Diagnostic doctor output from `devcontext doctor --json`.
 */
export interface DoctorData {
  cli_version: string;
  protocol_version: string;
  project_detected: boolean;
  project_info: ProjectDoctorInfo | null;
}

export interface ProjectDoctorInfo {
  root_path: string;
  schema_version: number;
  journal_mode: string;
  foreign_keys_valid: boolean;
  integrity_status: string;
}

/**
 * Project status summary from `devcontext status --json`.
 */
export interface ProjectStatus {
  project_id: string;
  name: string;
  root_path: string;
  created_at: string;
  vcs_identifier: string | null;
  stats: ProjectStats;
}

export interface ProjectStats {
  notes_count: number;
  decisions_count: number;
  milestones_count: number;
  log_entries_count: number;
}

/**
 * Note summary from `devcontext note list --json`.
 */
export interface NoteSummary {
  id: string;
  title: string;
  category: string | null;
  tags: string[];
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * Detailed Note record from `devcontext note show <id> --json`.
 */
export interface NoteDetail {
  record: {
    id: string;
    project_id: string;
    record_type: string;
    title: string;
    created_at: string;
    updated_at: string;
    tags: string[];
    is_deleted: boolean;
  };
  content_markdown: string;
  category: string | null;
  is_pinned: boolean;
  is_archived: boolean;
}

/**
 * Decision status enum.
 */
export type DecisionStatusType =
  | 'proposed'
  | 'accepted'
  | 'rejected'
  | 'deprecated'
  | 'superseded';

/**
 * Decision summary from `devcontext decision list --json`.
 */
export interface DecisionSummary {
  id: string;
  title: string;
  status: DecisionStatusType;
  superseded_by: string | null;
  created_at: string;
  tags: string[];
}

/**
 * Detailed Decision record from `devcontext decision show <id> --json`.
 */
export interface DecisionDetail {
  record: {
    id: string;
    project_id: string;
    record_type: string;
    title: string;
    created_at: string;
    updated_at: string;
    tags: string[];
    is_deleted: boolean;
  };
  status: DecisionStatusType;
  context: string;
  decision: string;
  consequences: string;
  alternatives_considered: string | null;
  superseded_by: string | null;
}

/**
 * Milestone summary from `devcontext milestone list --json`.
 */
export interface MilestoneSummary {
  id: string;
  title: string;
  description: string | null;
  target_date: string | null;
  completed_at: string | null;
  status: string;
}

/**
 * Development Log entry from `devcontext log list --json`.
 */
export interface LogEntrySummary {
  id: string;
  summary: string;
  details: string | null;
  category: string | null;
  created_at: string;
}

/**
 * Combined timeline event from `devcontext timeline --json`.
 */
export interface TimelineEvent {
  timestamp: string;
  event_type: 'commit' | 'note' | 'decision' | 'milestone' | 'log_entry';
  title: string;
  summary: string | null;
  author: string | null;
  identifier: string;
  metadata: Record<string, unknown>;
}

export interface TimelineView {
  events: TimelineEvent[];
  is_truncated: boolean;
}

/**
 * Reference item attached to a record.
 */
export interface Reference {
  id: string;
  record_id: string;
  ref_type: 'file' | 'folder' | 'symbol' | 'git_commit' | 'git_branch' | 'url' | 'command' | 'record';
  target: string;
  description: string | null;
  metadata: Record<string, unknown> | null;
}

export interface ResolvedReference {
  reference: Reference;
  status: 'resolved' | 'target_missing' | 'invalid' | 'unavailable';
  message: string | null;
}

/**
 * Full-text search match item from `devcontext search <query> --json`.
 */
export interface SearchResult {
  record_id: string;
  record_type: 'note' | 'decision' | 'milestone' | 'log_entry' | 'saved_command';
  title: string;
  snippet: string;
  score: number;
  created_at: string;
  updated_at: string;
  tags: string[];
  references: ResolvedReference[];
}

/**
 * Core executable version handshake status.
 */
export type HandshakeStatus =
  | 'ready'
  | 'incompatible_version'
  | 'binary_missing'
  | 'execution_failed';

export interface HandshakeResult {
  status: HandshakeStatus;
  executablePath: string | null;
  cliVersion: string | null;
  protocolVersion: string | null;
  message: string;
}

/**
 * Result returned by `devcontext export --json`.
 */
export interface ExportOutput {
  project_name: string;
  format: string;
  output_path: string;
  total_records: number;
  schema_version: string;
}

export type ExportResult = ExportOutput;

/**
 * Preview summary returned by `devcontext export --preview --json`.
 */
export interface ExportPreview {
  project_name: string;
  notes_count: number;
  decisions_count: number;
  milestones_count: number;
  logs_count: number;
  commands_count: number;
  references_count: number;
  total_records: number;
  redact_paths: boolean;
}

/**
 * Result returned by `devcontext import <file> --json`.
 */
export interface ImportResult {
  dry_run: boolean;
  total_incoming: number;
  imported_count: number;
  skipped_count: number;
  overwritten_count: number;
  conflicts_count: number;
  backup_path?: string | null;
}

/**
 * Detailed inspection report returned by `devcontext inspect [file] --json`.
 */
export interface InspectionReport {
  source_type: string;
  schema_version?: string;
  project_name: string;
  project_id: string;
  notes_count: number;
  decisions_count: number;
  milestones_count: number;
  logs_count: number;
  commands_count: number;
  references_count: number;
  total_records: number;
  conflicts_with_existing: number;
}

/**
 * Result returned by `devcontext delete-data --force --json`.
 */
export interface DeleteResult {
  project_name: string;
  deleted_records_count: number;
  backup_path?: string | null;
}

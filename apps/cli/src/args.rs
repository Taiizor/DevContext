use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "devcontext",
    author = "DevContext Contributors",
    version,
    about = "Local-first project memory system for software development",
    long_about = "DevContext preserves the human and technical context behind software projects:\ntechnical decisions, developer notes, milestones, and development logs directly in your local repository."
)]
pub struct CliArgs {
    /// Explicit project root directory (overrides auto-discovery)
    #[arg(long, global = true)]
    pub project_dir: Option<PathBuf>,

    /// Format output as machine-readable JSON (ADR 0007 envelope)
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new DevContext project in the current or specified directory
    Init {
        /// Optional project name (defaults to current folder name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Display current project context, identity, and statistics
    Status,

    /// Run diagnostic checks and verify database health
    Doctor,

    /// Manage developer context notes
    #[command(subcommand)]
    Note(NoteCommands),

    /// Manage Architecture Decision Records (ADRs)
    #[command(subcommand)]
    Decision(DecisionCommands),

    /// Manage project milestones and target goals
    #[command(subcommand)]
    Milestone(MilestoneCommands),

    /// Manage development log entries
    #[command(subcommand)]
    Log(LogCommands),

    /// Display unified chronological development timeline
    Timeline(TimelineArgs),

    /// Full-text search across notes, decisions, milestones, logs, and commands
    Search(SearchArgs),

    /// Export project context to versioned JSON, Markdown hierarchy, or portable ZIP archive
    Export(ExportArgs),

    /// Import context records from a versioned JSON bundle or portable ZIP archive
    Import(ImportArgs),

    /// Inspect an export bundle file or the current project database
    Inspect(InspectArgs),

    /// Delete all project context data safely (requires confirmation or --force)
    DeleteData(DeleteDataArgs),
}

#[derive(clap::Args, Debug)]
pub struct SearchArgs {
    /// Full-text search query string (optional when --reindex is specified)
    #[arg(default_value = "")]
    pub query: String,

    /// Filter results by record type (note, decision, milestone, log, command)
    #[arg(short = 't', long = "type")]
    pub record_type: Option<String>,

    /// Maximum number of search results to return
    #[arg(short, long, default_value = "20")]
    pub limit: usize,

    /// Rebuild the full-text search index from active database records
    #[arg(long)]
    pub reindex: bool,
}

#[derive(clap::Args, Debug)]
pub struct TimelineArgs {
    /// Maximum number of timeline events to display
    #[arg(short, long, default_value = "50")]
    pub limit: usize,

    /// Filter events by type (commit, note, decision, milestone, log)
    #[arg(short = 't', long = "type")]
    pub event_type: Option<String>,

    /// Filter events occurring on or after date (ISO-8601 string)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter events occurring on or before date (ISO-8601 string)
    #[arg(long)]
    pub until: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum NoteCommands {
    /// Add a new note
    Add {
        /// Title of the note
        title: String,

        /// Markdown content body (or defaults to empty markdown)
        #[arg(short, long)]
        content: Option<String>,

        /// Optional organizational category
        #[arg(short = 'C', long)]
        category: Option<String>,

        /// Optional tags to attach
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List notes for the current project
    List {
        /// Include archived notes in listing
        #[arg(long)]
        all: bool,

        /// Filter notes by specific tag
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// Show full details of a note
    Show {
        /// Note identifier (UUID or prefix)
        id: String,
    },

    /// Edit an existing note
    Edit {
        /// Note identifier
        id: String,

        /// Updated title
        #[arg(short, long)]
        title: Option<String>,

        /// Updated markdown content
        #[arg(short, long)]
        content: Option<String>,

        /// Pin this note to top of project overview
        #[arg(long, conflicts_with = "unpin")]
        pin: bool,

        /// Unpin this note
        #[arg(long, conflicts_with = "pin")]
        unpin: bool,

        /// Archive this note
        #[arg(long, conflicts_with = "unarchive")]
        archive: bool,

        /// Unarchive this note
        #[arg(long, conflicts_with = "archive")]
        unarchive: bool,
    },

    /// Delete a note (soft-delete by default)
    Delete {
        /// Note identifier
        id: String,

        /// Permanently remove note and cascade-delete child references
        #[arg(long)]
        hard: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DecisionCommands {
    /// Record a new Architecture Decision Record (ADR)
    Add {
        /// Title of the decision
        title: String,

        /// Context / problem statement driving the decision
        #[arg(short, long)]
        context: String,

        /// The technical decision taken
        #[arg(short, long)]
        decision: String,

        /// Expected consequences, trade-offs, and impacts
        #[arg(long)]
        consequences: String,

        /// Alternatives considered but rejected
        #[arg(long)]
        alternatives: Option<String>,

        /// Initial status (proposed, accepted, rejected, deprecated)
        #[arg(short, long, default_value = "accepted")]
        status: String,

        /// Optional tags
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List decisions for the current project
    List,

    /// Show full details of a decision
    Show {
        /// Decision identifier (UUID)
        id: String,
    },

    /// Update a decision status or supersede it
    Update {
        /// Decision identifier
        id: String,

        /// New status (proposed, accepted, rejected, deprecated)
        #[arg(short, long)]
        status: Option<String>,

        /// UUID of the newer decision superseding this decision
        #[arg(long)]
        superseded_by: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum MilestoneCommands {
    /// Add a new milestone
    Add {
        /// Title of the milestone
        title: String,

        /// Milestone status (planned, active, completed, cancelled)
        #[arg(short, long, default_value = "planned")]
        status: String,

        /// Target target date (ISO-8601 string)
        #[arg(long)]
        target_date: Option<String>,

        /// Description of the milestone
        #[arg(short, long)]
        description: Option<String>,

        /// Optional tags
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List milestones for the current project
    List,

    /// Complete an existing milestone
    Complete {
        /// Milestone identifier (UUID)
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    /// Add a development log entry
    Add {
        /// Title of the log event
        title: String,

        /// Event summary
        #[arg(short, long)]
        summary: String,

        /// Additional details or notes
        #[arg(short, long)]
        details: Option<String>,

        /// Event date (ISO-8601 string; defaults to now)
        #[arg(long)]
        date: Option<String>,

        /// Optional tags
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List development log entries
    List,
}

#[derive(clap::Args, Debug)]
pub struct ExportArgs {
    /// Export format target: json, markdown, or zip
    #[arg(short, long, default_value = "json")]
    pub format: String,

    /// Destination file or folder path (defaults to generated name in project root)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Redact personal usernames and machine-specific filesystem path prefixes
    #[arg(long)]
    pub redact_paths: bool,

    /// Show data inclusion summary preview without writing files
    #[arg(long)]
    pub preview: bool,
}

#[derive(clap::Args, Debug)]
pub struct ImportArgs {
    /// Path to the JSON bundle or ZIP archive file to import
    pub bundle_path: PathBuf,

    /// Strategy when incoming record ID collides: skip, overwrite, or generate-new-id
    #[arg(short = 'c', long = "on-conflict", default_value = "skip")]
    pub on_conflict: String,

    /// Validate bundle and show preview without modifying database
    #[arg(long, visible_alias = "preview")]
    pub dry_run: bool,

    /// Skip creating automatic pre-import backup snapshot
    #[arg(long)]
    pub no_backup: bool,
}

#[derive(clap::Args, Debug)]
pub struct InspectArgs {
    /// Optional path to export bundle (JSON or ZIP). If omitted, inspects local project database.
    pub file: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
pub struct DeleteDataArgs {
    /// Required confirmation flag to proceed with complete project data deletion
    #[arg(long)]
    pub force: bool,

    /// Skip creating automatic pre-deletion backup snapshot
    #[arg(long)]
    pub no_backup: bool,
}

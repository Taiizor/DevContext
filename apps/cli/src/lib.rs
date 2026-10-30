pub mod args;
pub mod commands;
pub mod output;

use args::{CliArgs, Commands};
use clap::Parser;
use devcontext_core::error::CoreError;
use output::OutputRenderer;

/// Dispatches parsed CLI arguments to the appropriate command handler.
pub fn run(args: CliArgs) -> i32 {
    let renderer = OutputRenderer::new(args.json);

    let result = match args.command {
        Some(Commands::Init { name }) => {
            commands::init::handle_init(args.project_dir, name, &renderer)
        }
        Some(Commands::Status) => commands::status::handle_status(args.project_dir, &renderer),
        Some(Commands::Doctor) => commands::doctor::handle_doctor(args.project_dir, &renderer),
        Some(Commands::Note(cmd)) => commands::note::handle_note(args.project_dir, cmd, &renderer),
        Some(Commands::Decision(cmd)) => {
            commands::decision::handle_decision(args.project_dir, cmd, &renderer)
        }
        Some(Commands::Milestone(cmd)) => {
            commands::milestone::handle_milestone(args.project_dir, cmd, &renderer)
        }
        Some(Commands::Log(cmd)) => commands::log::handle_log(args.project_dir, cmd, &renderer),
        None => {
            if args.json {
                renderer.error(
                    "NO_COMMAND",
                    "No command specified. Use --help to view available commands.",
                );
            } else {
                println!(
                    "DevContext v{} — Local-first project memory system.\nRun 'devcontext --help' for available commands.",
                    devcontext_core::version()
                );
            }
            Ok(0)
        }
    };

    match result {
        Ok(code) => code,
        Err(err) => {
            let (code, msg) = match &err {
                CoreError::NotFound(m) => ("NOT_FOUND", m.clone()),
                CoreError::Validation(m) => ("VALIDATION_ERROR", m.clone()),
                CoreError::Integrity(m) => ("INTEGRITY_ERROR", m.clone()),
                CoreError::Concurrency(m) => ("CONCURRENCY_ERROR", m.clone()),
                CoreError::Project(m) => ("PROJECT_ERROR", m.clone()),
                CoreError::Migration(m) => ("MIGRATION_ERROR", m.clone()),
                CoreError::Storage(m) => ("STORAGE_ERROR", m.clone()),
                CoreError::Sqlite(e) => ("DATABASE_ERROR", e.to_string()),
                CoreError::Io(e) => ("IO_ERROR", e.to_string()),
                CoreError::Serialization(e) => ("SERIALIZATION_ERROR", e.to_string()),
            };
            renderer.error(code, msg);
            1
        }
    }
}

/// Parses command-line iterator and executes the application.
pub fn run_from_iter<I, T>(iter: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    match CliArgs::try_parse_from(iter) {
        Ok(args) => run(args),
        Err(err) => {
            err.print().unwrap();
            if err.use_stderr() {
                2
            } else {
                0 // e.g. --help or --version
            }
        }
    }
}

pub fn main_entry() -> i32 {
    run_from_iter(std::env::args())
}

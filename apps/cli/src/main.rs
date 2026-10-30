use clap::{Parser, Subcommand};
use devcontext_core::version;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(
    name = "devcontext",
    author = "DevContext Contributors",
    version = version(),
    about = "Local-first project memory system for software development",
    long_about = None
)]
struct Cli {
    /// Emit machine-readable JSON output
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Display project context status
    Status,
    /// Diagnostic checks and version handshake
    Doctor,
}

#[derive(Serialize)]
struct StatusEnvelope<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct VersionInfo {
    version: &'static str,
    protocol_version: &'static str,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Doctor) => {
            let info = VersionInfo {
                version: version(),
                protocol_version: "1.0",
            };
            if cli.json {
                let envelope = StatusEnvelope {
                    success: true,
                    data: Some(info),
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            } else {
                println!("DevContext v{} (Protocol v1.0)", info.version);
                println!("System status: Ready for initialization.");
            }
        }
        Some(Commands::Status) => {
            if cli.json {
                let envelope = StatusEnvelope {
                    success: true,
                    data: Some("Phase 1 - Engineering Foundation"),
                    error: None,
                };
                println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            } else {
                println!("DevContext status: Foundation initialized.");
            }
        }
        None => {
            if cli.json {
                let info = VersionInfo {
                    version: version(),
                    protocol_version: "1.0",
                };
                let envelope = StatusEnvelope {
                    success: true,
                    data: Some(info),
                    error: None,
                };
                println!("{}", serde_json::to_string(&envelope).unwrap());
            } else {
                println!(
                    "DevContext v{} — Use --help to view available commands.",
                    version()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_serialization() {
        let info = VersionInfo {
            version: "0.1.0",
            protocol_version: "1.0",
        };
        let envelope = StatusEnvelope {
            success: true,
            data: Some(info),
            error: None,
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"version\":\"0.1.0\""));
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rtfm")]
#[command(author, version, about = "Read The F***ing Manual - CLI cheatsheet")]
pub struct Cli {
  /// Enable debug mode (show logs at TUI bottom)
  #[arg(short, long, global = true)]
  pub debug: bool,

  /// Command name to look up (e.g., rtfm tar)
  #[arg(value_name = "NAME")]
  pub query: Option<String>,

  /// Preferred language (e.g., en, zh)
  #[arg(short, long, default_value = "en")]
  pub lang: String,

  #[command(subcommand)]
  pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Start HTTP API server
  Serve {
    /// Listen port
    #[arg(short, long, default_value = "3030")]
    port: u16,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,
  },

  /// Update command cheatsheet data
  Update {
    /// Force update (ignore version check)
    #[arg(short, long)]
    force: bool,
  },

  /// Import local Markdown files
  Import {
    /// File or directory path
    path: String,
  },

  /// Learn a command from --help or man page
  Learn {
    /// Command to learn (e.g., rtfm learn docker)
    command: String,

    /// Force re-learn even if already exists
    #[arg(short, long)]
    force: bool,

    /// Prefer man page over --help
    #[arg(long)]
    man: bool,
  },

  /// Learn commands from the system (man pages, PowerShell, or PATH)
  LearnAll {
    /// Man section to learn (1=user commands, 8=admin commands) [Linux/macOS]
    /// On Windows: ignored, uses PowerShell cmdlets instead
    #[arg(short, long, default_value = "1")]
    section: String,

    /// Maximum number of commands to learn (0=unlimited)
    #[arg(short, long, default_value = "0")]
    limit: usize,

    /// Skip commands that already exist
    #[arg(long)]
    skip_existing: bool,

    /// Filter commands by prefix (e.g., "git" for git-*)
    #[arg(long)]
    prefix: Option<String>,

    /// Source type: "man" (Linux/macOS), "powershell" (Windows), "path" (all platforms)
    #[arg(long, default_value = "auto")]
    source: String,
  },

  /// Backup all application data (database, index, config) to archive
  Backup {
    /// Output file path (default: rtfm-backup.tar.gz)
    #[arg(short, long, default_value = "rtfm-backup.tar.gz")]
    output: String,
  },

  /// Restore application data from backup archive
  Restore {
    /// Archive file path
    path: String,

    /// Merge with existing data (default: replace all)
    #[arg(long)]
    merge: bool,
  },

  /// Reset all data (factory reset)
  Reset {
    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
  },
}

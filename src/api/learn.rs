//! Learn API - capture and index command help

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::learn;
use crate::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct LearnQuery {
  /// Command name to learn
  pub command: String,
  /// Force re-learn even if exists
  #[serde(default)]
  pub force: bool,
  /// Prefer man page over --help
  #[serde(default)]
  pub man: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LearnResponse {
  /// Whether the operation succeeded
  pub success: bool,
  /// Command name
  pub command: String,
  /// Source of help content (--help or man)
  pub source: String,
  /// Status message
  pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
  /// Error message
  pub error: String,
}

/// Learn a single command from --help or man page
#[utoipa::path(
    post,
    path = "/api/learn",
    params(LearnQuery),
    responses(
        (status = 200, description = "Learn result", body = LearnResponse),
        (status = 400, description = "Failed to learn command", body = ErrorResponse)
    ),
    tag = "Learn"
)]
pub async fn learn_command(
  State(state): State<Arc<AppState>>,
  Query(params): Query<LearnQuery>,
) -> Result<Json<LearnResponse>, Json<ErrorResponse>> {
  let command = &params.command;

  // Check if already exists
  if !params.force {
    if let Ok(Some(_)) = state.db.get_command(command, "local") {
      return Ok(Json(LearnResponse {
        success: false,
        command: command.clone(),
        source: "".to_string(),
        message: format!(
          "Command '{}' already learned. Use force=true to re-learn.",
          command
        ),
      }));
    }
  }

  // Get help content
  let (content, source) = if params.man {
    learn::get_man_page(command).or_else(|_| learn::get_help_output(command))
  } else {
    learn::get_help_output(command).or_else(|_| learn::get_man_page(command))
  }
  .map_err(|e| {
    Json(ErrorResponse {
      error: format!("Failed to get help for '{}': {}", command, e),
    })
  })?;

  // Parse help content
  let cmd = learn::parse_help_content(command, &content, &source);

  // Save to database
  state.db.save_command(&cmd).map_err(|e| {
    Json(ErrorResponse {
      error: format!("Failed to save command: {}", e),
    })
  })?;

  // Index for search
  let mut search = state.search.write().await;
  search.index_single_command(&cmd).map_err(|e| {
    Json(ErrorResponse {
      error: format!("Failed to index command: {}", e),
    })
  })?;

  Ok(Json(LearnResponse {
    success: true,
    command: command.clone(),
    source,
    message: format!("Learned '{}' successfully", command),
  }))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct LearnAllQuery {
  /// Man section (default: 1) - only used when source=man
  #[serde(default = "default_section")]
  pub section: String,
  /// Maximum commands to learn (0=unlimited)
  #[serde(default)]
  pub limit: usize,
  /// Skip existing commands
  #[serde(default)]
  pub skip_existing: bool,
  /// Filter by prefix
  pub prefix: Option<String>,
  /// Source type: "man" (Linux/macOS), "powershell" (Windows), "path" (all platforms), "auto" (default)
  #[serde(default = "default_source")]
  pub source: String,
}

fn default_section() -> String {
  "1".to_string()
}

fn default_source() -> String {
  "auto".to_string()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LearnAllResponse {
  /// Whether the operation succeeded
  pub success: bool,
  /// Total man pages found
  pub total: usize,
  /// Successfully learned count
  pub learned: usize,
  /// Skipped count (already exist)
  pub skipped: usize,
  /// Failed count
  pub failed: usize,
  /// Status message
  pub message: String,
}

/// Learn commands from the system (man pages, PowerShell, or PATH)
#[utoipa::path(
    post,
    path = "/api/learn-all",
    params(LearnAllQuery),
    responses(
        (status = 200, description = "Batch learn result", body = LearnAllResponse),
        (status = 400, description = "Failed to learn", body = ErrorResponse)
    ),
    tag = "Learn"
)]
pub async fn learn_all(
  State(state): State<Arc<AppState>>,
  Query(params): Query<LearnAllQuery>,
) -> Result<Json<LearnAllResponse>, Json<ErrorResponse>> {
  // Determine actual source based on platform
  let actual_source = if params.source == "auto" {
    #[cfg(target_os = "windows")]
    {
      "powershell"
    }
    #[cfg(not(target_os = "windows"))]
    {
      "man"
    }
  } else {
    params.source.as_str()
  };

  // Get command list based on source
  let pages = match actual_source {
    "man" => learn::list_man_pages(&params.section).map_err(|e| {
      Json(ErrorResponse {
        error: format!("Failed to list man pages: {}", e),
      })
    })?,
    "powershell" | "path" => learn::list_available_commands(actual_source).map_err(|e| {
      Json(ErrorResponse {
        error: format!("Failed to list commands: {}", e),
      })
    })?,
    _ => {
      return Err(Json(ErrorResponse {
        error: format!(
          "Unknown source '{}'. Use 'man', 'powershell', 'path', or 'auto'.",
          params.source
        ),
      }))
    }
  };

  if pages.is_empty() {
    return Ok(Json(LearnAllResponse {
      success: true,
      total: 0,
      learned: 0,
      skipped: 0,
      failed: 0,
      message: format!("No commands found for source '{}'", actual_source),
    }));
  }

  // Filter by prefix
  let pages: Vec<_> = pages
    .into_iter()
    .filter(|(name, _)| {
      if let Some(ref p) = params.prefix {
        name.to_lowercase().starts_with(&p.to_lowercase())
      } else {
        true
      }
    })
    .collect();

  // Limit
  let pages: Vec<_> = if params.limit > 0 && pages.len() > params.limit {
    pages.into_iter().take(params.limit).collect()
  } else {
    pages
  };

  let total = pages.len();
  let mut learned = 0;
  let mut skipped = 0;
  let mut failed = 0;

  let mut search = state.search.write().await;

  for (name, _) in pages {
    // Skip existing
    if params.skip_existing {
      if let Ok(Some(_)) = state.db.get_command(&name, "local") {
        skipped += 1;
        continue;
      }
    }

    // Get help content based on source
    let result = match actual_source {
      "man" => learn::get_man_page_with_section(&name, &params.section),
      _ => learn::get_help_output(&name),
    };

    match result {
      Ok((content, source)) => {
        let cmd = learn::parse_help_content(&name, &content, &source);
        if state.db.save_command(&cmd).is_ok() && search.index_single_command(&cmd).is_ok() {
          learned += 1;
        }
      }
      Err(_) => {
        failed += 1;
      }
    }
  }

  Ok(Json(LearnAllResponse {
    success: true,
    total,
    learned,
    skipped,
    failed,
    message: format!(
      "Learned {} commands from source '{}'",
      learned, actual_source
    ),
  }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupInfo {
  /// Data directory path
  pub data_dir: String,
  /// Database file size in bytes
  pub db_size: u64,
  /// Search index size in bytes
  pub index_size: u64,
  /// Total command count
  pub command_count: usize,
  /// Config file exists
  pub config_exists: bool,
}

/// Get backup info (actual backup requires CLI due to file streaming)
#[utoipa::path(
    get,
    path = "/api/backup/info",
    responses(
        (status = 200, description = "Backup info", body = BackupInfo),
        (status = 500, description = "Failed to get info", body = ErrorResponse)
    ),
    tag = "Data"
)]
pub async fn backup_info(
  State(state): State<Arc<AppState>>,
) -> Result<Json<BackupInfo>, Json<ErrorResponse>> {
  let data_dir = &state.data_dir;
  let db_path = data_dir.join(&state.config.storage.db_filename);
  let index_path = data_dir.join(&state.config.storage.index_dirname);
  let config_path = data_dir.join("config.toml");

  let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

  let index_size = if index_path.exists() {
    walkdir_size(&index_path).unwrap_or(0)
  } else {
    0
  };

  let command_count = state.db.count_commands().unwrap_or(0);
  let config_exists = config_path.exists();

  Ok(Json(BackupInfo {
    data_dir: data_dir.display().to_string(),
    db_size,
    index_size,
    command_count,
    config_exists,
  }))
}

fn walkdir_size(path: &std::path::Path) -> std::io::Result<u64> {
  let mut size = 0;
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let meta = entry.metadata()?;
    if meta.is_dir() {
      size += walkdir_size(&entry.path())?;
    } else {
      size += meta.len();
    }
  }
  Ok(size)
}

use std::sync::Arc;

use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::storage::{Command, Metadata};
use crate::update;
use crate::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListQuery {
  /// Language filter (default: zh)
  pub lang: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct CommandQuery {
  /// Language filter (default: zh)
  pub lang: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
  /// Error message
  pub error: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportResponse {
  /// Number of commands imported
  pub imported: usize,
  /// Number of files skipped (invalid format)
  pub skipped: usize,
  /// Status message
  pub message: String,
}

/// Get command by name
#[utoipa::path(
    get,
    path = "/api/command/{name}",
    params(
        ("name" = String, Path, description = "Command name"),
        CommandQuery
    ),
    responses(
        (status = 200, description = "Command details", body = Command),
        (status = 404, description = "Command not found", body = ErrorResponse)
    ),
    tag = "Commands"
)]
pub async fn get_command(
  State(state): State<Arc<AppState>>,
  Path(name): Path<String>,
  Query(params): Query<CommandQuery>,
) -> Result<Json<Command>, Json<ErrorResponse>> {
  let lang = params.lang.as_deref().unwrap_or("zh");

  match state.db.get_command(&name, lang) {
    Ok(Some(cmd)) => Ok(Json(cmd)),
    Ok(None) => Err(Json(ErrorResponse {
      error: format!("Command '{}' not found", name),
    })),
    Err(e) => Err(Json(ErrorResponse {
      error: e.to_string(),
    })),
  }
}

/// List all commands
#[utoipa::path(
    get,
    path = "/api/commands",
    params(ListQuery),
    responses(
        (status = 200, description = "List of all commands", body = Vec<Command>),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Commands"
)]
pub async fn list_commands(
  State(state): State<Arc<AppState>>,
  Query(params): Query<ListQuery>,
) -> Result<Json<Vec<Command>>, Json<ErrorResponse>> {
  let lang = params.lang.as_deref().unwrap_or("zh");

  match state.db.get_all_commands(lang) {
    Ok(commands) => Ok(Json(commands)),
    Err(e) => Err(Json(ErrorResponse {
      error: e.to_string(),
    })),
  }
}

/// Get database metadata
#[utoipa::path(
    get,
    path = "/api/metadata",
    responses(
        (status = 200, description = "Database metadata", body = Metadata),
        (status = 500, description = "Internal error", body = ErrorResponse)
    ),
    tag = "Commands"
)]
pub async fn get_metadata(
  State(state): State<Arc<AppState>>,
) -> Result<Json<Metadata>, Json<ErrorResponse>> {
  match state.db.get_metadata() {
    Ok(Some(meta)) => Ok(Json(meta)),
    Ok(None) => Ok(Json(Metadata {
      version: "0.0.0".to_string(),
      command_count: 0,
      last_update: "never".to_string(),
      languages: vec![],
    })),
    Err(e) => Err(Json(ErrorResponse {
      error: e.to_string(),
    })),
  }
}

/// Import commands from JSON
#[utoipa::path(
    post,
    path = "/api/import",
    request_body = Vec<Command>,
    responses(
        (status = 200, description = "Import successful", body = ImportResponse),
        (status = 500, description = "Import failed", body = ErrorResponse)
    ),
    tag = "Data"
)]
pub async fn import_json(
  State(state): State<Arc<AppState>>,
  Json(commands): Json<Vec<Command>>,
) -> Result<Json<ImportResponse>, Json<ErrorResponse>> {
  let count = commands.len();

  // 保存到数据库
  if let Err(e) = state.db.save_commands(&commands) {
    return Err(Json(ErrorResponse {
      error: e.to_string(),
    }));
  }

  // 重建索引
  let mut search = state.search.write().await;
  if let Err(e) = search.index_commands(&commands) {
    return Err(Json(ErrorResponse {
      error: e.to_string(),
    }));
  }

  // 更新元数据
  let meta = Metadata {
    version: chrono::Utc::now().format("%Y.%m.%d").to_string(),
    command_count: state.db.count_commands().unwrap_or(0),
    last_update: chrono::Utc::now().to_rfc3339(),
    languages: state.config.update.languages.clone(),
  };
  let _ = state.db.save_metadata(&meta);

  Ok(Json(ImportResponse {
    imported: count,
    skipped: 0,
    message: format!("Successfully imported {} commands", count),
  }))
}

/// File upload request body for import
#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct FileUpload {
  /// File to import (md, zip, tar, tar.gz, tgz)
  #[schema(value_type = String, format = Binary)]
  pub file: Vec<u8>,
}

/// Import commands from file upload (supports .md, .zip, .tar, .tar.gz, .tgz)
#[utoipa::path(
    post,
    path = "/api/import/file",
    request_body(content_type = "multipart/form-data", content = FileUpload, description = "File to import in tldr-pages format"),
    responses(
        (status = 200, description = "Import successful", body = ImportResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Import failed", body = ErrorResponse)
    ),
    tag = "Data"
)]
pub async fn import_file(
  State(state): State<Arc<AppState>>,
  mut multipart: Multipart,
) -> Result<Json<ImportResponse>, Json<ErrorResponse>> {
  let mut commands = Vec::new();
  let mut total_skipped = 0;
  let languages = &state.config.update.languages;

  while let Ok(Some(field)) = multipart.next_field().await {
    let filename = field.file_name().unwrap_or("unknown").to_string();
    let data = field.bytes().await.map_err(|e| {
      Json(ErrorResponse {
        error: format!("Failed to read file: {}", e),
      })
    })?;

    // Parse based on file extension
    let (parsed, skipped) = parse_file_data(&filename, &data, languages).map_err(|e| {
      Json(ErrorResponse {
        error: e.to_string(),
      })
    })?;

    commands.extend(parsed);
    total_skipped += skipped;
  }

  if commands.is_empty() {
    return Err(Json(ErrorResponse {
            error: "No valid Markdown files found. Files must follow tldr-pages format with description or examples.".to_string(),
        }));
  }

  let count = commands.len();

  // 保存到数据库
  if let Err(e) = state.db.save_commands(&commands) {
    return Err(Json(ErrorResponse {
      error: e.to_string(),
    }));
  }

  // 重建索引
  let mut search = state.search.write().await;
  if let Err(e) = search.index_commands(&commands) {
    return Err(Json(ErrorResponse {
      error: e.to_string(),
    }));
  }

  // 更新元数据
  let meta = Metadata {
    version: chrono::Utc::now().format("%Y.%m.%d").to_string(),
    command_count: state.db.count_commands().unwrap_or(0),
    last_update: chrono::Utc::now().to_rfc3339(),
    languages: if languages.is_empty() {
      vec!["en".to_string(), "zh".to_string()]
    } else {
      languages.clone()
    },
  };
  let _ = state.db.save_metadata(&meta);

  Ok(Json(ImportResponse {
    imported: count,
    skipped: total_skipped,
    message: format!("Successfully imported {} commands", count),
  }))
}

/// Parse file data based on filename extension
/// Returns (commands, skipped_count)
fn parse_file_data(
  filename: &str,
  data: &[u8],
  languages: &[String],
) -> anyhow::Result<(Vec<Command>, usize)> {
  let ext = std::path::Path::new(filename)
    .extension()
    .and_then(|e| e.to_str())
    .unwrap_or("")
    .to_lowercase();

  match ext.as_str() {
    "md" => {
      // Single markdown file - no language filtering (use as-is)
      let content = String::from_utf8_lossy(data);
      if let Some(cmd) = update::parse_local_markdown(&content, filename) {
        Ok((vec![cmd], 0))
      } else {
        Ok((vec![], 1))
      }
    }
    "zip" | "gz" | "tgz" | "tar" => {
      // Archive file - use parse_tldr_archive with language filtering
      match update::parse_tldr_archive(data, languages) {
        Ok(commands) => Ok((commands, 0)),
        Err(e) => Err(anyhow::anyhow!("Failed to parse archive: {}", e)),
      }
    }
    _ => {
      // Try as markdown
      let content = String::from_utf8_lossy(data);
      if let Some(cmd) = update::parse_local_markdown(&content, filename) {
        Ok((vec![cmd], 0))
      } else {
        Ok((vec![], 1))
      }
    }
  }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResetResponse {
  /// Whether reset was successful
  pub success: bool,
  /// Status message
  pub message: String,
  /// Items deleted
  pub deleted: Vec<String>,
}

/// Reset all data (factory reset)
#[utoipa::path(
    post,
    path = "/api/reset",
    responses(
        (status = 200, description = "Reset successful", body = ResetResponse),
        (status = 500, description = "Reset failed", body = ErrorResponse)
    ),
    tag = "Data"
)]
pub async fn reset_data(
  State(state): State<Arc<AppState>>,
) -> Result<Json<ResetResponse>, Json<ErrorResponse>> {
  let mut deleted = Vec::new();

  // 清空数据库中的命令
  if let Err(e) = state.db.clear_commands() {
    return Err(Json(ErrorResponse {
      error: format!("Failed to clear commands: {}", e),
    }));
  }
  deleted.push("commands".to_string());

  // 清空元数据
  let empty_meta = crate::storage::Metadata {
    version: "0.0.0".to_string(),
    command_count: 0,
    last_update: "never".to_string(),
    languages: vec![],
  };
  if let Err(e) = state.db.save_metadata(&empty_meta) {
    return Err(Json(ErrorResponse {
      error: format!("Failed to reset metadata: {}", e),
    }));
  }
  deleted.push("metadata".to_string());

  // 重建空索引
  let mut search = state.search.write().await;
  if let Err(e) = search.clear() {
    return Err(Json(ErrorResponse {
      error: format!("Failed to clear search index: {}", e),
    }));
  }
  deleted.push("search_index".to_string());

  Ok(Json(ResetResponse {
    success: true,
    message: "All data has been reset. RTFM is now in factory state.".to_string(),
    deleted,
  }))
}

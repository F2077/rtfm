use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::storage::{Command, Metadata};
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
pub async fn import_files(
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
        languages: vec!["zh".to_string(), "en".to_string()],
    };
    let _ = state.db.save_metadata(&meta);

    Ok(Json(ImportResponse {
        imported: count,
        message: format!("Successfully imported {} commands", count),
    }))
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

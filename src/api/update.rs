use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateInfo {
    /// Whether an update is available
    pub available: bool,
    /// Current installed version
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Download URL for the update
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateProgress {
    /// Update status (downloading, completed, failed)
    pub status: String,
    /// Progress percentage (0-100)
    pub progress: f32,
    /// Status message
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Check for updates
#[utoipa::path(
    get,
    path = "/api/update/check",
    responses(
        (status = 200, description = "Update check result", body = UpdateInfo),
        (status = 500, description = "Failed to check updates", body = ErrorResponse)
    ),
    tag = "Update"
)]
pub async fn check_update(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UpdateInfo>, Json<ErrorResponse>> {
    // 获取当前版本
    let current_version = state
        .db
        .get_metadata()
        .ok()
        .flatten()
        .map(|m| m.version)
        .unwrap_or_else(|| "0.0.0".to_string());

    let update_config = &state.config.update;

    // 检查 tldr-pages 最新版本
    let client = reqwest::Client::new();
    let response = client
        .get(&update_config.github_api_url)
        .header("User-Agent", &update_config.user_agent)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let release: GithubRelease = resp.json().await.map_err(|e| {
                Json(ErrorResponse {
                    error: e.to_string(),
                })
            })?;

            let latest_version = release.tag_name.trim_start_matches('v').to_string();

            // 查找 pages.zh 资源，如果没有则使用配置的下载模板
            let download_url = release
                .assets
                .iter()
                .find(|a| a.name.contains("pages") && a.name.ends_with(".zip"))
                .map(|a| a.browser_download_url.clone())
                .or_else(|| Some(update_config.download_url_template.replace("{version}", &release.tag_name)));

            Ok(Json(UpdateInfo {
                available: latest_version != current_version,
                current_version,
                latest_version,
                download_url,
            }))
        }
        Ok(resp) => Err(Json(ErrorResponse {
            error: format!("GitHub API error: {}", resp.status()),
        })),
        Err(e) => Err(Json(ErrorResponse {
            error: format!("Network error: {}", e),
        })),
    }
}

/// Download and apply updates
#[utoipa::path(
    post,
    path = "/api/update/download",
    responses(
        (status = 200, description = "Update progress", body = UpdateProgress),
        (status = 500, description = "Update failed", body = ErrorResponse)
    ),
    tag = "Update"
)]
pub async fn download_update(
    State(state): State<Arc<AppState>>,
) -> Result<Json<UpdateProgress>, Json<ErrorResponse>> {
    // 检查更新
    let update_info = match check_update(State(state.clone())).await {
        Ok(Json(info)) => info,
        Err(e) => return Err(e),
    };

    if !update_info.available {
        return Ok(Json(UpdateProgress {
            status: "completed".to_string(),
            progress: 100.0,
            message: "Already up to date".to_string(),
        }));
    }

    let download_url = match update_info.download_url {
        Some(url) => url,
        None => {
            return Err(Json(ErrorResponse {
                error: "Download URL not found".to_string(),
            }))
        }
    };

    // 下载并解析 tldr-pages
    tracing::info!("Starting download: {}", download_url);

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", &state.config.update.user_agent)
        .send()
        .await
        .map_err(|e| {
            Json(ErrorResponse {
                error: e.to_string(),
            })
        })?;

    if !response.status().is_success() {
        return Err(Json(ErrorResponse {
            error: format!("Download failed: {}", response.status()),
        }));
    }

    let bytes = response.bytes().await.map_err(|e| {
        Json(ErrorResponse {
            error: e.to_string(),
        })
    })?;

    tracing::info!("Download complete, size: {} bytes", bytes.len());

    // 解析并导入数据
    let commands = crate::update::parse_tldr_archive(&bytes).map_err(|e| {
        Json(ErrorResponse {
            error: e.to_string(),
        })
    })?;

    tracing::info!("Parse complete, command count: {}", commands.len());

    // 保存到数据库
    state.db.save_commands(&commands).map_err(|e| {
        Json(ErrorResponse {
            error: e.to_string(),
        })
    })?;

    // 重建索引
    let mut search = state.search.write().await;
    search.index_commands(&commands).map_err(|e| {
        Json(ErrorResponse {
            error: e.to_string(),
        })
    })?;

    // 更新元数据
    let meta = crate::storage::Metadata {
        version: update_info.latest_version,
        command_count: commands.len(),
        last_update: chrono::Utc::now().to_rfc3339(),
        languages: vec!["zh".to_string(), "en".to_string()],
    };
    let _ = state.db.save_metadata(&meta);

    Ok(Json(UpdateProgress {
        status: "completed".to_string(),
        progress: 100.0,
        message: format!("Successfully updated {} commands", commands.len()),
    }))
}

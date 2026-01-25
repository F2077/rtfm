mod data;
mod learn;
mod search;
mod update;

use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

/// OpenAPI 文档定义
#[derive(OpenApi)]
#[openapi(
    info(
        title = "RTFM API",
        version = "0.1.0",
        description = "Read The F***ing Manual - Offline CLI cheatsheet API",
        license(name = "GPL-3.0", url = "https://www.gnu.org/licenses/gpl-3.0.html")
    ),
    servers(
        (url = "/", description = "Current server")
    ),
    paths(
        search::search,
        data::get_command,
        data::list_commands,
        data::get_metadata,
        data::import_json,
        data::import_file,
        data::reset_data,
        update::check_update,
        update::download_update,
        learn::learn_command,
        learn::learn_all,
        learn::backup_info,
    ),
    components(schemas(
        crate::storage::Command,
        crate::storage::Example,
        crate::storage::Metadata,
        crate::search::SearchResult,
        crate::search::SearchResponse,
        search::ErrorResponse,
        data::ErrorResponse,
        data::ImportResponse,
        data::ResetResponse,
        data::FileUpload,
        update::UpdateInfo,
        update::UpdateProgress,
        update::ErrorResponse,
        learn::LearnResponse,
        learn::LearnAllResponse,
        learn::BackupInfo,
        learn::ErrorResponse,
    )),
    tags(
        (name = "Search", description = "Full-text search operations"),
        (name = "Commands", description = "Command CRUD operations"),
        (name = "Data", description = "Data import/backup/reset operations"),
        (name = "Update", description = "Update management"),
        (name = "Learn", description = "Learn commands from system help")
    )
)]
pub struct ApiDoc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/search", get(search::search))
        .route("/command/{name}", get(data::get_command))
        .route("/commands", get(data::list_commands))
        .route("/metadata", get(data::get_metadata))
        .route("/update/check", get(update::check_update))
        .route("/update/download", post(update::download_update))
        .route("/import", post(data::import_json))
        .route("/import/file", post(data::import_file).layer(DefaultBodyLimit::max(data::MAX_UPLOAD_SIZE)))
        .route("/reset", post(data::reset_data))
        // Learn endpoints
        .route("/learn", post(learn::learn_command))
        .route("/learn-all", post(learn::learn_all))
        .route("/backup/info", get(learn::backup_info))
}

/// 创建包含 Swagger UI 的完整路由
pub fn routes_with_docs() -> Router<Arc<AppState>> {
    let api_routes = routes();
    
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes)
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "Health"
)]
async fn health() -> &'static str {
    "OK"
}

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::search::SearchResponse;
use crate::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Language filter (e.g., en, zh)
    pub lang: Option<String>,
    /// Maximum results to return (default: 20, max: 100)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}

/// Search commands by keyword
#[utoipa::path(
    get,
    path = "/api/search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Invalid query", body = ErrorResponse)
    ),
    tag = "Search"
)]
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, Json<ErrorResponse>> {
    let default_limit = state.config.search.default_limit;
    let max_limit = state.config.search.max_limit;
    let limit = params.limit.unwrap_or(default_limit).min(max_limit);
    let lang = params.lang.as_deref();

    let search = state.search.read().await;
    match search.search(&params.q, lang, limit) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(Json(ErrorResponse {
            error: e.to_string(),
        })),
    }
}

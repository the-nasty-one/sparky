use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use crate::middleware::auth::AppState;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/containers", get(get_containers))
        .route("/api/v1/containers/action", post(post_container_action))
}

async fn get_containers(
    State(_state): State<AppState>,
) -> Result<Json<Vec<spark_types::ContainerSummary>>, (StatusCode, String)> {
    match spark_providers::docker::collect().await {
        Ok(containers) => Ok(Json(containers)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

async fn post_container_action(
    State(_state): State<AppState>,
    Json(action): Json<spark_types::ContainerAction>,
) -> Json<spark_types::ContainerActionResult> {
    let result =
        spark_providers::docker::execute_action(&action.container_id, &action.action).await;
    Json(result)
}

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::middleware::auth::AppState;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/containers", get(get_containers))
        .route("/api/v1/containers/action", post(container_action))
}

async fn get_containers(
    State(_state): State<AppState>,
) -> Json<Vec<spark_types::ContainerSummary>> {
    let containers = spark_providers::docker::collect().await;
    Json(containers)
}

async fn container_action(
    State(_state): State<AppState>,
    Json(body): Json<spark_types::ContainerAction>,
) -> Json<spark_types::ContainerActionResult> {
    let result = spark_providers::docker::execute_action(&body.container_id, &body.action).await;
    Json(result)
}

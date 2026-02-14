use axum::{extract::State, routing::get, Json, Router};

use crate::middleware::auth::AppState;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/api/v1/models", get(get_models))
}

async fn get_models(
    State(_state): State<AppState>,
) -> Json<Vec<spark_types::ModelEntry>> {
    let models = spark_providers::models::collect().await;
    Json(models)
}

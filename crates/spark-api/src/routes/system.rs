use axum::{
    extract::State,
    middleware,
    routing::get,
    Json, Router,
};

use crate::middleware::auth::{require_api_auth, AppState};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/system", get(get_system_metrics))
        .route("/api/v1/system/gpu", get(get_gpu_metrics))
        .route("/api/v1/system/memory", get(get_memory_metrics))
        .route_layer(middleware::from_fn_with_state(
            state,
            require_api_auth,
        ))
}

async fn get_system_metrics(
    State(_state): State<AppState>,
) -> Json<spark_types::SystemMetrics> {
    let metrics = spark_providers::collect_system_metrics().await;
    Json(metrics)
}

async fn get_gpu_metrics(
    State(_state): State<AppState>,
) -> Json<spark_types::GpuMetrics> {
    let metrics = spark_providers::gpu::collect().await;
    Json(metrics)
}

async fn get_memory_metrics(
    State(_state): State<AppState>,
) -> Json<spark_types::MemoryMetrics> {
    let metrics = spark_providers::memory::collect().await;
    Json(metrics)
}

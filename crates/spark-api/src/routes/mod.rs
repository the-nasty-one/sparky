pub mod containers;
pub mod models;
pub mod system;

use axum::Router;

use crate::middleware::auth::AppState;

pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(system::routes(state.clone()))
        .merge(containers::routes(state.clone()))
        .merge(models::routes(state))
}

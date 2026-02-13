#![allow(non_snake_case)]

pub mod middleware;
pub mod routes;

use axum::Router;

use crate::middleware::auth::AppState;

pub fn api_router(state: AppState) -> Router {
    let apiRoutes = routes::api_routes(state.clone());
    let authRoutes = middleware::auth::auth_routes(state.clone());

    Router::new()
        .merge(apiRoutes)
        .merge(authRoutes)
        .with_state(state)
}

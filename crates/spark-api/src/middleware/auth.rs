use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Redirect, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub auth_token: String,
    pub config_path: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    token: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new().route("/api/v1/auth/login", post(handle_login))
}

async fn handle_login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Response {
    if body.token != state.auth_token {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid token".into(),
            }),
        )
            .into_response();
    }

    let cookieValue = format!(
        "session_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=604800",
        body.token
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookieValue).unwrap_or_else(|_| HeaderValue::from_static("")),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"ok":true}"#))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Middleware for API routes: checks Authorization: Bearer <token> header.
pub async fn require_api_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let authHeader = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    // Also accept session_token cookie for API requests from the browser
    let cookieHeader = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let cookieToken = extract_cookie_value(cookieHeader, "session_token");

    let isAuthorized = match authHeader {
        Some(h) if h.starts_with("Bearer ") => {
            let token = &h[7..];
            token == state.auth_token
        }
        _ => cookieToken.as_deref() == Some(&state.auth_token),
    };

    if !isAuthorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".into(),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

/// Middleware for page routes: checks session_token cookie, redirects to /login if missing.
pub async fn require_page_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Exempt paths: login page, static assets, pkg files, api routes
    if path == "/login"
        || path.starts_with("/pkg/")
        || path.starts_with("/api/")
        || path.starts_with("/assets/")
    {
        return next.run(request).await;
    }

    let cookieHeader = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let cookieToken = extract_cookie_value(cookieHeader, "session_token");

    let isAuthorized = cookieToken.as_deref() == Some(&state.auth_token);

    if !isAuthorized {
        return Redirect::to("/login").into_response();
    }

    next.run(request).await
}

fn extract_cookie_value(cookieHeader: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for part in cookieHeader.split(';') {
        let trimmed = part.trim();
        if trimmed.starts_with(&prefix) {
            return Some(trimmed[prefix.len()..].to_string());
        }
    }
    None
}

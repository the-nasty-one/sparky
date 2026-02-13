#![allow(non_snake_case)]

#[cfg(feature = "ssr")]
mod config {
    use serde::Deserialize;

    #[derive(Deserialize, Clone, Debug)]
    pub struct Config {
        pub server: ServerConfig,
    }

    #[derive(Deserialize, Clone, Debug)]
    pub struct ServerConfig {
        pub bind: String,
        pub port: u16,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                server: ServerConfig {
                    bind: "0.0.0.0".into(),
                    port: 3000,
                },
            }
        }
    }

    pub fn load(path: &str) -> Config {
        match std::fs::read_to_string(path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    tracing::warn!("failed to parse config {path}: {e}, using defaults");
                    Config::default()
                }
            },
            Err(e) => {
                tracing::warn!("failed to read config {path}: {e}, using defaults");
                Config::default()
            }
        }
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use spark_api::middleware::auth::AppState;
    use spark_ui::{shell, App};
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::{fmt, EnvFilter};

    // Initialize tracing
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Parse config path from args
    let args: Vec<String> = std::env::args().collect();
    let configPath = if let Some(idx) = args.iter().position(|a| a == "--config") {
        args.get(idx + 1)
            .cloned()
            .unwrap_or_else(|| "config.example.toml".into())
    } else {
        "config.example.toml".into()
    };

    let appConfig = config::load(&configPath);
    tracing::info!(
        "loaded config from {configPath}: bind={}:{}",
        appConfig.server.bind,
        appConfig.server.port
    );

    let appState = AppState {
        config_path: configPath,
    };

    // Get Leptos configuration
    let conf = get_configuration(None).expect("failed to load Leptos configuration");
    let leptosOptions = conf.leptos_options;
    let addr = leptosOptions.site_addr;

    // Generate route list from Leptos App
    let routes = generate_route_list(App);

    // Build the API sub-router with its own state, then convert to a stateless Router
    let apiRouter = spark_api::api_router(appState);

    // Compose the full router:
    // - API routes are nested and carry their own AppState (via .with_state)
    // - Leptos routes use LeptosOptions as state
    let app = Router::new()
        .leptos_routes_with_context(
            &leptosOptions,
            routes,
            move || {},
            {
                let leptosOptions = leptosOptions.clone();
                move || shell(leptosOptions.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptosOptions)
        .merge(apiRouter)
        .layer(TraceLayer::new_for_http());

    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));
    axum::serve(listener, app.into_make_service())
        .await
        .expect("server exited with error");
}

#[cfg(not(feature = "ssr"))]
fn main() {}

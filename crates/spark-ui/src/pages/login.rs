use leptos::prelude::*;

#[server]
async fn login(token: String) -> Result<(), ServerFnError> {
    use http::header::{HeaderValue, SET_COOKIE};
    use leptos_axum::ResponseOptions;

    let expectedToken = std::env::var("SPARK_AUTH_TOKEN")
        .or_else(|_| -> Result<String, String> {
            let configPath = std::env::var("SPARK_CONFIG")
                .unwrap_or_else(|_| "/etc/spark-console/config.toml".to_string());
            let configContent =
                std::fs::read_to_string(&configPath).map_err(|e| e.to_string())?;
            let configTable: toml::Table =
                configContent.parse::<toml::Table>().map_err(|e| e.to_string())?;
            configTable
                .get("auth")
                .and_then(|a: &toml::Value| a.get("token"))
                .and_then(|t: &toml::Value| t.as_str())
                .map(|s: &str| s.to_string())
                .ok_or_else(|| "no auth token in config".to_string())
        })
        .map_err(|e| ServerFnError::new(format!("config error: {e}")))?;

    if token != expectedToken {
        return Err(ServerFnError::new("invalid token"));
    }

    let responseOptions = expect_context::<ResponseOptions>();
    let cookieValue = format!(
        "session_token={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=604800"
    );
    responseOptions.insert_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookieValue)
            .map_err(|e| ServerFnError::new(format!("cookie error: {e}")))?,
    );

    leptos_axum::redirect("/");

    Ok(())
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let loginAction = ServerAction::<Login>::new();
    let loginValue = loginAction.value();

    let hasError = move || loginValue.get().is_some_and(|result| result.is_err());

    let errorMessage = move || {
        loginValue
            .get()
            .and_then(|result| result.err())
            .map(|e| e.to_string())
            .unwrap_or_default()
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                <div class="login-header">
                    <div class="login-icon">"S"</div>
                    <h1>"Spark Console"</h1>
                    <p>"Enter your access token to continue"</p>
                </div>

                {move || {
                    hasError()
                        .then(|| {
                            view! { <div class="login-error">{errorMessage()}</div> }
                        })
                }}

                <ActionForm action=loginAction>
                    <div class="form-group">
                        <label for="token">"Access Token"</label>
                        <input
                            type="password"
                            id="token"
                            name="token"
                            placeholder="Enter your token"
                            required
                        />
                    </div>
                    <button type="submit" class="btn btn-primary">
                        "Sign In"
                    </button>
                </ActionForm>
            </div>
        </div>
    }
}

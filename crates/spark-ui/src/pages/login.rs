use leptos::prelude::*;

#[server]
async fn login(token: String) -> Result<(), ServerFnError> {
    use http::header::{HeaderValue, SET_COOKIE};
    use leptos_axum::ResponseOptions;
    use spark_types::AuthToken;

    let authToken = use_context::<AuthToken>()
        .ok_or_else(|| ServerFnError::new("auth context unavailable"))?;

    if token != authToken.0 {
        return Err(ServerFnError::new("invalid token"));
    }

    let responseOptions = expect_context::<ResponseOptions>();
    let cookieValue = format!(
        "session_token={token}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=604800"
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

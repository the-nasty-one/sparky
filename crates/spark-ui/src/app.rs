use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::nav::Nav;
use crate::components::toast::ToastProvider;
use crate::pages::containers::ContainersPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::models::ModelsPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/spark-console.css" />
        <Title text="Spark Console" />
        <ToastProvider>
            <Router>
                <Routes fallback=|| view! { <p>"Page not found."</p> }.into_any()>
                    <Route path=StaticSegment("") view=DashboardView />
                    <Route path=StaticSegment("containers") view=ContainersView />
                    <Route path=StaticSegment("models") view=ModelsView />
                </Routes>
            </Router>
        </ToastProvider>
    }
}

#[component]
fn DashboardView() -> impl IntoView {
    view! {
        <div class="app-layout">
            <Nav />
            <main class="main-content">
                <DashboardPage />
            </main>
        </div>
    }
}

#[component]
fn ContainersView() -> impl IntoView {
    view! {
        <div class="app-layout">
            <Nav />
            <main class="main-content">
                <ContainersPage />
            </main>
        </div>
    }
}

#[component]
fn ModelsView() -> impl IntoView {
    view! {
        <div class="app-layout">
            <Nav />
            <main class="main-content">
                <ModelsPage />
            </main>
        </div>
    }
}

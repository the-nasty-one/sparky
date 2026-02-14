use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Nav() -> impl IntoView {
    let location = use_location();

    let dashboardClass = move || {
        if location.pathname.get() == "/" {
            "nav-item active"
        } else {
            "nav-item"
        }
    };

    let containersClass = move || {
        if location.pathname.get() == "/containers" {
            "nav-item active"
        } else {
            "nav-item"
        }
    };

    let modelsClass = move || {
        if location.pathname.get() == "/models" {
            "nav-item active"
        } else {
            "nav-item"
        }
    };

    view! {
        <nav class="nav-sidebar">
            <div class="nav-brand">
                <div class="brand-icon">"S"</div>
                <span class="brand-text">"Spark Console"</span>
            </div>
            <ul class="nav-links">
                <li class=dashboardClass>
                    <a href="/">
                        <span class="nav-icon">"\u{25A3}"</span>
                        <span>"Dashboard"</span>
                    </a>
                </li>
                <li class=containersClass>
                    <a href="/containers">
                        <span class="nav-icon">"\u{2338}"</span>
                        <span>"Containers"</span>
                    </a>
                </li>
                <li class=modelsClass>
                    <a href="/models">
                        <span class="nav-icon">"\u{2B21}"</span>
                        <span>"Models"</span>
                    </a>
                </li>
                <li class="nav-item disabled">
                    <span>
                        <span class="nav-icon">"\u{26EE}"</span>
                        <span>"Services"</span>
                    </span>
                </li>
                <li class="nav-item disabled">
                    <span>
                        <span class="nav-icon">"\u{21BB}"</span>
                        <span>"Updates"</span>
                    </span>
                </li>
                <li class="nav-item disabled">
                    <span>
                        <span class="nav-icon">"\u{26C1}"</span>
                        <span>"Storage"</span>
                    </span>
                </li>
            </ul>
        </nav>
    }
}

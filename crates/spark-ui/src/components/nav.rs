use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="nav-sidebar">
            <div class="nav-brand">
                <div class="brand-icon">"S"</div>
                <span class="brand-text">"Spark Console"</span>
            </div>
            <ul class="nav-links">
                <li class="nav-item active">
                    <a href="/">
                        <span class="nav-icon">"\u{25A3}"</span>
                        <span>"Dashboard"</span>
                    </a>
                </li>
                <li class="nav-item">
                    <a href="/containers">
                        <span class="nav-icon">"\u{2338}"</span>
                        <span>"Containers"</span>
                    </a>
                </li>
                <li class="nav-item disabled">
                    <span>
                        <span class="nav-icon">"\u{2B21}"</span>
                        <span>"Models"</span>
                    </span>
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

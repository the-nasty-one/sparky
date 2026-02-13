use leptos::prelude::*;

/// Dark card wrapper component with a title header.
/// Used to wrap gauge components and metric displays.
#[component]
pub fn MetricCard(
    /// Title displayed at the top of the card
    title: String,
    /// Card content (typically a Gauge or metric rows)
    children: Children,
) -> impl IntoView {
    view! {
        <div class="card">
            <div class="card-title">{title}</div>
            {children()}
        </div>
    }
}

pub mod system;
pub use system::*;

/// Auth token wrapper for sharing via Leptos context.
#[derive(Clone, Debug)]
pub struct AuthToken(pub String);

#![allow(non_snake_case)]

pub mod app;
pub mod components;
pub mod pages;

pub use app::{shell, App};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

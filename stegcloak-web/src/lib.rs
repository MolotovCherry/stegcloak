mod app;
mod components;
mod pages;

use std::sync::OnceLock;

use leptos::*;

pub use app::App;

static BASE_PATH: OnceLock<String> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Panic(pub String);

pub fn get_base_url() -> String {
    BASE_PATH
        .get_or_init(|| document().location().unwrap().pathname().unwrap())
        .clone()
}

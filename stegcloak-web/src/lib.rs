mod app;
mod components;
mod pages;

pub use app::App;

#[derive(Debug, Clone)]
pub struct Panic(pub String);

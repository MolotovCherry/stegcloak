use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{
    use_color_mode_with_options, use_preferred_dark, ColorMode, UseColorModeOptions,
    UseColorModeReturn,
};

use crate::{
    components::page_base::PageBase,
    get_base_url,
    pages::{home::Home, not_found::NotFound},
};

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // theme color handler
    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode_with_options(
        UseColorModeOptions::default()
            .storage_key("theme")
            .attribute("data-theme")
            .emit_auto(true),
    );

    let is_dark_preferred = use_preferred_dark();
    let get_theme = GetTheme {
        mode,
        is_dark_preferred,
    };
    let set_theme = SetTheme { set_mode };

    provide_context(get_theme);
    provide_context(set_theme);

    let base: &'static str = get_base_url().leak();
    let base: &'static str = base
        .strip_prefix('/')
        .unwrap_or(base)
        .strip_suffix('/')
        .unwrap_or(base);

    view! {
        <Html lang="en"/>

        <Title text="Stegcloak"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <PageBase>
            <Router base>
                <Routes base=base.to_owned()>
                    <Route path="/" view=Home/>
                    <Route path="/*" view=NotFound/>
                </Routes>
            </Router>
        </PageBase>
    }
}

#[derive(Copy, Clone)]
pub struct SetTheme {
    set_mode: WriteSignal<ColorMode>,
}

impl SetTheme {
    pub fn set_auto(&self) {
        self.set_mode.set(ColorMode::Auto);
    }

    pub fn set_mode(&self, mode: &str) {
        let mode = mode.to_lowercase();
        let mode = match &*mode {
            "dark" => ColorMode::Dark,
            "light" => ColorMode::Light,
            theme => ColorMode::Custom(theme.to_owned()),
        };
        self.set_mode.set(mode);
    }
}

#[derive(Copy, Clone)]
pub struct GetTheme {
    mode: Signal<ColorMode>,
    is_dark_preferred: Signal<bool>,
}

impl GetTheme {
    pub fn theme(&self) -> ColorMode {
        self.mode.get()
    }

    pub fn is_auto(&self) -> bool {
        matches!(self.mode.get(), ColorMode::Auto)
    }

    // returns string display of theme
    pub fn theme_string(&self) -> String {
        match self.mode.get() {
            ColorMode::Auto => "auto".to_owned(),
            ColorMode::Light => "light".to_owned(),
            ColorMode::Dark => "dark".to_owned(),
            ColorMode::Custom(theme) => theme,
        }
    }

    // checks if dark mode
    pub fn is_dark_preferred(&self) -> bool {
        self.is_dark_preferred.get()
    }

    pub fn is_dark_theme(&self) -> bool {
        match self.mode.get() {
            ColorMode::Auto => self.is_dark_preferred.get(),
            ColorMode::Light => false,
            ColorMode::Dark => true,
            ColorMode::Custom(theme) => match &*theme {
                "dracula" => true,
                "night" => true,
                "dim" => true,
                "cupcake" => false,
                "valentine" => false,
                _ => unreachable!(),
            },
        }
    }

    pub fn themes(&self) -> &'static [&'static str] {
        &[
            "light",
            "dark",
            "dracula",
            "night",
            "dim",
            "cupcake",
            "valentine",
        ]
    }
}

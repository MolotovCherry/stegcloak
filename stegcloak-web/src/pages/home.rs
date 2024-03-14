use leptos::*;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    view! {
        <picture>
            <source
                srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_pref_dark_RGB.svg"
                media="(prefers-color-scheme: dark)"
            />
            <img
                src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg"
                alt="Leptos Logo"
                height="200"
                width="400"
            />
        </picture>

        <h1>"Welcome to Leptos"</h1>
    }
}

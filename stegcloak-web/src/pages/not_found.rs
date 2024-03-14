use leptos::*;
use leptos_router::use_location;

/// 404 Not Found Page
#[component]
pub fn NotFound() -> impl IntoView {
    let document = document();
    let base = document.location().unwrap().origin().unwrap();

    let location = use_location();

    view! {
        <div class="items-center justify-center flex h-full">
            <div class="mockup-browser border border-base-content bg-base-300 w-96">
                <div class="mockup-browser-toolbar">
                    <div class="input">{base}{move || location.pathname}</div>
                </div>
                <div class="flex justify-center px-4 py-16 bg-base-200">
                    "404 Not Found"
                </div>
            </div>
        </div>
    }
}

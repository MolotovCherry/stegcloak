use std::collections::HashMap;

use js_sys::Date;
use leptos::*;

use crate::{
    app::{GetTheme, SetTheme},
    components::mockup_code::{MockupCode, Prefix},
    get_base_url, Panic,
};

#[component]
pub fn PageBase(children: ChildrenFn) -> impl IntoView {
    let panic = use_context::<ReadSignal<Option<Panic>>>().unwrap();

    // regular error boundary error notification
    let fallback = |errors: RwSignal<Errors>| {
        let (_errors, _) = errors.split();
        let errors = move || {
            std::iter::once("trunk serve --release".to_owned())
                .chain(errors.get().into_iter().map(|(_, error)| error.to_string()))
                .chain(std::iter::once("Error! (code: 500)".to_owned()))
                .collect()
        };

        let error_highlight = move || {
            let num = _errors.get().iter().count() + 2;
            let mut hm = HashMap::new();

            // error display on all center lines
            for n in 2..num {
                hm.insert(n, "text-error".to_owned());
            }

            // we added one extra at the end
            hm.insert(num, "bg-error text-error-content".to_owned());
            hm
        };

        view! {
            <div class="flex items-center justify-center h-full">
                <div class="my-8 flex flex-col items-center justify-center">
                    <div class="p-4">
                        <h1 class="text-center text-4xl mb-2">"Oops!"</h1>
                        <p class="text-center text-xl">"Something went wrong."</p>
                    </div>

                    <MockupCode lines=errors prefixes=Prefix::UnixShell line_classes=error_highlight />
                </div>
            </div>
        }
    };

    // panic page notification
    let panic_impl = |panic: Panic| {
        let lines = move || {
            vec![
                "trunk serve --release".to_owned(),
                panic.0.clone(),
                "Error! (code: 500)".to_owned(),
            ]
        };

        let highlight = move || {
            let mut hm = HashMap::new();

            // error display on all center lines
            hm.insert(2, "text-error".to_owned());

            // we added one extra at the end
            hm.insert(3, "bg-error text-error-content".to_owned());
            hm
        };

        view! {
            <div class="flex items-center justify-center h-full">
                <div class="my-8 flex flex-col items-center justify-center">
                    <div class="p-4">
                        <h1 class="text-center text-4xl mb-2">"Oops!"</h1>
                        <p class="text-center text-xl">"Something went wrong."</p>
                    </div>

                    <MockupCode lines prefixes=Prefix::UnixShell line_classes=highlight />
                </div>
            </div>
        }
    };

    view! {
        <div
            class="flex flex-col items-center justify-center"
            // safari click fix
            // https://github.com/leptos-rs/leptos/issues/2381
            // https://stackoverflow.com/questions/24077725/mobile-safari-sometimes-does-not-trigger-the-click-event/39712411#39712411
            onclick="void(0);"
        >
            <div class="drop-shadow-lg md:my-8 sm:my-0 bg-base-300 rounded w-full md:w-[768px]">
                // Navbar
                <Navbar/>

                // Main content
                <main class="container p-4 text-base-content">
                    <ErrorBoundary fallback>
                        {move || {
                            if let Some(panic) = panic.get() {
                                // display panic page
                                panic_impl(panic).into_view()
                            } else {
                                // Regular page content
                                view! {
                                    {children()}
                                }.into_view()
                            }
                        }}
                    </ErrorBoundary>
                </main>

                // Footer
                <Footer/>
            </div>
        </div>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    let set_theme = use_context::<SetTheme>().unwrap();
    let get_theme = use_context::<GetTheme>().unwrap();

    view! {
        <div class="navbar bg-neutral text-neutral-content rounded-t">
            // title
            <div class="flex-1 px-2 lg:flex-none">
                <span class="text-xl font-bold">"StegCloak 🧙🏻‍♂️"</span>
            </div>

            // theme dropdown
            <div class="flex justify-end flex-1 px-2">
                <div class="flex items-stretch">
                    <div class="dropdown dropdown-end">
                        <div tabindex="0" role="button" class="btn btn-ghost rounded-btn">
                            "Theme"
                            <svg width="12px" height="12px" class="h-2 w-2 fill-current opacity-60 inline-block" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2048 2048"><path d="M1799 349l242 241-1017 1017L7 590l242-241 775 775 775-775z"></path></svg>
                        </div>
                        <ul
                            tabindex="0"
                            class="menu dropdown-content z-[1] p-2 shadow bg-neutral rounded-box w-30 mt-4"
                        >
                            // auto
                            <li>
                                <input
                                    type="radio"
                                    name="theme-dropdown"
                                    checked=move || get_theme.is_auto()
                                    class="theme-controller btn btn-sm btn-block btn-ghost justify-start"
                                    aria-label="auto"
                                    value=move || {
                                        if get_theme.is_dark_preferred() { "dark" } else { "light" }
                                    }

                                    on:click=move |_| {
                                        set_theme.set_auto()
                                    }
                                />
                            </li>

                            {move || {
                                get_theme.themes()
                                    .iter()
                                    .map(|&theme| {
                                        view! {
                                            <li>
                                                <input
                                                    type="radio"
                                                    name="theme-dropdown"
                                                    class="theme-controller btn btn-sm btn-block btn-ghost justify-start"
                                                    checked=move || get_theme.theme_string() == theme
                                                    aria-label=theme
                                                    value=theme
                                                    on:click=move |_| {
                                                        set_theme.set_mode(theme)
                                                    }
                                                />
                                            </li>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Footer() -> impl IntoView {
    let get_theme = use_context::<GetTheme>().unwrap();

    let is_footer_dark_mode = move || match &*get_theme.theme_string() {
        // the footer itself is dark
        "light" => true,
        "dark" => true,
        "dracula" => true,
        "night" => true,
        "dim" => true,
        "cupcake" => true,
        "valentine" => true,
        // because both light and dark theme have a dark footer
        "auto" => true,
        _ => unreachable!(),
    };

    let base = get_base_url();

    view! {
        <footer class="footer items-center p-4 bg-neutral text-neutral-content rounded-b">
            <aside class="items-center grid-flow-col">
                <p>"Copyright © " {Date::new_0().get_full_year()} " Cherry 🍒"</p>
            </aside>

            <nav class="grid-flow-col gap-4 md:place-self-center md:justify-self-end">
                <a href="https://github.com/MolotovCherry" target="_blank">
                    {move || {
                        if is_footer_dark_mode() {
                            view! {
                                <img src=format!("{base}static/image/github-mark-white.svg") height="24" width="24" />
                            }
                        } else {
                            view! {
                                <img src=format!("{base}static/image/github-mark.svg") height="24" width="24" />
                            }
                        }
                    }}
                </a>
            </nav>
        </footer>
    }
}

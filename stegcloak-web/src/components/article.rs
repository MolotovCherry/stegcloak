use leptos::*;

#[component]
pub fn Article(children: Children) -> impl IntoView {
    view! {
        <article class="prose">
            {children()}
        </article>
    }
}

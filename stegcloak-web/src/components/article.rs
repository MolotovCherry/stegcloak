use leptos::*;

#[component]
pub fn Article(
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "prose".to_owned())] prose: String,
    children: Children,
) -> impl IntoView {
    view! {
        <article class=format!("{class} {prose}")>
            {children()}
        </article>
    }
}

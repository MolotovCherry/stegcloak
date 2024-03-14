use leptos::*;
use web_sys::SubmitEvent;

use crate::components::article::Article;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let on_submit = move |evt: SubmitEvent| {
        evt.prevent_default();
    };

    view! {
        <Article class="w-full justify-center content px-6" prose="">
            // header
            <h1 class="text-center text-3xl font-bold block">"StegCloak"</h1>

            // hide
            <div class="mt-12">
                <h2 class="text-center text-2xl font-bold">"HIDE"</h2>
                <hr class="mt-3"/>
            </div>

            // secret password buttons
            <form on:submit=on_submit class="mt-6 text-center">
                <div>
                    <input type="text" placeholder="Secret" class="input input-bordered input-primary max-w-30" />
                    <input type="text" placeholder="Password" class="input input-bordered input-primary max-w-30 ml-3" />
                </div>

                <div class="mt-6">
                    <span>Message</span>
                    <textarea placeholder="This is a confidential message." class="textarea textarea-bordered textarea-md w-full max-w-lg" ></textarea>
                </div>

                <div class="mt-6">
                    <input type="submit" class="btn btn-active btn-primary" value="Submit"/>
                </div>
            </form>

        </Article>
    }
}

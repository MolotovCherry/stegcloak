use leptos::html::{Input, Textarea};
use leptos::*;
use leptos_use::{use_clipboard, use_permission, PermissionState, UseClipboardReturn};
use web_sys::{HtmlTextAreaElement, SubmitEvent};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Tab {
    Hide,
    Reveal,
}

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(Tab::Hide);

    view! {
        <div class="w-full h-full flex flex-col justify-center items-center content px-6">
            // selector
            <div role="tablist" class="tabs tabs-boxed max-w-sm mx-auto">
                <a
                    role="tab"
                    on:click=move |_| set_active_tab.set(Tab::Hide)
                    class="tab"
                    class:tab-active=move || active_tab.get() == Tab::Hide
                >
                    "Hide"
                </a>

                <a
                    role="tab"
                    on:click=move |_| set_active_tab.set(Tab::Reveal)
                    class="tab"
                    class:tab-active=move || active_tab.get() == Tab::Reveal
                >
                    "Reveal"
                </a>
            </div>

            // secret password buttons
            <Hide active_tab />

            // reveal
            <Reveal active_tab />
        </div>
    }
}

#[component]
fn Reveal(active_tab: ReadSignal<Tab>) -> impl IntoView {
    let on_submit_reveal = move |evt: SubmitEvent| {
        evt.prevent_default();
    };

    view! {
        <form on:submit=on_submit_reveal class="mt-6 text-center" class:hidden=move || active_tab.get() != Tab::Reveal>
            <input type="text" placeholder="Secret" class="input input-bordered input-primary max-w-30" />
            <input type="text" placeholder="Password" class="input input-bordered input-primary max-w-30 ml-3" />
        </form>
    }
}

#[component]
fn Hide(active_tab: ReadSignal<Tab>) -> impl IntoView {
    let UseClipboardReturn {
        is_supported,
        text,
        copied,
        copy,
    } = use_clipboard();

    let secret: NodeRef<Input> = create_node_ref();
    let password: NodeRef<Input> = create_node_ref();
    let message: NodeRef<Textarea> = create_node_ref();

    let (cloaked_msg, set_cloaked_msg) = create_signal(String::new());
    let (encrypt, set_encrypt) = create_signal(true);
    let (hmac, set_hmac) = create_signal(false);

    let permission_write = use_permission("clipboard-write");

    let on_submit_hide = move |evt: SubmitEvent| {
        evt.prevent_default();

        let message_target = message.get_untracked().unwrap();

        let secret = secret.get_untracked().unwrap().value();
        let password = password.get_untracked().unwrap().value();
        let message = message.get_untracked().unwrap().value();

        if message.contains(' ') {
            message_target.set_custom_validity("");
            message_target.report_validity();
        } else {
            // validation failed, so return here
            message_target.set_custom_validity("Cover text requires 2 words minimum");
            message_target.report_validity();
            return;
        }

        let encrypt = encrypt.get_untracked();
        let hmac = hmac.get_untracked();

        let hidden = if encrypt {
            stegcloak::encrypt::hide(secret, password, hmac, message).unwrap()
        } else {
            stegcloak::plaintext::hide(secret, message).unwrap()
        };

        set_cloaked_msg.set(hidden);
    };

    view! {
        <form on:submit=on_submit_hide class="mt-6 text-center" class:hidden=move || active_tab.get() != Tab::Hide>
            <div>
                <input
                    type="text"
                    placeholder="Secret"
                    class="input input-bordered input-primary max-w-30"
                    node_ref=secret
                    required
                    min-length=1
                />

                <input
                    type="text"
                    placeholder="Password"
                    class="input input-bordered input-primary max-w-30 ml-3"
                    node_ref=password
                    min-length=1
                    required=move|| encrypt.get()
                    disabled=move || !encrypt.get()
                />
            </div>

            <div class="form-control flex flex-row">
                <label class="cursor-pointer label justify-normal w-fit">
                    <span class="label-text pr-2">"ENCRYPT"</span>
                    <input
                        type="checkbox"
                        checked
                        on:click=move |ev| set_encrypt.set(event_target_checked(&ev))
                        class="checkbox checkbox-secondary checkbox-sm"
                    />
                </label>

                <label class="cursor-pointer label justify-normal w-fit">
                    <span class="label-text pr-2">"HMAC"</span>
                    <input
                        type="checkbox"
                        class="checkbox checkbox-secondary checkbox-sm"
                        disabled=move || !encrypt.get()
                        on:click=move |ev| set_hmac.set(event_target_checked(&ev))
                    />
                </label>
            </div>

            <div class="mt-6">
                <div class="text-left mb-2">"MESSAGE"</div>
                <textarea
                    placeholder="This is a confidential message."
                    class="textarea textarea-bordered textarea-md w-full max-w-lg"
                    node_ref=message
                    on:input=move |ev| {
                        // erase the validity since it's annoying when it pops up every time you type
                        let target: HtmlTextAreaElement = event_target(&ev);
                        target.set_custom_validity("");
                        target.report_validity();
                    }
                >
                </textarea>
            </div>

            <div class="mt-6">
                <div class="mb-2 w-full flex justify-between items-center">
                    <div class="text-left">
                        "CLOAKED MESSAGE"
                    </div>
                    <div class="text-right">
                        <button
                            type="button"
                            class="btn btn-sm btn-outline btn-secondary"
                            disabled=move || permission_write.get() != PermissionState::Granted || !is_supported.get()
                            on:click=move |_| copy(&cloaked_msg.get_untracked())
                        >
                            Copy
                        </button>
                    </div>
                </div>

                <textarea
                    class="textarea textarea-bordered textarea-md w-full max-w-lg"
                    prop:value=move || cloaked_msg.get()
                    readonly
                ></textarea>
            </div>

            <div class="mt-6">
                <input type="submit" class="btn btn-active btn-primary" value="Hide"/>
            </div>
        </form>
    }
}
use leptos::html::{Form, Input, Textarea};
use leptos::*;
use leptos_use::{use_clipboard, use_permission, PermissionState, UseClipboardReturn};
use stegcloak::{crypto::DeEncryptError, StegError};
use web_sys::{HtmlTextAreaElement, SubmitEvent};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Tab {
    Cloak,
    Reveal,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum PwType {
    Text,
    Password,
}

impl PwType {
    fn to_str(self) -> &'static str {
        match self {
            PwType::Text => "text",
            PwType::Password => "password",
        }
    }

    fn flip(self) -> Self {
        match self {
            PwType::Text => PwType::Password,
            PwType::Password => PwType::Text,
        }
    }
}

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let (active_tab, set_active_tab) = create_signal(Tab::Cloak);

    view! {
        <div class="w-full h-full flex flex-col justify-center items-center content px-6">
            // selector
            <div role="tablist" class="tabs tabs-boxed w-full max-w-md md:max-w-sm md:w-auto md:mx-auto">
                <a
                    role="tab"
                    on:click=move |_| set_active_tab.set(Tab::Cloak)
                    class="tab"
                    class:tab-active=move || active_tab.get() == Tab::Cloak
                >
                    "Cloak"
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
            <Cloak active_tab />

            // reveal
            <Reveal active_tab />
        </div>
    }
}

#[component]
fn Reveal(active_tab: ReadSignal<Tab>) -> impl IntoView {
    let UseClipboardReturn {
        is_supported,
        copied,
        copy,
        ..
    } = use_clipboard();

    let (pw_type, set_pw_type) = create_signal(PwType::Password);
    let (secret, set_secret) = create_signal(String::new());
    let (show, set_show) = create_signal(false);

    let password: NodeRef<Input> = create_node_ref();
    let form: NodeRef<Form> = create_node_ref();
    let message: NodeRef<Textarea> = create_node_ref();

    let permission_write = use_permission("clipboard-write");

    let on_submit = move |evt: SubmitEvent| {
        evt.prevent_default();

        let message_target = message.get_untracked().unwrap();
        let password_target = password.get_untracked().unwrap();

        password_target.set_custom_validity("");
        password_target.report_validity();
        message_target.set_custom_validity("");
        message_target.report_validity();

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

        if !password.is_empty() {
            // try encrypted mode
            let result = stegcloak::encrypt::reveal(&password, &message);

            if let Err(StegError::DeEncryptError(DeEncryptError::IncorrectPassword)) = result {
                password_target.set_custom_validity("Incorrect password");
                password_target.report_validity();
                return;
            }

            if let Err(StegError::DeEncryptError(DeEncryptError::IntegrityError)) = result {
                message_target.set_custom_validity("Message integrity check failed");
                message_target.report_validity();
                return;
            }

            if let Err(StegError::DeEncryptError(e)) = result {
                password_target
                    .set_custom_validity("This message is not encrypted, try removing this");
                password_target.report_validity();
                log::error!("Failed decryption: {e:?}");
                return;
            }

            let Ok(data) = result else {
                message_target.set_custom_validity("Message is corrupted");
                message_target.report_validity();
                log::error!("Failed decryption: {:?}", result.unwrap_err());
                return;
            };

            set_secret.set(data);
        } else {
            // try plaintext mode
            let data = match stegcloak::plaintext::reveal(message) {
                Ok(data) => data,
                Err(e) => match e {
                    StegError::DeCompressError(e) => {
                        message_target.set_custom_validity(
                            "This is either encrypted or corrupted. Try inputting a password",
                        );
                        message_target.report_validity();
                        log::error!("Failed plaintext decoding: {e:?}");
                        return;
                    }
                    StegError::CodecError(e) => {
                        message_target.set_custom_validity("Message is corrupted");
                        message_target.report_validity();
                        log::error!("Failed plaintext decoding: {e:?}");
                        return;
                    }
                    _ => unreachable!(),
                },
            };

            set_secret.set(data);
        }
    };

    view! {
        <form on:submit=on_submit class="mt-6 text-center w-full max-w-md" class:hidden={move || active_tab.get() != Tab::Reveal} node_ref=form>
            <label class="input input-bordered input-primary flex items-center gap-2">
                // password icon
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 flex-none"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 0 1 3 3m3 0a6 6 0 0 1-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1 1 21.75 8.25Z" /></svg>

                <input
                    type=move || pw_type.get().to_str()
                    placeholder="Password"
                    class="w-full"
                    node_ref=password
                    on:input=move |ev| {
                        // erase the validity since it's annoying when it pops up every time you type
                        let target: HtmlTextAreaElement = event_target(&ev);
                        target.set_custom_validity("");
                        target.report_validity();

                        // this needs to be reset in case we got a previous error about using plaintext mode for encrypted stuff
                        let message_target = message.get_untracked().unwrap();
                        message_target.set_custom_validity("");
                        message_target.report_validity();
                    }
                />

                // show eye
                <svg
                    class:hidden=move || show.get()
                    on:click=move |_| {
                        set_pw_type.set(pw_type.get_untracked().flip());
                        set_show.set(true);
                    }
                    class="w-5 h-5 flex-none cursor-pointer"
                    xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                </svg>

                // hide eye
                <svg
                    class:hidden=move || !show.get()
                    on:click=move |_| {
                        set_pw_type.set(pw_type.get_untracked().flip());
                        set_show.set(false);
                    }
                    class="w-5 h-5 flex-none cursor-pointer"
                    xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 0 0 1.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.451 10.451 0 0 1 12 4.5c4.756 0 8.773 3.162 10.065 7.498a10.522 10.522 0 0 1-4.293 5.774M6.228 6.228 3 3m3.228 3.228 3.65 3.65m7.894 7.894L21 21m-3.228-3.228-3.65-3.65m0 0a3 3 0 1 0-4.243-4.243m4.242 4.242L9.88 9.88" />
                </svg>

            </label>

            <div class="mt-6">
                <div class="text-left mb-2">"COVER MESSAGE"</div>
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
                        "SECRET"
                    </div>
                    <div class="text-right">
                        <button
                            type="button"
                            class="btn btn-sm btn-outline"
                            class:btn-secondary=move || !copied.get()
                            class:btn-success=move || copied.get()
                            disabled=move || permission_write.get() != PermissionState::Granted || !is_supported.get()
                            on:click=move |_| copy(&secret.get_untracked())
                        >
                            "Copy"
                        </button>
                    </div>
                </div>

                <textarea
                    class="textarea textarea-bordered textarea-md w-full max-w-lg textarea-accent"
                    prop:value=move || secret.get()
                    readonly
                ></textarea>
            </div>

            <div class="mt-6">
                <button
                    type="button"
                    class="btn btn-sm btn-outline mr-2"
                    on:click=move |_| form.get_untracked().unwrap().reset()
                >
                    "Clear"
                </button>

                <input
                    type="submit"
                    class="btn btn-sm btn-outline btn-primary"
                    value="Reveal"
                />
            </div>
        </form>
    }
}

#[component]
fn Cloak(active_tab: ReadSignal<Tab>) -> impl IntoView {
    let UseClipboardReturn {
        is_supported,
        copied,
        copy,
        ..
    } = use_clipboard();

    let secret: NodeRef<Input> = create_node_ref();
    let password: NodeRef<Input> = create_node_ref();
    let message: NodeRef<Textarea> = create_node_ref();
    let form: NodeRef<Form> = create_node_ref();

    let (pw_type, set_pw_type) = create_signal(PwType::Password);
    let (cloaked_msg, set_cloaked_msg) = create_signal(String::new());
    let (encrypt, set_encrypt) = create_signal(true);
    let (hmac, set_hmac) = create_signal(false);
    let (show, set_show) = create_signal(false);

    let permission_write = use_permission("clipboard-write");

    let on_submit = move |evt: SubmitEvent| {
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
        <form on:submit=on_submit class="mt-6 text-center w-full max-w-md" class:hidden={move || active_tab.get() != Tab::Cloak} node_ref=form>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 gap-y-0">
                <label class="input input-bordered input-primary flex items-center max-w-30 gap-2 my-4 md:my-0">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 flex-none"><path stroke-linecap="round" stroke-linejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" /></svg>

                    <input
                        type="text"
                        placeholder="Secret"
                        class="w-full"
                        node_ref=secret
                        required
                        min-length=1
                    />
                </label>

                // password
                <label class="input input-bordered input-primary flex items-center max-w-30 gap-2 my-4 md:my-0" disabled=move || !encrypt.get()>
                    // password icon
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5 flex-none"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25a3 3 0 0 1 3 3m3 0a6 6 0 0 1-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1 1 21.75 8.25Z" /></svg>

                    <input
                        type=move || pw_type.get().to_str()
                        placeholder="Password"
                        class="w-full"
                        node_ref=password
                        min-length=1
                        required=move || encrypt.get()
                        disabled=move || !encrypt.get()
                    />

                    // show eye
                    <svg
                        class:hidden=move || show.get()
                        class:cursor-pointer=move || encrypt.get()
                        on:click=move |_| {
                            // disable click if encrypt not enabled
                            if !encrypt.get_untracked() {
                                return;
                            }

                            set_pw_type.set(pw_type.get_untracked().flip());
                            set_show.set(true);
                        }
                        class="w-5 h-5 flex-none"
                        xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                    </svg>

                    // hide eye
                    <svg
                        class:hidden=move || !show.get()
                        class:cursor-pointer=move || encrypt.get()
                        on:click=move |_| {
                            // disable click if encrypt not enabled
                            if !encrypt.get_untracked() {
                                return;
                            }

                            set_pw_type.set(pw_type.get_untracked().flip());
                            set_show.set(false);
                        }
                        class="w-5 h-5 flex-none"
                        xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 0 0 1.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.451 10.451 0 0 1 12 4.5c4.756 0 8.773 3.162 10.065 7.498a10.522 10.522 0 0 1-4.293 5.774M6.228 6.228 3 3m3.228 3.228 3.65 3.65m7.894 7.894L21 21m-3.228-3.228-3.65-3.65m0 0a3 3 0 1 0-4.243-4.243m4.242 4.242L9.88 9.88" />
                    </svg>
                </label>

                // encrypt / hmac checkboxes
                <div class="flex flex-row items-center">
                    <label class="cursor-pointer label justify-normal w-fit">
                        <span class="label-text pr-2">"ENCRYPT"</span>
                        <input
                            type="checkbox"
                            checked
                            on:click=move |ev| set_encrypt.set(event_target_checked(&ev))
                            class="checkbox checkbox-secondary checkbox-sm"
                        />
                    </label>

                    <label class:cursor-pointer={move|| encrypt.get()} class="label justify-normal w-fit">
                        <span class="label-text pr-2">"HMAC"</span>
                        <input
                            type="checkbox"
                            class="checkbox checkbox-secondary checkbox-sm"
                            disabled=move || !encrypt.get()
                            on:click=move |ev| set_hmac.set(event_target_checked(&ev))
                        />
                    </label>
                </div>
            </div>

            // cover message textarea
            <div class="mt-6">
                <div class="text-left mb-2">"COVER MESSAGE"</div>
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

            // cloaked message textarea
            <div class="mt-6">
                <div class="mb-2 w-full flex justify-between items-center">
                    <div class="text-left">
                        "CLOAKED MESSAGE"
                    </div>
                    <div class="text-right">
                        <button
                            type="button"
                            class="btn btn-sm btn-outline"
                            class:btn-secondary=move || !copied.get()
                            class:btn-success=move || copied.get()
                            disabled=move || permission_write.get() != PermissionState::Granted || !is_supported.get()
                            on:click=move |_| copy(&cloaked_msg.get_untracked())
                        >
                            "Copy"
                        </button>
                    </div>
                </div>

                <textarea
                    class="textarea textarea-bordered textarea-md w-full max-w-lg textarea-accent"
                    prop:value=move || cloaked_msg.get()
                    readonly
                ></textarea>
            </div>

            // clear / submit buttons
            <div class="mt-6">
                <button
                    type="button"
                    class="btn btn-sm btn-outline mr-2"
                    on:click=move |_| {
                        form.get_untracked().unwrap().reset();
                        // this is important, because default form state encrypt is true
                        // if this is not set, then state will not sync
                        set_encrypt.set(true);
                    }
                >
                    "Clear"
                </button>

                <input type="submit" class="btn btn-sm btn-outline btn-primary" value="Cloak"/>
            </div>
        </form>
    }
}

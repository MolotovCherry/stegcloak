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
            <div>
                <input
                    type=move || pw_type.get().to_str()
                    placeholder="Password"
                    class="input input-bordered input-primary w-full"
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

                <div>
                    <label class="cursor-pointer label justify-normal w-fit">
                        <span class="label-text pr-2">"Show"</span>
                        <input
                            type="checkbox"
                            class="checkbox checkbox-secondary checkbox-sm"
                            on:click=move |_| {
                                set_pw_type.set(pw_type.get_untracked().flip());
                            }
                        />
                    </label>
                </div>
            </div>

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
                <input
                    type="text"
                    placeholder="Secret"
                    class="input input-bordered input-primary max-w-30"
                    node_ref=secret
                    required
                    min-length=1
                />

                <input
                    type=move || pw_type.get().to_str()
                    placeholder="Password"
                    class="input input-bordered input-primary max-w-30 my-4 md:my-0"
                    node_ref=password
                    min-length=1
                    required=move || encrypt.get()
                    disabled=move || !encrypt.get()
                />

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

                    <label class="cursor-pointer label justify-normal w-fit">
                        <span class="label-text pr-2">"HMAC"</span>
                        <input
                            type="checkbox"
                            class="checkbox checkbox-secondary checkbox-sm"
                            disabled=move || !encrypt.get()
                            on:click=move |ev| set_hmac.set(event_target_checked(&ev))
                        />
                    </label>

                    // display in mobile mode, hide in md+
                    <label class="cursor-pointer label justify-normal w-fit flex md:hidden">
                        <span class="label-text pr-2">"Show"</span>
                        <input
                            type="checkbox"
                            class="checkbox checkbox-secondary checkbox-sm"
                            disabled=move || !encrypt.get()
                            on:click=move |_| {
                                set_pw_type.set(pw_type.get_untracked().flip());
                            }
                        />
                    </label>
                </div>

                // hide in mobile mode, display in md+
                <div class="hidden md:block">
                    <label class="cursor-pointer label justify-normal w-fit">
                        <span class="label-text pr-2">"Show"</span>
                        <input
                            type="checkbox"
                            class="checkbox checkbox-secondary checkbox-sm"
                            disabled=move || !encrypt.get()
                            on:click=move |_| {
                                set_pw_type.set(pw_type.get_untracked().flip());
                            }
                        />
                    </label>
                </div>
            </div>

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

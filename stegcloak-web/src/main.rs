use leptos::*;
use stegcloak_web::{App, Panic};

fn main() {
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let (panic, set_panic) = create_signal(None);
    set_hook(set_panic);

    provide_context(panic);

    mount_to_body(move || {
        view! { <App/> }
    })
}

fn set_hook(sig: WriteSignal<Option<Panic>>) {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        sig.set(Some(Panic(info.to_string())));
        hook(info);
    }));
}

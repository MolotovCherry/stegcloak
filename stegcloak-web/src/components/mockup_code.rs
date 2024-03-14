use std::{cell::RefCell, hash::DefaultHasher, rc::Rc};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use leptos::*;
use leptos_meta::Style;

#[derive(Debug, PartialEq, Default)]
pub enum Prefix {
    #[default]
    None,
    // 1
    // 2
    // 3
    // ..
    Numbered,
    // $
    // >
    // >
    // ..
    UnixShell,
    Custom(Signal<HashMap<usize, String>>),
}

#[component]
pub fn MockupCode(
    #[prop(into)] lines: Signal<Vec<String>>,
    #[prop(optional)] prefixes: Prefix,
    #[prop(into, optional)] line_classes: Signal<HashMap<usize, String>>,
    #[prop(optional)] mockup_class: String,
) -> impl IntoView {
    thread_local! {
        static INIT: RefCell<bool> = RefCell::new(false);
    }

    let numbered = || make_iter((1usize..).map(|n| n.to_string()));
    let unix_shell = || make_iter(UnixShellPrefix::default());
    let custom = |map| make_iter(CustomPrefix::new(map));
    let none = || make_iter(std::iter::empty());

    let no_prefixes = prefixes == Prefix::None;

    view! {
        <div class=format!("mockup-code text-content text-wrap max-w-sm {mockup_class}")>
            {move || {
                let prefix = Rc::new(RefCell::new(match prefixes {
                    Prefix::None => none(),
                    Prefix::Numbered => numbered(),
                    Prefix::UnixShell => unix_shell(),
                    Prefix::Custom(c) => {
                        custom(c.get())
                    },
                }));

                view! {
                    <For
                        each=move || lines.get().into_iter().enumerate()
                        key=|(i, line)| {
                            let mut hasher = DefaultHasher::new();
                            line.hash(&mut hasher);
                            let hash = hasher.finish();

                            (*i, hash)
                        }
                        let:line
                    >
                        {
                            let (i, line) = line;
                            let prefix = prefix.clone();

                            view! {
                                <pre
                                    data-prefix=move || prefix.borrow_mut().next()
                                    class=move || {
                                        if no_prefixes {
                                            "whitespace-normal flex break-all".to_owned()
                                        } else {
                                            format!("whitespace-normal flex before:pl-6 pre-wrap break-all {}", line_classes.get().get(&(i + 1)).map(|n| &**n).unwrap_or(""))
                                        }
                                    }
                                >
                                    <code>{line}</code>
                                </pre>
                            }
                        }
                    </For>
                }
            }}
        </div>

        // change the before width on mockup code pre so we can reset it and set our own value to keep wrapping newlines safe
        // this only inits once
        {
            INIT.with_borrow_mut(|init| {
                if !*init {
                    *init = true;

                    view! {
                        <Style>
                            "
                            .mockup-code pre.pre-wrap[data-prefix]:before {
                                width: auto;
                            }
                            "
                        </Style>
                    }
                } else {
                    view! {}.into_view()
                }
            })

        }
    }
}

fn make_iter<I: Iterator<Item = String> + 'static>(i: I) -> Box<dyn Iterator<Item = String>> {
    Box::new(i)
}

#[derive(Debug, Default)]
struct UnixShellPrefix {
    init: bool,
}
impl Iterator for UnixShellPrefix {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.init {
            self.init = true;
            Some("$".to_string())
        } else {
            Some(">".to_string())
        }
    }
}

#[derive(Debug)]
struct CustomPrefix {
    counter: usize,
    map: HashMap<usize, String>,
}

impl CustomPrefix {
    fn new(map: HashMap<usize, String>) -> Self {
        Self { counter: 0, map }
    }
}

impl Iterator for CustomPrefix {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.counter += 1;
        let c = self.counter;

        self.map.get(&c).map(ToOwned::to_owned)
    }
}

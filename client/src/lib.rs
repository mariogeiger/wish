use leptos::prelude::*;

mod api;
mod components;
mod hungarian;
mod i18n;
mod pages;
mod parse;

use i18n::{detect_lang, save_lang, translations};
use wish_shared::Lang;

pub fn input_value(ev: &web_sys::Event) -> String {
    use wasm_bindgen::JsCast;
    ev.target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let path = get_path();
    let query = get_query();

    let (lang, set_lang) = signal(detect_lang());
    provide_context(lang);
    provide_context(set_lang);

    // Persist language changes
    Effect::new(move |_| {
        save_lang(lang.get());
    });

    view! {
        {move || {
            let p = path.clone();
            let q = query.clone();
            match p.as_str() {
                "/wish" => view! { <pages::wish::WishPage key=q /> }.into_any(),
                "/admin" => view! { <pages::admin::AdminPage key=q /> }.into_any(),
                "/offline" => view! { <pages::offline::OfflinePage /> }.into_any(),
                "/history" => view! { <pages::history::HistoryPage /> }.into_any(),
                "/email" => view! { <pages::email::EmailPage /> }.into_any(),
                "/help" => view! { <pages::help::HelpPage /> }.into_any(),
                _ => view! { <pages::home::HomePage /> }.into_any(),
            }
        }}
    }
}

/// Shared nav bar with Home/Help links plus the language switcher. Pages pass
/// which links to show via props.
#[component]
pub fn NavBar(
    #[prop(default = true)] home: bool,
    #[prop(default = true)] help: bool,
    #[prop(default = false)] offline: bool,
) -> impl IntoView {
    let lang = i18n::use_lang();
    view! {
        <nav>
            {move || {
                let t = translations(lang.get());
                let mut parts: Vec<leptos::prelude::AnyView> = Vec::new();
                if home {
                    parts.push(view! { <a href="/">{t.nav_home}</a> }.into_any());
                }
                if help {
                    parts.push(view! { <a href="/help">{t.nav_help}</a> }.into_any());
                }
                if offline {
                    parts.push(view! { <a href="/offline">{t.nav_offline}</a> }.into_any());
                }
                parts
            }}
            <LangSwitcher />
        </nav>
    }
}

#[component]
fn LangSwitcher() -> impl IntoView {
    let lang = i18n::use_lang();
    let set_lang = i18n::use_set_lang();
    let make_on_click = move |target: Lang| move |_: web_sys::MouseEvent| set_lang.set(target);
    view! {
        <span class="lang-switcher">
            <button
                class:active=move || lang.get() == Lang::En
                on:click=make_on_click(Lang::En)
                title="English"
            >"EN"</button>
            <button
                class:active=move || lang.get() == Lang::Fr
                on:click=make_on_click(Lang::Fr)
                title="Français"
            >"FR"</button>
            <button
                class:active=move || lang.get() == Lang::It
                on:click=make_on_click(Lang::It)
                title="Italiano"
            >"IT"</button>
            <button
                class:active=move || lang.get() == Lang::De
                on:click=make_on_click(Lang::De)
                title="Deutsch"
            >"DE"</button>
        </span>
    }
}

fn get_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

fn get_query() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return String::new(),
    };
    let location = window.location();
    // Support both ?key (new) and #key (old app compat)
    let search = location.search().ok().unwrap_or_default();
    let hash = location.hash().ok().unwrap_or_default();
    let from_search = search.trim_start_matches('?');
    let from_hash = hash.trim_start_matches('#');
    if !from_search.is_empty() {
        from_search.to_string()
    } else {
        from_hash.to_string()
    }
}

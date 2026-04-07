use leptos::prelude::*;

mod api;
mod components;
mod hungarian;
mod pages;
mod parse;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let path = get_path();
    let query = get_query();

    view! {
        {move || {
            let p = path.clone();
            let q = query.clone();
            match p.as_str() {
                "/wish" => view! { <pages::wish::WishPage key=q /> }.into_any(),
                "/admin" => view! { <pages::admin::AdminPage key=q /> }.into_any(),
                "/offline" => view! { <pages::offline::OfflinePage /> }.into_any(),
                "/history" => view! { <pages::history::HistoryPage /> }.into_any(),
                "/help" => view! { <pages::help::HelpPage /> }.into_any(),
                _ => view! { <pages::home::HomePage /> }.into_any(),
            }
        }}
    }
}

fn get_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

fn get_query() -> String {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| s.trim_start_matches('?').to_string())
        .unwrap_or_default()
}

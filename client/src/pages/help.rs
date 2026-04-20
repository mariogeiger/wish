use leptos::prelude::*;

use crate::NavBar;
use crate::components::markdown;
use crate::i18n::{translations, use_lang};

#[component]
pub fn HelpPage() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="container">
            <h1>{move || translations(lang.get()).help_heading}</h1>
            <NavBar help=false offline=true />
            <div inner_html=move || markdown::render(translations(lang.get()).help_markdown) />
        </div>
    }
}

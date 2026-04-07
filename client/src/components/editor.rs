use leptos::prelude::*;

use crate::parse;

/// A textarea-based editor with Rust-driven parsing and validation feedback.
#[component]
pub fn Editor(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
) -> impl IntoView {
    let errors = move || {
        let parsed = parse::parse(&text.get());
        parsed.errors.iter().map(|e| {
            format!("Line {}: {}", e.line + 1, e.message)
        }).collect::<Vec<_>>()
    };

    let warnings = move || {
        let parsed = parse::parse(&text.get());
        parsed.warnings.iter().map(|w| {
            format!("Line {}: {}", w.line + 1, w.message)
        }).collect::<Vec<_>>()
    };

    view! {
        <textarea
            class="editor-area"
            prop:value=move || text.get()
            on:input=move |ev| {
                use wasm_bindgen::JsCast;
                let target: web_sys::HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
                set_text.set(target.value());
            }
            spellcheck="false"
        />
        {move || {
            let errs = errors();
            if errs.is_empty() {
                view! { <div /> }.into_any()
            } else {
                view! {
                    <div class="editor-errors">
                        {errs.into_iter().map(|e| view! { <div>{e}</div> }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }
        }}
        {move || {
            let warns = warnings();
            if warns.is_empty() {
                view! { <div /> }.into_any()
            } else {
                view! {
                    <div class="editor-warnings">
                        {warns.into_iter().map(|w| view! { <div>{w}</div> }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }
        }}
    }
}

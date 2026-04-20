use leptos::prelude::*;

use crate::NavBar;
use crate::components::editor::{Editor, highlight};
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::hungarian;
use crate::i18n::{translations, use_lang};

const DEFAULT_TEXT: &str = r#"[slots]
"Monday morning"    0 10
"Monday afternoon"  0 10
"Tuesday morning"   0 10

[participants]
"alice@example.com"   0 1 2
"bob@example.com"     2 0 1
"charlie@example.com" 1 2 0
"#;

#[component]
pub fn OfflinePage() -> impl IntoView {
    let lang = use_lang();
    let (toasts, set_toasts) = signal(Vec::new());
    let (editor_text, set_editor_text) = signal(DEFAULT_TEXT.to_string());
    let (results_text, set_results_text) = signal(String::new());

    let on_compute = move |_| match hungarian::compute_and_format(&editor_text.get()) {
        Ok(text) => set_results_text.set(text),
        Err(errors) => {
            let t = translations(lang.get());
            add_toast(
                &set_toasts,
                t.admin_parse_errors,
                &errors
                    .iter()
                    .map(|e| format!("Line {}: {}", e.line + 1, e.message))
                    .collect::<Vec<_>>()
                    .join("<br/>"),
                ToastKind::Error,
            );
        }
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>{move || translations(lang.get()).offline_heading}</h1>
            <NavBar help=true />

            <p>{move || translations(lang.get()).offline_note}</p>

            <h3>{move || translations(lang.get()).admin_problem_settings}</h3>
            <Editor text=editor_text set_text=set_editor_text />

            <div class="btn-row">
                <button class="btn-success" on:click=on_compute>
                    {move || translations(lang.get()).admin_compute_assignment}
                </button>
            </div>

            <h3>{move || translations(lang.get()).admin_assignment}</h3>
            <pre
                class="editor-area results-area"
                inner_html=move || {
                    let t = results_text.get();
                    if t.is_empty() { String::new() } else { highlight(&t) }
                }
            />
        </div>
    }
}

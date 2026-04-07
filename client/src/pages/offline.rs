use leptos::prelude::*;

use crate::components::editor::Editor;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::hungarian;
use crate::parse;

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
    let (toasts, set_toasts) = signal(Vec::new());
    let (editor_text, set_editor_text) = signal(DEFAULT_TEXT.to_string());
    let (results_text, set_results_text) = signal(String::new());

    let on_compute = move |_| {
        let text = editor_text.get();
        let parsed = parse::parse(&text);

        if !parsed.errors.is_empty() {
            add_toast(
                &set_toasts,
                "Parse errors",
                &parsed
                    .errors
                    .iter()
                    .map(|e| format!("Line {}: {}", e.line + 1, e.message))
                    .collect::<Vec<_>>()
                    .join("<br/>"),
                ToastKind::Error,
            );
            return;
        }

        let slots_data: Vec<(u32, u32)> =
            parsed.slots.iter().map(|s| (s.vmin, s.vmax)).collect();
        let n = parsed.participants.len();

        // Shuffle permutation
        let perm: Vec<usize> = {
            let mut p: Vec<usize> = (0..n).collect();
            for i in (1..p.len()).rev() {
                let j = (js_sys::Math::random() * (i + 1) as f64) as usize;
                p.swap(i, j);
            }
            p
        };

        let wishes: Vec<Vec<i32>> = parsed.participants.iter().map(|p| p.wish.clone()).collect();

        let mut permuted_wishes = vec![Vec::new(); n];
        for (i, &pi) in perm.iter().enumerate() {
            permuted_wishes[pi] = wishes[i].clone();
        }

        let cost = hungarian::build_cost_matrix(&permuted_wishes, &slots_data, n);
        let assignment = hungarian::hungarian(&cost);
        let slot_indices = hungarian::assignment_to_slots(&assignment, &slots_data, n);

        let mut result = vec![0usize; n];
        for i in 0..n {
            result[i] = slot_indices[perm[i]];
        }

        let participants_for_results: Vec<(String, Vec<i32>)> = parsed
            .participants
            .iter()
            .map(|p| (p.mail.clone(), p.wish.clone()))
            .collect();

        let text = parse::format_results(&parsed.slots, &participants_for_results, &result);
        set_results_text.set(text);
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish — Offline"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/help">"Help"</a>
            </nav>

            <p>"This is the offline version. No emails are sent and no data is saved on the server."</p>

            <h3>"Problem Settings"</h3>
            <Editor text=editor_text set_text=set_editor_text />

            <div class="btn-row">
                <button class="btn-success" on:click=on_compute>"Compute Assignment"</button>
            </div>

            <h3>"Assignment"</h3>
            <textarea class="editor-area results-area" readonly
                prop:value=move || results_text.get()
            />
        </div>
    }
}

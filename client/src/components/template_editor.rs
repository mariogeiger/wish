use leptos::prelude::*;
use web_sys::HtmlElement;
use wish_shared::{TemplateSpan, scan_template};

/// Produce highlighted HTML for an email template: known `$name` tokens are
/// wrapped in `<span class="hl-variable">`, everything else is HTML-escaped
/// plain text. Unknown tokens like `$ur` render as literal text.
pub fn highlight_template(text: &str) -> String {
    let mut html = String::with_capacity(text.len());
    scan_template(text, |span| match span {
        TemplateSpan::Text(t) => push_escaped(&mut html, t),
        TemplateSpan::Var { raw, known, .. } => {
            if known {
                html.push_str("<span class=\"hl-variable\">");
                push_escaped(&mut html, raw);
                html.push_str("</span>");
            } else {
                push_escaped(&mut html, raw);
            }
        }
    });
    html
}

fn push_escaped(html: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            _ => html.push(c),
        }
    }
}

#[component]
pub fn TemplateEditor(text: ReadSignal<String>, set_text: WriteSignal<String>) -> impl IntoView {
    let pre_ref = NodeRef::<leptos::html::Pre>::new();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

    let on_scroll = move |_| {
        if let (Some(ta), Some(pre)) = (textarea_ref.get(), pre_ref.get()) {
            let ta_el: &HtmlElement = &ta;
            let pre_el: &HtmlElement = &pre;
            pre_el.set_scroll_top(ta_el.scroll_top());
            pre_el.set_scroll_left(ta_el.scroll_left());
        }
    };

    view! {
        <div class="editor-wrap template-editor-wrap">
            <pre
                class="editor-highlight template-editor-highlight"
                node_ref=pre_ref
                aria-hidden="true"
                inner_html=move || {
                    let mut h = highlight_template(&text.get());
                    h.push('\n');
                    h
                }
            />
            <textarea
                class="editor-area template-editor-area"
                node_ref=textarea_ref
                prop:value=move || text.get()
                on:input=move |ev| {
                    set_text.set(crate::input_value(&ev));
                }
                on:scroll=on_scroll
                spellcheck="false"
            />
        </div>
    }
}

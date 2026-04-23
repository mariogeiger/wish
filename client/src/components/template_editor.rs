use leptos::prelude::*;
use wish_shared::{TemplateSpan, scan_template};

use crate::components::highlighted_textarea::HighlightedTextarea;

/// Produce highlighted HTML for an email template: `$name` tokens whose name
/// appears in `allowed` are wrapped in `<span class="hl-variable">`; everything
/// else is HTML-escaped plain text.
pub fn highlight_template(text: &str, allowed: &[&str]) -> String {
    let mut html = String::with_capacity(text.len());
    scan_template(text, |span| match span {
        TemplateSpan::Text(t) => push_escaped(&mut html, t),
        TemplateSpan::Var { raw, name, .. } => {
            if allowed.contains(&name) {
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
pub fn TemplateEditor(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
    allowed: &'static [&'static str],
) -> impl IntoView {
    view! {
        <HighlightedTextarea
            text
            set_text
            highlight=move |t: String| highlight_template(&t, allowed)
            extra_class="template-editor"
        />
    }
}

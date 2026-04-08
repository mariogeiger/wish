use leptos::prelude::*;
use web_sys::HtmlElement;

use crate::parse;

/// Produce highlighted HTML from the editor text.
pub fn highlight(text: &str) -> String {
    let mut html = String::new();
    for line in text.split('\n') {
        let trimmed = line.trim();

        if trimmed.starts_with('%') || trimmed.starts_with('#') {
            // Comment line
            html.push_str("<span class=\"hl-comment\">");
            push_escaped(&mut html, line);
            html.push_str("</span>");
        } else if trimmed.starts_with('[') {
            // Section header
            html.push_str("<span class=\"hl-section\">");
            push_escaped(&mut html, line);
            html.push_str("</span>");
        } else {
            // Data line: highlight quoted strings, numbers, inline comments
            highlight_data_line(&mut html, line);
        }
        html.push('\n');
    }
    // Remove the trailing newline we added after the last line
    if html.ends_with('\n') {
        html.pop();
    }
    html
}

fn highlight_data_line(html: &mut String, line: &str) {
    let mut chars = line.char_indices().peekable();

    while let Some(&(i, c)) = chars.peek() {
        if c == '"' {
            // Find closing quote
            let mut end = i + 1;
            let mut found = false;
            for (j, ch) in chars.clone().skip(1) {
                end = j;
                if ch == '"' {
                    found = true;
                    break;
                }
            }
            if found {
                let quoted = &line[i..=end];
                html.push_str("<span class=\"hl-string\">");
                push_escaped(html, quoted);
                html.push_str("</span>");
                // Advance past the closing quote
                while let Some(&(idx, _)) = chars.peek() {
                    chars.next();
                    if idx == end {
                        break;
                    }
                }
            } else {
                push_escaped_char(html, c);
                chars.next();
            }
        } else if c == '%' || c == '#' {
            // Inline comment — rest of line
            html.push_str("<span class=\"hl-comment\">");
            push_escaped(html, &line[i..]);
            html.push_str("</span>");
            break;
        } else if c.is_ascii_digit() {
            // Number token
            let start = i;
            chars.next();
            let mut end = start + c.len_utf8();
            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_ascii_digit() {
                    end = j + ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            html.push_str("<span class=\"hl-number\">");
            push_escaped(html, &line[start..end]);
            html.push_str("</span>");
        } else {
            push_escaped_char(html, c);
            chars.next();
        }
    }
}

fn push_escaped(html: &mut String, s: &str) {
    for c in s.chars() {
        push_escaped_char(html, c);
    }
}

fn push_escaped_char(html: &mut String, c: char) {
    match c {
        '&' => html.push_str("&amp;"),
        '<' => html.push_str("&lt;"),
        '>' => html.push_str("&gt;"),
        _ => html.push(c),
    }
}

/// A textarea-based editor with syntax highlighting overlay.
#[component]
pub fn Editor(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
) -> impl IntoView {
    let pre_ref = NodeRef::<leptos::html::Pre>::new();
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();

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

    // Sync scroll from textarea to pre
    let on_scroll = move |_| {
        if let (Some(ta), Some(pre)) = (textarea_ref.get(), pre_ref.get()) {
            let ta_el: &HtmlElement = &ta;
            let pre_el: &HtmlElement = &pre;
            pre_el.set_scroll_top(ta_el.scroll_top());
            pre_el.set_scroll_left(ta_el.scroll_left());
        }
    };

    view! {
        <div class="editor-wrap">
            <pre
                class="editor-highlight"
                node_ref=pre_ref
                aria-hidden="true"
                inner_html=move || {
                    let t = text.get();
                    // Append a trailing newline so the pre always has the same height as the textarea
                    let mut h = highlight(&t);
                    h.push('\n');
                    h
                }
            />
            <textarea
                class="editor-area"
                node_ref=textarea_ref
                prop:value=move || text.get()
                on:input=move |ev| {
                    use wasm_bindgen::JsCast;
                    let target: web_sys::HtmlTextAreaElement = ev.target().unwrap().unchecked_into();
                    set_text.set(target.value());
                }
                on:scroll=on_scroll
                spellcheck="false"
            />
        </div>
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

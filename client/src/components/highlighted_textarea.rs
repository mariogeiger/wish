use leptos::prelude::*;
use web_sys::HtmlElement;

/// A textarea paired with a `<pre>` overlay rendered via the provided
/// `highlight` function, with scroll sync between the two. Shared by the
/// editor and template-editor pages.
///
/// `extra_class` is a CSS-class suffix: when non-empty, each of the three
/// elements gets a second class built as `{extra_class}-wrap|highlight|area`
/// so callers can target their layer's extras without duplicating scaffolding.
#[component]
pub fn HighlightedTextarea<H>(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
    highlight: H,
    #[prop(default = "")] extra_class: &'static str,
) -> impl IntoView
where
    H: Fn(String) -> String + Copy + Send + Sync + 'static,
{
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

    let class = move |base: &str| {
        if extra_class.is_empty() {
            base.to_string()
        } else {
            format!(
                "{base} {extra_class}-{}",
                base.trim_start_matches("editor-")
            )
        }
    };

    view! {
        <div class=class("editor-wrap")>
            <pre
                class=class("editor-highlight")
                node_ref=pre_ref
                aria-hidden="true"
                inner_html=move || {
                    let mut h = highlight(text.get());
                    h.push('\n');
                    h
                }
            />
            <textarea
                class=class("editor-area")
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

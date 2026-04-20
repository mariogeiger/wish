use leptos::prelude::*;
use wish_shared::{DebugEmailRequest, HistoryRequest};

use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};

#[component]
pub fn EmailPage() -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::new());
    let (password, set_password) = signal(String::new());
    let (authed, set_authed) = signal(false);
    let (to, set_to) = signal(String::new());
    let (subject, set_subject) = signal(String::new());
    let (html, set_html) = signal(String::new());
    let (text, set_text) = signal(String::new());
    let (sending, set_sending) = signal(false);

    let do_auth = move || {
        let pw = password.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, serde_json::Value>(
                "/api/debug/auth",
                &HistoryRequest { password: pw },
            )
            .await
            {
                Ok(_) => set_authed.set(true),
                Err(e) => add_toast(&set_toasts, "Error", &e, ToastKind::Error),
            }
        });
    };

    let on_auth_click = move |_: web_sys::MouseEvent| do_auth();
    let on_auth_key = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_auth();
        }
    };

    let on_send = move |_: web_sys::MouseEvent| {
        set_sending.set(true);
        let req = DebugEmailRequest {
            password: password.get(),
            to: to.get(),
            subject: subject.get(),
            html: html.get(),
            text: text.get(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, serde_json::Value>("/api/debug/email", &req).await {
                Ok(_) => add_toast(&set_toasts, "Sent", "Email sent.", ToastKind::Success),
                Err(e) => add_toast(&set_toasts, "Error", &e, ToastKind::Error),
            }
            set_sending.set(false);
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish \u{2014} Email Debug"</h1>
            <nav>
                <a href="/">"Home"</a>
            </nav>

            {move || {
                if !authed.get() {
                    view! {
                        <div class="row" style="max-width: 300px">
                            <div>
                                <label>"Password"</label>
                                <input type="password"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(crate::input_value(&ev))
                                    on:keydown=on_auth_key
                                />
                            </div>
                        </div>
                        <div class="btn-row">
                            <button on:click=on_auth_click>"Enter"</button>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div>
                            <label for="dbg-to">"To"</label>
                            <input id="dbg-to" type="email" placeholder="recipient@example.com"
                                prop:value=move || to.get()
                                on:input=move |ev| set_to.set(crate::input_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="dbg-subject">"Subject"</label>
                            <input id="dbg-subject" type="text"
                                prop:value=move || subject.get()
                                on:input=move |ev| set_subject.set(crate::input_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="dbg-html">"HTML body"</label>
                            <textarea id="dbg-html" rows="12"
                                prop:value=move || html.get()
                                on:input=move |ev| set_html.set(crate::input_value(&ev))
                            />
                        </div>
                        <div>
                            <label for="dbg-text">"Text body"</label>
                            <textarea id="dbg-text" rows="6"
                                prop:value=move || text.get()
                                on:input=move |ev| set_text.set(crate::input_value(&ev))
                            />
                        </div>
                        <div class="btn-row">
                            <button on:click=on_send disabled=move || sending.get()>
                                {move || if sending.get() { "Sending..." } else { "Send" }}
                            </button>
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

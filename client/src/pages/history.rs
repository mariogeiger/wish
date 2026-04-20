use leptos::prelude::*;
use wish_shared::{HistoryEntry, HistoryRequest};

use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};

#[component]
pub fn HistoryPage() -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::new());
    let (password, set_password) = signal(String::new());
    let (entries, set_entries) = signal(Vec::<HistoryEntry>::new());
    let (loaded, set_loaded) = signal(false);

    let do_submit = move || {
        let pw = password.get();
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, Vec<HistoryEntry>>(
                "/api/history",
                &HistoryRequest { password: pw },
            )
            .await
            {
                Ok(data) => {
                    set_entries.set(data);
                    set_loaded.set(true);
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                }
            }
        });
    };

    let on_click = move |_: web_sys::MouseEvent| {
        do_submit();
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_submit();
        }
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish \u{2014} History"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/help">"Help"</a>
            </nav>

            {move || {
                if !loaded.get() {
                    view! {
                        <div class="row" style="max-width: 300px">
                            <div>
                                <label>"Password"</label>
                                <input type="password"
                                    prop:value=move || password.get()
                                    on:input=move |ev| {
                                        set_password.set(crate::input_value(&ev));
                                    }
                                    on:keydown=on_keydown
                                />
                            </div>
                        </div>
                        <button on:click=on_click>"View History"</button>
                    }
                    .into_any()
                } else {
                    view! {
                        <ul>
                            {move || {
                                entries
                                    .get()
                                    .iter()
                                    .map(|e| {
                                        let date = format_timestamp(e.creation_time);
                                        let admin_url = format!("/admin?{}", e.id);
                                        view! {
                                            <li>
                                                {date}" "
                                                <a href=admin_url>
                                                    <strong>{e.name.clone()}</strong>
                                                </a>
                                                " (admin: "{e.admin_mail.clone()}", "
                                                {e.num_participants}" participants)"
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </ul>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

fn format_timestamp(ms: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    date.to_date_string().as_string().unwrap_or_default()
}

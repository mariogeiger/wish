use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wish_shared::*;

use crate::api;
use crate::components::editor::{Editor, highlight};
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::hungarian;
use crate::parse;

#[component]
pub fn AdminPage(key: String) -> impl IntoView {
    let (toasts, set_toasts) = signal(Vec::new());
    let (editor_text, set_editor_text) = signal(String::new());
    let (results_text, set_results_text) = signal(String::new());
    let (event_name, set_event_name) = signal(String::new());
    let (saving, set_saving) = signal(false);
    let (ws_banner, set_ws_banner) = signal(None::<String>);

    // Fetch admin data
    let key_load = key.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match api::get::<AdminData>(&format!("/api/events/{key_load}")).await {
            Ok(data) => {
                set_event_name.set(data.name.clone());
                let participants: Vec<(String, Vec<i32>, ParticipantStatus, Option<String>)> = data
                    .participants
                    .iter()
                    .map(|p| (p.mail.clone(), p.wish.clone(), p.status, Some(p.id.clone())))
                    .collect();
                set_editor_text.set(parse::to_editor_text(&data.slots, &participants));
            }
            Err(e) => {
                add_toast(&set_toasts, "Error", &format!("Failed to load: {e}"), ToastKind::Error);
            }
        }
    });

    // WebSocket for real-time notifications
    let key_ws = key.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let location = web_sys::window().unwrap().location();
        let protocol = if location.protocol().unwrap_or_default() == "https:" {
            "wss:"
        } else {
            "ws:"
        };
        let host = location.host().unwrap_or_default();
        let ws_url = format!("{protocol}//{host}/api/events/{key_ws}/ws");

        if let Ok(ws) = web_sys::WebSocket::new(&ws_url) {
            let key_refresh = key_ws.clone();
            let on_message = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::MessageEvent)>::new(
                move |ev: web_sys::MessageEvent| {
                    if let Some(text) = ev.data().as_string() {
                        if let Ok(msg) = serde_json::from_str::<WsMsg>(&text) {
                            match msg {
                                WsMsg::NewWish { mail } => {
                                    set_ws_banner.set(Some(format!(
                                        "{mail} modified their wish. Reload to see changes."
                                    )));
                                }
                                WsMsg::MailProgress { sent, total, errors } => {
                                    let kind = if errors.is_empty() {
                                        if sent == total { ToastKind::Success } else { ToastKind::Info }
                                    } else {
                                        ToastKind::Error
                                    };
                                    let mut msg = format!("{sent}/{total} mails sent");
                                    for e in &errors {
                                        msg.push_str(&format!("<br/>Error: {e}"));
                                    }
                                    add_toast(&set_toasts, "Mail status", &msg, kind);

                                    if sent == total {
                                        let key = key_refresh.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            if let Ok(data) = api::get::<AdminData>(&format!("/api/events/{key}")).await {
                                                let participants: Vec<(String, Vec<i32>, ParticipantStatus, Option<String>)> = data
                                                    .participants
                                                    .iter()
                                                    .map(|p| (p.mail.clone(), p.wish.clone(), p.status, Some(p.id.clone())))
                                                    .collect();
                                                set_editor_text.set(parse::to_editor_text(&data.slots, &participants));
                                                set_event_name.set(data.name);
                                            }
                                        });
                                    }
                                }
                                WsMsg::Feedback { title, html, msg_type } => {
                                    let kind = match msg_type.as_str() {
                                        "success" => ToastKind::Success,
                                        "error" => ToastKind::Error,
                                        _ => ToastKind::Info,
                                    };
                                    add_toast(&set_toasts, &title, &html, kind);
                                }
                            }
                        }
                    }
                },
            );
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
        }
    });

    let key_save = key.clone();
    let on_save = move |send_mails: bool| {
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

        set_saving.set(true);
        let key = key_save.clone();
        let req = SetDataRequest {
            slots: parsed.slots,
            participants: parsed
                .participants
                .into_iter()
                .map(|p| ParticipantInput {
                    mail: p.mail,
                    wish: p.wish,
                })
                .collect(),
            send_mails,
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::put::<_, AdminData>(&format!("/api/events/{key}"), &req).await {
                Ok(data) => {
                    // Refresh editor with fresh server state
                    let participants: Vec<(String, Vec<i32>, ParticipantStatus, Option<String>)> =
                        data.participants
                            .iter()
                            .map(|p| (p.mail.clone(), p.wish.clone(), p.status, Some(p.id.clone())))
                            .collect();
                    set_editor_text.set(parse::to_editor_text(&data.slots, &participants));
                    set_event_name.set(data.name);

                    if send_mails {
                        add_toast(&set_toasts, "Saved & sending", "Data saved. Sending mails...", ToastKind::Info);
                    } else {
                        add_toast(&set_toasts, "Saved", "Data saved.", ToastKind::Success);
                    }
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                }
            }
            set_saving.set(false);
        });
    };

    let on_save_only = {
        let on_save = on_save.clone();
        move |_| on_save(false)
    };
    let on_save_and_mail = {
        let on_save = on_save.clone();
        move |_| on_save(true)
    };

    let key_remind = key.clone();
    let on_remind = move |_| {
        let key = key_remind.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, SendMailsResponse>(
                &format!("/api/events/{key}/remind"),
                &serde_json::json!({}),
            )
            .await
            {
                Ok(resp) => {
                    add_toast(
                        &set_toasts,
                        "Reminders",
                        &format!("Sending {} reminders...", resp.total),
                        ToastKind::Info,
                    );
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                }
            }
        });
    };

    let on_compute = move |_| {
        match hungarian::compute_and_format(&editor_text.get()) {
            Ok(text) => set_results_text.set(text),
            Err(_) => {
                add_toast(&set_toasts, "Parse errors", "Fix errors before computing.", ToastKind::Error);
            }
        }
    };

    let key_results = key.clone();
    let on_send_results = move |_| {
        let text = results_text.get();
        // Parse results to extract mail -> slot mapping
        let mut entries = Vec::new();
        let mut in_results = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed == "[results]" {
                in_results = true;
                continue;
            }
            if !in_results || trimmed.starts_with('%') || trimmed.is_empty() {
                continue;
            }
            // Extract two quoted strings: "mail" "slot name"
            if let Some((mail, rest)) = parse::parse_quoted_string(trimmed) {
                if let Some((slot, _)) = parse::parse_quoted_string(rest) {
                    entries.push(ResultEntry {
                        mail: mail.to_string(),
                        slot: slot.to_string(),
                    });
                }
            }
        }

        if entries.is_empty() {
            add_toast(&set_toasts, "Error", "No results to send. Compute assignment first.", ToastKind::Error);
            return;
        }

        let key = key_results.clone();
        let req = SendResultsRequest { results: entries };
        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, SendMailsResponse>(
                &format!("/api/events/{key}/results"),
                &req,
            )
            .await
            {
                Ok(resp) => {
                    add_toast(
                        &set_toasts,
                        "Results",
                        &format!("Sending {} result emails...", resp.total),
                        ToastKind::Info,
                    );
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                }
            }
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/help">"Help"</a>
            </nav>

            <h2>{move || event_name.get()}</h2>

            {move || {
                ws_banner.get().map(|msg| {
                    view! {
                        <div class="editor-warnings" style="cursor:pointer"
                            on:click=move |_| {
                                let _ = web_sys::window().unwrap().location().reload();
                            }
                        >
                            {msg}" (click to reload)"
                        </div>
                    }
                })
            }}

            <h3>"Problem Settings"</h3>
            <Editor text=editor_text set_text=set_editor_text />

            <div class="btn-row">
                <button on:click=on_save_only disabled=move || saving.get()>"Save"</button>
                <button on:click=on_save_and_mail disabled=move || saving.get()>"Save & Send Mails"</button>
                <button class="btn-secondary" on:click=on_remind>"Send Reminder"</button>
                <button class="btn-success" on:click=on_compute>"Compute Assignment"</button>
            </div>

            <h3>"Assignment"</h3>
            <pre
                class="editor-area results-area"
                inner_html=move || {
                    let t = results_text.get();
                    if t.is_empty() { String::new() } else { highlight(&t) }
                }
            />

            <div class="btn-row">
                <button class="btn-success" on:click=on_send_results>"Send Results"</button>
            </div>
        </div>
    }
}

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wish_shared::*;

use crate::NavBar;
use crate::api;
use crate::components::editor::{Editor, highlight};
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::components::template_editor::TemplateEditor;
use crate::hungarian;
use crate::i18n::{translations, use_lang};
use crate::parse;

#[component]
pub fn AdminPage(key: String) -> impl IntoView {
    let lang = use_lang();
    let (toasts, set_toasts) = signal(Vec::new());
    let (editor_text, set_editor_text) = signal(String::new());
    let (results_text, set_results_text) = signal(String::new());
    let (event_name, set_event_name) = signal(String::new());
    let (saving, set_saving) = signal(false);
    let (tpl_invite, set_tpl_invite) = signal(String::new());
    let (tpl_update, set_tpl_update) = signal(String::new());
    let (tpl_reminder, set_tpl_reminder) = signal(String::new());
    let (tpl_results, set_tpl_results) = signal(String::new());
    let (mail_seen, set_mail_seen) = signal(0usize);

    // Snapshot of last server-known state. `dirty` = current editor/template
    // signals differ from the snapshot (→ user has unsaved edits).
    let (clean_editor_text, set_clean_editor_text) = signal(String::new());
    let (clean_tpl_invite, set_clean_tpl_invite) = signal(String::new());
    let (clean_tpl_update, set_clean_tpl_update) = signal(String::new());
    let (clean_tpl_reminder, set_clean_tpl_reminder) = signal(String::new());
    let (clean_tpl_results, set_clean_tpl_results) = signal(String::new());

    let is_dirty = move || {
        editor_text.get_untracked() != clean_editor_text.get_untracked()
            || tpl_invite.get_untracked() != clean_tpl_invite.get_untracked()
            || tpl_update.get_untracked() != clean_tpl_update.get_untracked()
            || tpl_reminder.get_untracked() != clean_tpl_reminder.get_untracked()
            || tpl_results.get_untracked() != clean_tpl_results.get_untracked()
    };

    // Server has state we haven't loaded. Some(body) → banner visible, saves blocked.
    let (stale_banner, set_stale_banner) = signal(None::<String>);

    // Write server data into both live and clean signals; clear stale state.
    let apply_admin_data = move |data: AdminData| {
        let editor = parse::to_editor_text(&data.slots, &data.participants);
        set_event_name.set(data.name);
        set_editor_text.set(editor.clone());
        set_clean_editor_text.set(editor);
        set_clean_tpl_invite.set(data.templates.invite.clone());
        set_tpl_invite.set(data.templates.invite);
        set_clean_tpl_update.set(data.templates.update.clone());
        set_tpl_update.set(data.templates.update);
        set_clean_tpl_reminder.set(data.templates.reminder.clone());
        set_tpl_reminder.set(data.templates.reminder);
        set_clean_tpl_results.set(data.templates.results.clone());
        set_tpl_results.set(data.templates.results);
        set_stale_banner.set(None);
    };

    // Initial fetch
    let key_load = key.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match api::get::<AdminData>(&format!("/api/events/{key_load}")).await {
            Ok(data) => apply_admin_data(data),
            Err(e) => {
                let t = translations(lang.get());
                add_toast(
                    &set_toasts,
                    t.error,
                    &format!("{}{e}", t.failed_to_load),
                    ToastKind::Error,
                );
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
                    if let Some(text) = ev.data().as_string()
                        && let Ok(msg) = serde_json::from_str::<WsMsg>(&text)
                    {
                        match msg {
                            WsMsg::NewWish { mail } => {
                                let t = translations(lang.get());
                                if is_dirty() {
                                    // Block save: show banner + error toast, do not clobber edits.
                                    set_stale_banner
                                        .set(Some(format!("{mail}{}", t.admin_ws_banner_suffix)));
                                    add_toast(
                                        &set_toasts,
                                        t.admin_stale_title,
                                        &format!(
                                            "{}{}",
                                            escape_html(&mail),
                                            t.admin_ws_banner_suffix
                                        ),
                                        ToastKind::Error,
                                    );
                                } else {
                                    // Not editing: silently refetch and show info toast.
                                    add_toast(
                                        &set_toasts,
                                        t.admin_data_updated_title,
                                        &escape_html(&mail),
                                        ToastKind::Info,
                                    );
                                    let key = key_refresh.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Ok(data) =
                                            api::get::<AdminData>(&format!("/api/events/{key}"))
                                                .await
                                        {
                                            apply_admin_data(data);
                                        }
                                    });
                                }
                            }
                            WsMsg::MailProgress {
                                sent,
                                total,
                                mail,
                                error,
                            } => {
                                let t = translations(lang.get());
                                let safe_mail = escape_html(&mail);
                                let (kind, msg) = match &error {
                                    None => (
                                        ToastKind::Success,
                                        format!("{sent}/{total} — {safe_mail}"),
                                    ),
                                    Some(e) => (
                                        ToastKind::Error,
                                        format!(
                                            "{sent}/{total} — {safe_mail}<br/>{}{}",
                                            t.admin_error_prefix,
                                            escape_html(e)
                                        ),
                                    ),
                                };
                                add_toast(&set_toasts, t.admin_mail_status, &msg, kind);

                                set_mail_seen.update(|c| *c += 1);
                                if mail_seen.get_untracked() >= total {
                                    set_mail_seen.set(0);
                                    if is_dirty() {
                                        set_stale_banner
                                            .set(Some(t.admin_stale_mail_send.to_string()));
                                        add_toast(
                                            &set_toasts,
                                            t.admin_stale_title,
                                            t.admin_stale_mail_send,
                                            ToastKind::Error,
                                        );
                                    } else {
                                        let key = key_refresh.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            if let Ok(data) =
                                                api::get::<AdminData>(&format!("/api/events/{key}"))
                                                    .await
                                            {
                                                apply_admin_data(data);
                                            }
                                        });
                                    }
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
        let t = translations(lang.get());

        if !parsed.errors.is_empty() {
            add_toast(
                &set_toasts,
                t.admin_parse_errors,
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

        let tpl_invite_val = tpl_invite.get();
        let tpl_update_val = tpl_update.get();
        let tpl_reminder_val = tpl_reminder.get();
        let tpl_results_val = tpl_results.get();
        let required_checks: [(&str, &str, &[&str]); 4] = [
            (t.admin_invite_heading, &tpl_invite_val, INVITE_REQUIRED),
            (t.admin_update_heading, &tpl_update_val, UPDATE_REQUIRED),
            (
                t.admin_reminder_heading,
                &tpl_reminder_val,
                REMINDER_REQUIRED,
            ),
            (t.admin_results_heading, &tpl_results_val, RESULTS_REQUIRED),
        ];
        let missing: Vec<String> = required_checks
            .iter()
            .filter_map(|(heading, text, required)| {
                let missing = missing_required_vars(text, required);
                (!missing.is_empty()).then(|| {
                    let vars = missing
                        .iter()
                        .map(|v| format!("${v}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<b>{heading}</b>: {vars}")
                })
            })
            .collect();
        if !missing.is_empty() {
            add_toast(
                &set_toasts,
                t.admin_err_required_title,
                &missing.join("<br/>"),
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
            templates: EmailTemplates {
                invite: tpl_invite_val,
                update: tpl_update_val,
                reminder: tpl_reminder_val,
                results: tpl_results_val,
            },
        };

        wasm_bindgen_futures::spawn_local(async move {
            let t = translations(lang.get());
            match api::put::<_, AdminData>(&format!("/api/events/{key}"), &req).await {
                Ok(data) => {
                    apply_admin_data(data);
                    if send_mails {
                        add_toast(
                            &set_toasts,
                            t.admin_saved_and_sending,
                            t.admin_data_saved_sending,
                            ToastKind::Info,
                        );
                    } else {
                        add_toast(&set_toasts, t.saved, t.admin_data_saved, ToastKind::Success);
                    }
                }
                Err(e) => {
                    add_toast(&set_toasts, t.error, &e, ToastKind::Error);
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
        let t = translations(lang.get());
        let confirmed = web_sys::window()
            .and_then(|w| w.confirm_with_message(t.admin_confirm_remind).ok())
            .unwrap_or(false);
        if !confirmed {
            return;
        }
        let key = key_remind.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let t = translations(lang.get());
            match api::post::<_, SendMailsResponse>(
                &format!("/api/events/{key}/remind"),
                &serde_json::json!({}),
            )
            .await
            {
                Ok(resp) => {
                    add_toast(
                        &set_toasts,
                        t.admin_reminders_title,
                        &format!("{}{}", resp.total, t.admin_reminders_sending),
                        ToastKind::Info,
                    );
                }
                Err(e) => {
                    add_toast(&set_toasts, t.error, &e, ToastKind::Error);
                }
            }
        });
    };

    let on_compute = move |_| match hungarian::compute_and_format(&editor_text.get()) {
        Ok(text) => set_results_text.set(text),
        Err(_) => {
            let t = translations(lang.get());
            add_toast(
                &set_toasts,
                t.admin_parse_errors,
                t.admin_fix_errors,
                ToastKind::Error,
            );
        }
    };

    let key_results = key.clone();
    let on_send_results = move |_| {
        let text = results_text.get();
        let t = translations(lang.get());
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
            if let Some((mail, rest)) = parse::parse_quoted_string(trimmed)
                && let Some((slot, _)) = parse::parse_quoted_string(rest)
            {
                entries.push(ResultEntry {
                    mail: mail.to_string(),
                    slot: slot.to_string(),
                });
            }
        }

        if entries.is_empty() {
            add_toast(&set_toasts, t.error, t.admin_no_results, ToastKind::Error);
            return;
        }

        let key = key_results.clone();
        let req = SendResultsRequest { results: entries };
        wasm_bindgen_futures::spawn_local(async move {
            let t = translations(lang.get());
            match api::post::<_, SendMailsResponse>(&format!("/api/events/{key}/results"), &req)
                .await
            {
                Ok(resp) => {
                    add_toast(
                        &set_toasts,
                        t.admin_results_title,
                        &format!("{}{}", resp.total, t.admin_results_sending),
                        ToastKind::Info,
                    );
                }
                Err(e) => {
                    add_toast(&set_toasts, t.error, &e, ToastKind::Error);
                }
            }
        });
    };

    let save_disabled = move || saving.get() || stale_banner.get().is_some();

    view! {
        <ToastContainer toasts=toasts set_toasts=set_toasts />
        <div class="container container-wide">
            <h1>"Wish"</h1>
            <NavBar />

            <h2>{move || event_name.get()}</h2>

            {move || {
                let t = translations(lang.get());
                stale_banner.get().map(|body| {
                    view! {
                        <div class="editor-warnings" style="cursor:pointer"
                            on:click=move |_| {
                                let _ = web_sys::window().unwrap().location().reload();
                            }
                        >
                            {body}{t.admin_click_to_reload}
                        </div>
                    }
                })
            }}

            <h3>{move || translations(lang.get()).admin_problem_settings}</h3>
            <Editor text=editor_text set_text=set_editor_text />

            <details class="tpl-section">
                <summary>{move || translations(lang.get()).admin_email_templates}</summary>
                <p class="muted">{move || translations(lang.get()).admin_templates_hint}</p>

                <h4>{move || translations(lang.get()).admin_invite_heading}</h4>
                <p class="muted">
                    {move || translations(lang.get()).admin_available_prefix}
                    "$event_name, $admin_mail, $url"
                </p>
                <TemplateEditor text=tpl_invite set_text=set_tpl_invite allowed=wish_shared::INVITE_VARS />

                <h4>{move || translations(lang.get()).admin_update_heading}</h4>
                <p class="muted">
                    {move || translations(lang.get()).admin_available_prefix}
                    "$event_name, $admin_mail, $url"
                </p>
                <TemplateEditor text=tpl_update set_text=set_tpl_update allowed=wish_shared::UPDATE_VARS />

                <h4>{move || translations(lang.get()).admin_reminder_heading}</h4>
                <p class="muted">
                    {move || translations(lang.get()).admin_available_prefix}
                    "$event_name, $admin_mail, $url"
                </p>
                <TemplateEditor text=tpl_reminder set_text=set_tpl_reminder allowed=wish_shared::REMINDER_VARS />

                <h4>{move || translations(lang.get()).admin_results_heading}</h4>
                <p class="muted">
                    {move || translations(lang.get()).admin_available_prefix}
                    "$event_name, $slot"
                </p>
                <TemplateEditor text=tpl_results set_text=set_tpl_results allowed=wish_shared::RESULTS_VARS />
            </details>

            <div class="btn-row">
                <button on:click=on_save_only disabled=save_disabled>
                    {move || translations(lang.get()).admin_save}
                </button>
                <button on:click=on_save_and_mail disabled=save_disabled>
                    {move || translations(lang.get()).admin_save_and_send}
                </button>
                <button class="btn-secondary" on:click=on_remind>
                    {move || translations(lang.get()).admin_send_reminder}
                </button>
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

            <div class="btn-row">
                <button class="btn-success" on:click=on_send_results>
                    {move || translations(lang.get()).admin_send_results}
                </button>
            </div>
        </div>
    }
}

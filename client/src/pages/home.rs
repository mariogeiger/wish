use leptos::prelude::*;
use wish_shared::{CreateEventRequest, CreateEventResponse, Slot};

use crate::NavBar;
use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::i18n::{translations, use_lang};

fn split_emails(s: &str) -> Vec<String> {
    s.split([',', '\n', ';'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[component]
pub fn HomePage() -> impl IntoView {
    let lang = use_lang();
    let (name, set_name) = signal(String::new());
    let (admin_mail, set_admin_mail) = signal(String::new());
    let (mails_text, set_mails_text) = signal(String::new());
    let (num_slots, set_num_slots) = signal(2u32);
    let (slots, set_slots) = signal(vec![
        ("".to_string(), 0u32, 10u32),
        ("".to_string(), 0u32, 10u32),
    ]);
    let (submitting, set_submitting) = signal(false);
    let (toasts, set_toasts) = signal(Vec::new());

    // Keep slots vec in sync with num_slots
    Effect::new(move |_| {
        let n = num_slots.get() as usize;
        set_slots.update(|s| {
            while s.len() < n {
                s.push(("".to_string(), 0, 10));
            }
            s.truncate(n);
        });
    });

    let on_submit = move |_| {
        set_submitting.set(true);
        let t = translations(lang.get());
        let name_val = name.get();
        let admin_mail_val = admin_mail.get();
        let mails_val = mails_text.get();
        let slots_val = slots.get();

        let mails = split_emails(&mails_val);

        if name_val.is_empty() {
            add_toast(
                &set_toasts,
                t.error,
                t.home_err_activity_required,
                ToastKind::Error,
            );
            set_submitting.set(false);
            return;
        }
        if admin_mail_val.is_empty() {
            add_toast(
                &set_toasts,
                t.error,
                t.home_err_admin_required,
                ToastKind::Error,
            );
            set_submitting.set(false);
            return;
        }
        if mails.is_empty() {
            add_toast(
                &set_toasts,
                t.error,
                t.home_err_participants_required,
                ToastKind::Error,
            );
            set_submitting.set(false);
            return;
        }

        let slot_prefix = t.home_slot_placeholder.to_string();
        let api_slots: Vec<Slot> = slots_val
            .iter()
            .enumerate()
            .map(|(i, (name, vmin, vmax))| Slot {
                name: if name.is_empty() {
                    format!("{slot_prefix}{}", i + 1)
                } else {
                    name.clone()
                },
                vmin: *vmin,
                vmax: *vmax,
            })
            .collect();

        let req = CreateEventRequest {
            name: name_val,
            admin_mail: admin_mail_val,
            mails,
            slots: api_slots,
            lang: lang.get(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, CreateEventResponse>("/api/events", &req).await {
                Ok(resp) => {
                    // Redirect to admin page
                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/admin?{}", resp.event_id));
                    }
                }
                Err(e) => {
                    let t = translations(lang.get());
                    add_toast(&set_toasts, t.error, &e, ToastKind::Error);
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish"</h1>
            <NavBar home=false help=true offline=true />

            <p><em>{move || translations(lang.get()).home_tagline}</em></p>
            <p>{move || translations(lang.get()).home_description}</p>
            <p>
                {move || translations(lang.get()).home_first_time_prefix}
                <a href="/help">{move || translations(lang.get()).home_first_time_link}</a>
                {move || translations(lang.get()).home_first_time_suffix}
            </p>
            <p>
                {move || translations(lang.get()).home_offline_prefix}
                <a href="/offline">{move || translations(lang.get()).home_offline_link}</a>
                {move || translations(lang.get()).home_offline_suffix}
            </p>

            <div class="row">
                <div>
                    <label for="name">{move || translations(lang.get()).home_activity_name}</label>
                    <input id="name" type="text"
                        prop:value=move || name.get()
                        on:input=move |ev| {
                            set_name.set(crate::input_value(&ev));
                        }
                    />
                </div>
                <div style="max-width: 160px">
                    <label for="nslots">{move || translations(lang.get()).home_num_slots}</label>
                    <input id="nslots" type="number" min="1" max="100" step="1"
                        prop:value=move || num_slots.get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = crate::input_value(&ev).parse::<u32>()
                                && (1..=100).contains(&v) {
                                    set_num_slots.set(v);
                                }
                        }
                    />
                </div>
            </div>

            <div class="slot-header">
                <span>{move || translations(lang.get()).home_slot_name}</span>
                <span>{move || translations(lang.get()).home_min}</span>
                <span>{move || translations(lang.get()).home_max}</span>
            </div>
            {move || {
                let slot_prefix = translations(lang.get()).home_slot_placeholder.to_string();
                slots.get().iter().enumerate().map(|(i, (name, vmin, vmax))| {
                    let name = name.clone();
                    let vmin = *vmin;
                    let vmax = *vmax;
                    view! {
                        <div class="slot-row">
                            <input type="text" placeholder=format!("{slot_prefix}{}", i + 1)
                                prop:value=name
                                on:input=move |ev| {
                                    let val = crate::input_value(&ev);
                                    set_slots.update(|s| if i < s.len() { s[i].0 = val; });
                                }
                            />
                            <input type="number" min="0" max="100" step="1"
                                prop:value=vmin.to_string()
                                on:input=move |ev| {
                                    if let Ok(v) = crate::input_value(&ev).parse::<u32>() {
                                        set_slots.update(|s| if i < s.len() { s[i].1 = v; });
                                    }
                                }
                            />
                            <input type="number" min="0" max="100" step="1"
                                prop:value=vmax.to_string()
                                on:input=move |ev| {
                                    if let Ok(v) = crate::input_value(&ev).parse::<u32>() {
                                        set_slots.update(|s| if i < s.len() { s[i].2 = v; });
                                    }
                                }
                            />
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}

            <div>
                <label for="admin_mail">{move || translations(lang.get()).home_admin_email}</label>
                <input id="admin_mail" type="email" placeholder="your@email.com"
                    prop:value=move || admin_mail.get()
                    on:input=move |ev| {
                        set_admin_mail.set(crate::input_value(&ev));
                    }
                />
            </div>

            <div>
                <label for="mails">{move || translations(lang.get()).home_participant_emails}</label>
                <textarea id="mails"
                    placeholder=move || translations(lang.get()).home_participant_emails_placeholder
                    prop:value=move || mails_text.get()
                    on:input=move |ev| {
                        set_mails_text.set(crate::input_value(&ev));
                    }
                />
                <div class="muted">
                    {move || {
                        let count = split_emails(&mails_text.get()).len();
                        let suffix = translations(lang.get()).home_participant_count_suffix;
                        format!("{count}{suffix}")
                    }}
                </div>
            </div>

            <p class="muted">{move || translations(lang.get()).home_customize_emails_note}</p>

            <div class="btn-row">
                <button
                    on:click=on_submit
                    disabled=move || submitting.get()
                >
                    {move || {
                        let t = translations(lang.get());
                        if submitting.get() { t.home_creating } else { t.home_create }
                    }}
                </button>
            </div>
        </div>
    }
}

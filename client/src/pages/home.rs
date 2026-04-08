use leptos::prelude::*;
use wish_shared::{CreateEventRequest, CreateEventResponse, Slot};

use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};

fn split_emails(s: &str) -> Vec<String> {
    s.split([',', '\n', ';'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[component]
pub fn HomePage() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (admin_mail, set_admin_mail) = signal(String::new());
    let (mails_text, set_mails_text) = signal(String::new());
    let (message, set_message) = signal(String::new());
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
        let name_val = name.get();
        let admin_mail_val = admin_mail.get();
        let mails_val = mails_text.get();
        let message_val = message.get();
        let slots_val = slots.get();

        let mails = split_emails(&mails_val);

        if name_val.is_empty() {
            add_toast(&set_toasts, "Error", "Activity name is required", ToastKind::Error);
            set_submitting.set(false);
            return;
        }
        if admin_mail_val.is_empty() {
            add_toast(&set_toasts, "Error", "Admin email is required", ToastKind::Error);
            set_submitting.set(false);
            return;
        }
        if mails.is_empty() {
            add_toast(&set_toasts, "Error", "At least one participant email is required", ToastKind::Error);
            set_submitting.set(false);
            return;
        }

        let api_slots: Vec<Slot> = slots_val
            .iter()
            .enumerate()
            .map(|(i, (name, vmin, vmax))| Slot {
                name: if name.is_empty() {
                    format!("Slot {}", i + 1)
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
            message: message_val,
        };

        wasm_bindgen_futures::spawn_local(async move {
            match api::post::<_, CreateEventResponse>("/api/events", &req).await {
                Ok(resp) => {
                    // Redirect to admin page
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&format!("/admin?{}", resp.event_id));
                    }
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish"</h1>
            <p><em>"Distributes people in various slots maximizing the global satisfaction, taking into account quotas for each slot."</em></p>
            <p>"Organize the groups for various activities according to the desires of your friends, prepare the schedule of an oral exam taking into account the wishes of the students, plan who does what in the organization of a party, ..."</p>
            <p>"If you are using Wish for the first time, take a look at "<a href="/help">"the help page"</a>"."</p>
            <p>"If you don't need to contact the participants by email you can use the "<a href="/offline">"offline version"</a>"."</p>

            <div class="row">
                <div>
                    <label for="name">"Activity name"</label>
                    <input id="name" type="text"
                        prop:value=move || name.get()
                        on:input=move |ev| {
                            set_name.set(crate::input_value(&ev));
                        }
                    />
                </div>
                <div style="max-width: 160px">
                    <label for="nslots">"Number of slots"</label>
                    <input id="nslots" type="number" min="1" max="100" step="1"
                        prop:value=move || num_slots.get().to_string()
                        on:input=move |ev| {
                            if let Ok(v) = crate::input_value(&ev).parse::<u32>() {
                                if v >= 1 && v <= 100 {
                                    set_num_slots.set(v);
                                }
                            }
                        }
                    />
                </div>
            </div>

            <div class="slot-header">
                <span>"Slot name"</span>
                <span>"Min"</span>
                <span>"Max"</span>
            </div>
            {move || {
                slots.get().iter().enumerate().map(|(i, (name, vmin, vmax))| {
                    let name = name.clone();
                    let vmin = *vmin;
                    let vmax = *vmax;
                    view! {
                        <div class="slot-row">
                            <input type="text" placeholder=format!("Slot {}", i + 1)
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
                <label for="admin_mail">"Admin email"</label>
                <input id="admin_mail" type="email" placeholder="your@email.com"
                    prop:value=move || admin_mail.get()
                    on:input=move |ev| {
                        set_admin_mail.set(crate::input_value(&ev));
                    }
                />
            </div>

            <div>
                <label for="mails">"Participant emails"</label>
                <textarea id="mails" placeholder="first@mail, second@mail, ..."
                    prop:value=move || mails_text.get()
                    on:input=move |ev| {
                        set_mails_text.set(crate::input_value(&ev));
                    }
                />
                <div class="muted">
                    {move || {
                        let count = split_emails(&mails_text.get()).len();
                        format!("{count} participant(s)")
                    }}
                </div>
            </div>

            <div>
                <label for="message">"Message (added to invitation email)"</label>
                <textarea id="message"
                    prop:value=move || message.get()
                    on:input=move |ev| {
                        set_message.set(crate::input_value(&ev));
                    }
                />
            </div>

            <div class="btn-row">
                <button
                    on:click=on_submit
                    disabled=move || submitting.get()
                >
                    {move || if submitting.get() { "Creating..." } else { "Create" }}
                </button>
            </div>
        </div>
    }
}

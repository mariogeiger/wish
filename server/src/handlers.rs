use actix_web::{HttpResponse, web};
use std::future::Future;
use wish_shared::*;

use crate::db::Db;
use crate::email::ResendClient;
use crate::{AppState, gen_id};

// ── Create event ───────────────────────────────────────────────────

const MAX_SLOTS: usize = 100;
const MAX_PARTICIPANTS: usize = 1000;

fn get_sorted_participants(db: &Db, event: &Event) -> Vec<Participant> {
    let mut participants: Vec<Participant> = db
        .participants
        .iter()
        .filter(|p| event.participants.contains(&p.id))
        .cloned()
        .collect();
    participants.sort_by(|a, b| a.mail.cmp(&b.mail));
    participants
}

// ── Shared email-sending loop ─────────────────────────────────────

struct OutgoingEmail {
    to: String,
    subject: String,
    html: String,
    text: String,
}

/// Send a batch of emails, broadcasting progress over the WebSocket channel.
///
/// `total` is the denominator shown in progress messages (may exceed
/// `emails.len()` when the caller sends extra emails after this returns).
///
/// `on_result` is called after each send attempt with the email index and
/// whether it succeeded; callers use it to update participant status in the DB.
async fn send_emails<F, Fut>(
    resend: ResendClient,
    broadcast_tx: tokio::sync::broadcast::Sender<String>,
    emails: Vec<OutgoingEmail>,
    total: usize,
    on_result: F,
) -> (usize, Vec<String>)
where
    F: Fn(usize, bool) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut sent = 0usize;
    let mut errors = Vec::new();

    for (i, email) in emails.iter().enumerate() {
        match resend
            .send(&email.to, &email.subject, &email.html, &email.text)
            .await
        {
            Ok(()) => {
                sent += 1;
                on_result(i, true).await;
            }
            Err(e) => {
                errors.push(format!("{}: {e}", email.to));
                on_result(i, false).await;
            }
        }

        let msg = WsMsg::MailProgress {
            sent,
            total,
            errors: errors.clone(),
        };
        let _ = broadcast_tx.send(serde_json::to_string(&msg).unwrap());
    }

    (sent, errors)
}

pub async fn create_event(
    state: web::Data<AppState>,
    body: web::Json<CreateEventRequest>,
) -> HttpResponse {
    if body.slots.len() > MAX_SLOTS {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "too many slots (max 100)"}));
    }
    if body.mails.len() > MAX_PARTICIPANTS {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "too many participants (max 1000)"}));
    }

    let base_url = state.config.base_url.clone();

    let event_id = state.with_db_save(|db| {
        let event_id = gen_id();
        let mut participant_ids = Vec::new();

        let default_wish = vec![0i32; body.slots.len()];
        for mail in &body.mails {
            let pid = gen_id();
            db.participants.push(Participant {
                id: pid.clone(),
                mail: mail.clone(),
                wish: default_wish.clone(),
                event: event_id.clone(),
                status: ParticipantStatus::New,
            });
            participant_ids.push(pid);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        db.events.push(Event {
            id: event_id.clone(),
            name: body.name.clone(),
            admin_mail: body.admin_mail.clone(),
            slots: body.slots.clone(),
            url: base_url.clone(),
            message: body.message.clone(),
            participants: participant_ids,
            creation_time: now,
        });

        event_id
    });

    // Send admin confirmation email in background
    let resend = state.resend.clone();
    let admin_mail = body.admin_mail.clone();
    let event_name = body.name.clone();
    let eid = event_id.clone();
    tokio::spawn(async move {
        let admin_url = format!("{base_url}/admin?{eid}");
        let safe_name = escape_html(&event_name);
        let html = format!(
            "<p>Hi,</p>\
             <p>An event has been created with your email address.<br />\
             <strong>If you are not concerned, please do not click on the following url.</strong><br />\
             <a href=\"{admin_url}\">Click here</a> to administrate the activity.</p>\
             <p>Have a nice day,<br />The Wish team</p>"
        );
        let text = format!(
            "Hi,\n\
             An event has been created with your email address.\n\
             If you are not concerned, please do not click on the following url.\n\
             To administrate the activity, go to: {admin_url}\n\n\
             Have a nice day,\nThe Wish team"
        );
        if let Err(e) = resend
            .send(&admin_mail, &format!("Wish: {safe_name}"), &html, &text)
            .await
        {
            log::error!("Failed to send admin email: {e}");
        }
    });

    HttpResponse::Ok().json(CreateEventResponse { event_id })
}

// ── Admin data ─────────────────────────────────────────────────────

pub async fn get_admin_data(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let event_id = path.into_inner();
    state.with_db(|db| {
        let event = match db.events.iter().find(|e| e.id == event_id) {
            Some(e) => e,
            None => return HttpResponse::NotFound().finish(),
        };

        HttpResponse::Ok().json(AdminData {
            name: event.name.clone(),
            slots: event.slots.clone(),
            participants: get_sorted_participants(db, event),
        })
    })
}

// ── Update event data (+ optionally send mails) ───────────────────

pub async fn set_admin_data(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SetDataRequest>,
) -> HttpResponse {
    let event_id = path.into_inner();

    if body.slots.len() > MAX_SLOTS {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "too many slots (max 100)"}));
    }
    if body.participants.len() > MAX_PARTICIPANTS {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "too many participants (max 1000)"}));
    }

    // Validate
    let mut sum_vmin = 0u32;
    let mut sum_vmax = 0u32;
    for slot in &body.slots {
        if slot.vmin > slot.vmax {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "vmin > vmax"}));
        }
        sum_vmin += slot.vmin;
        sum_vmax += slot.vmax;
    }
    let n = body.participants.len() as u32;
    if n > sum_vmax || n < sum_vmin {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "participant count not in range [sum_vmin, sum_vmax]"}),
        );
    }
    for p in &body.participants {
        if p.wish.len() != body.slots.len() {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "wish length != slots length"}));
        }
    }

    // Save data
    let save_result: Option<AdminData> = state.with_db_save(|db| {
        let event = match db.events.iter_mut().find(|e| e.id == event_id) {
            Some(e) => e,
            None => return None,
        };

        // Detect if slots changed (names differ)
        let slots_changed = event.slots.len() != body.slots.len()
            || event
                .slots
                .iter()
                .zip(body.slots.iter())
                .any(|(a, b)| a.name != b.name);

        event.slots = body.slots.clone();

        // Reconcile participants
        let mut new_participant_ids = Vec::new();
        for pi in &body.participants {
            let existing = db
                .participants
                .iter_mut()
                .find(|p| p.mail == pi.mail && p.event == event_id);

            if let Some(p) = existing {
                p.wish = pi.wish.clone();
                if slots_changed && p.status.as_i32() > 10 {
                    p.status = ParticipantStatus::UpdatePending;
                }
                new_participant_ids.push(p.id.clone());
            } else {
                let pid = gen_id();
                db.participants.push(Participant {
                    id: pid.clone(),
                    mail: pi.mail.clone(),
                    wish: pi.wish.clone(),
                    event: event_id.clone(),
                    status: ParticipantStatus::New,
                });
                new_participant_ids.push(pid);
            }
        }

        // Clean up removed participants
        let old_ids = std::mem::replace(&mut event.participants, new_participant_ids.clone());
        let _ = event;
        db.participants
            .retain(|p| p.event != event_id || new_participant_ids.contains(&p.id));
        let removed = old_ids.len()
            - old_ids
                .iter()
                .filter(|id| new_participant_ids.contains(id))
                .count();
        if removed > 0 {
            log::info!("Removed {removed} stale participants from event {event_id}");
        }

        let event = db.events.iter().find(|e| e.id == event_id).unwrap();
        Some(AdminData {
            name: event.name.clone(),
            slots: event.slots.clone(),
            participants: get_sorted_participants(db, event),
        })
    });

    let admin_data = match save_result {
        Some(d) => d,
        None => return HttpResponse::NotFound().finish(),
    };

    // Optionally send mails
    if body.send_mails {
        let info = match get_event_info(&state, &event_id) {
            Some(i) => i,
            None => return HttpResponse::Ok().json(admin_data),
        };
        let resend = state.resend.clone();

        let to_send: Vec<(String, String, ParticipantStatus)> = state.with_db(|db| {
            let event = match db.events.iter().find(|e| e.id == event_id) {
                Some(e) => e,
                None => return Vec::new(),
            };
            db.participants
                .iter()
                .filter(|p| event.participants.contains(&p.id) && p.status.as_i32() <= 10)
                .map(|p| (p.id.clone(), p.mail.clone(), p.status))
                .collect()
        });

        let broadcast_tx = state.get_broadcast(&event_id);
        let total = to_send.len();
        let safe_name = escape_html(&info.event_name);
        let safe_admin = escape_html(&info.admin_mail);
        let safe_message = escape_html(&info.event_message);

        let pids: Vec<String> = to_send.iter().map(|(pid, _, _)| pid.clone()).collect();
        let emails: Vec<OutgoingEmail> = to_send
            .iter()
            .map(|(pid, mail, status)| {
                let wish_url = format!("{}/wish?{pid}", info.base_url);
                let (html, text) = if status.as_i32() <= 0 {
                    (
                        format!(
                            "<p>Hi,</p>\
                             <p>You have been invited by {safe_admin} to give your wishes about the event: <strong>{safe_name}</strong></p><br />\
                             <pre>{safe_message}</pre>\
                             <p><a href=\"{wish_url}\">Click here</a> to set your wishes.</p>\
                             <p>Have a nice day,<br />The Wish team</p>"
                        ),
                        format!(
                            "Hi,\n\
                             You have been invited by {} to give your wishes about the event: {}\n\
                             {}\n\n\
                             {wish_url}\n\n\
                             Have a nice day,\nThe Wish team",
                            info.admin_mail, info.event_name, info.event_message
                        ),
                    )
                } else {
                    (
                        format!(
                            "<p>Hi,</p>\
                             <p>The administrator ({safe_admin}) of the event <strong>{safe_name}</strong> has modified the slots.</p>\
                             <p>Please look at <a href=\"{wish_url}\">your wish</a>.</p>\
                             <p>Have a nice day,<br />The Wish team</p>"
                        ),
                        format!(
                            "Hi,\n\
                             The administrator ({}) of the event {} has modified the slots.\n\
                             Please look at your wish: {wish_url}\n\n\
                             Have a nice day,\nThe Wish team",
                            info.admin_mail, info.event_name
                        ),
                    )
                };
                OutgoingEmail {
                    to: mail.clone(),
                    subject: format!("Wish: {}", info.event_name),
                    html,
                    text,
                }
            })
            .collect();

        let state_ref = state.clone();
        tokio::spawn(async move {
            send_emails(resend, broadcast_tx, emails, total, |i, ok| {
                let state_ref = state_ref.clone();
                let pid = pids[i].clone();
                async move {
                    let new_status = if ok {
                        ParticipantStatus::Mailed
                    } else {
                        ParticipantStatus::MailError
                    };
                    state_ref.with_db_save(|db| {
                        if let Some(p) = db.participants.iter_mut().find(|p| p.id == pid) {
                            p.status = new_status;
                        }
                    });
                }
            })
            .await;
        });
    }

    HttpResponse::Ok().json(admin_data)
}

// ── Helper: load event info or return 404 ──────────────────────────

struct EventInfo {
    base_url: String,
    admin_mail: String,
    event_name: String,
    event_message: String,
}

fn get_event_info(state: &AppState, event_id: &str) -> Option<EventInfo> {
    state.with_db(|db| {
        db.events
            .iter()
            .find(|e| e.id == event_id)
            .map(|event| EventInfo {
                base_url: event.url.clone(),
                admin_mail: event.admin_mail.clone(),
                event_name: event.name.clone(),
                event_message: event.message.clone(),
            })
    })
}

// ── Send reminders ─────────────────────────────────────────────────

pub async fn send_reminders(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let event_id = path.into_inner();
    let resend = state.resend.clone();

    let info = match get_event_info(&state, &event_id) {
        Some(i) => i,
        None => return HttpResponse::NotFound().finish(),
    };

    let to_remind: Vec<(String, String)> = state.with_db(|db| {
        let event = match db.events.iter().find(|e| e.id == event_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        db.participants
            .iter()
            .filter(|p| event.participants.contains(&p.id) && p.status.needs_reminder())
            .map(|p| (p.id.clone(), p.mail.clone()))
            .collect()
    });

    let total = to_remind.len();
    let broadcast_tx = state.get_broadcast(&event_id);
    let safe_name = escape_html(&info.event_name);

    let pids: Vec<String> = to_remind.iter().map(|(pid, _)| pid.clone()).collect();
    let emails: Vec<OutgoingEmail> = to_remind
        .iter()
        .map(|(pid, mail)| {
            let wish_url = format!("{}/wish?{pid}", info.base_url);
            OutgoingEmail {
                to: mail.clone(),
                subject: format!("Wish: {}", info.event_name),
                html: format!(
                    "<p>Hi,</p>\
                     <p>Don't forget to fill <a href=\"{wish_url}\">your wish</a> for the event <strong>{safe_name}</strong>.</p>\
                     <p>Have a nice day,<br />The Wish team</p>"
                ),
                text: format!(
                    "Hi,\n\
                     Don't forget to fill your wish for the event {}.\n\
                     {wish_url}\n\n\
                     Have a nice day,\nThe Wish team",
                    info.event_name
                ),
            }
        })
        .collect();

    let state_ref = state.clone();
    tokio::spawn(async move {
        send_emails(resend, broadcast_tx, emails, total, |i, ok| {
            let state_ref = state_ref.clone();
            let pid = pids[i].clone();
            async move {
                if !ok {
                    state_ref.with_db_save(|db| {
                        if let Some(p) = db.participants.iter_mut().find(|p| p.id == pid) {
                            p.status = ParticipantStatus::MailError;
                        }
                    });
                }
            }
        })
        .await;
    });

    HttpResponse::Ok().json(SendMailsResponse { total })
}

// ── Send results ───────────────────────────────────────────────────

pub async fn send_results(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SendResultsRequest>,
) -> HttpResponse {
    let event_id = path.into_inner();
    let resend = state.resend.clone();

    let info = match get_event_info(&state, &event_id) {
        Some(i) => i,
        None => return HttpResponse::NotFound().finish(),
    };

    let results = body.results.clone();
    let total = results.len() + 1; // +1 for admin summary
    let broadcast_tx = state.get_broadcast(&event_id);
    let safe_name = escape_html(&info.event_name);

    // Build per-participant result emails + admin summary as last entry
    let mut emails: Vec<OutgoingEmail> = results
        .iter()
        .map(|entry| {
            let safe_slot = escape_html(&entry.slot);
            OutgoingEmail {
                to: entry.mail.clone(),
                subject: format!("Wish: {}", info.event_name),
                html: format!(
                    "<p>Hi,</p>\
                     <p>You have been put in the slot <strong>{safe_slot}</strong> for the event <strong>{safe_name}</strong>.</p>\
                     <p>Have a nice day,<br />The Wish team</p>"
                ),
                text: format!(
                    "Hi,\n\
                     You have been put in the slot {} for the event {}.\n\n\
                     Have a nice day,\nThe Wish team",
                    entry.slot, info.event_name
                ),
            }
        })
        .collect();

    // Admin summary email
    let rows: String = results
        .iter()
        .map(|r| {
            format!(
                "<tr><td>{}</td><td>{}</td></tr>",
                escape_html(&r.mail),
                escape_html(&r.slot)
            )
        })
        .collect();
    let text_rows: String = results
        .iter()
        .map(|r| format!("{}  {}\n", r.mail, r.slot))
        .collect();
    emails.push(OutgoingEmail {
        to: info.admin_mail.clone(),
        subject: format!("Wish: {}", info.event_name),
        html: format!(
            "<p>Hi,</p>\
             <p>The following information have been sent to the participants of the event <strong>{safe_name}</strong>.</p>\
             <table><tr><th>mail</th><th>slot</th></tr>{rows}</table>\
             <p>Have a nice day,<br />The Wish team</p>"
        ),
        text: format!(
            "Hi,\n\
             The following information have been sent to the participants of the event {}.\n\n\
             {text_rows}\n\
             Have a nice day,\nThe Wish team",
            info.event_name
        ),
    });

    tokio::spawn(async move {
        send_emails(resend, broadcast_tx, emails, total, |_i, _ok| async {}).await;
    });

    HttpResponse::Ok().json(SendMailsResponse { total })
}

// ── Wish endpoints ─────────────────────────────────────────────────

pub async fn get_wish(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    let pid = path.into_inner();
    state.with_db_save(|db| {
        let participant = match db.participants.iter_mut().find(|p| p.id == pid) {
            Some(p) => p,
            None => return HttpResponse::NotFound().finish(),
        };
        let event = match db.events.iter().find(|e| e.id == participant.event) {
            Some(e) => e,
            None => return HttpResponse::NotFound().finish(),
        };

        // Fix wish length if slots changed
        if event.slots.len() != participant.wish.len() {
            participant.wish = vec![0; event.slots.len()];
        }

        // Mark as visited
        if participant.status.as_i32() < 30 {
            participant.status = ParticipantStatus::Visited;
        }

        HttpResponse::Ok().json(WishData {
            name: event.name.clone(),
            mail: participant.mail.clone(),
            slots: event.slots.clone(),
            wish: participant.wish.clone(),
        })
    })
}

pub async fn set_wish(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<SetWishRequest>,
) -> HttpResponse {
    let pid = path.into_inner();

    if !wish_shared::is_fair_wish(&body.wish) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "unfair wish"}));
    }

    let event_id = state.with_db_save(|db| {
        let participant = match db.participants.iter_mut().find(|p| p.id == pid) {
            Some(p) => p,
            None => return None,
        };
        let event = match db.events.iter().find(|e| e.id == participant.event) {
            Some(e) => e,
            None => return None,
        };

        if event.slots.len() != body.wish.len() {
            return None;
        }

        participant.wish = body.wish.clone();
        participant.status = ParticipantStatus::Modified;
        Some((participant.event.clone(), participant.mail.clone()))
    });

    match event_id {
        Some((eid, mail)) => {
            // Notify admin via WebSocket
            let msg = WsMsg::NewWish { mail };
            state.broadcast(&eid, &serde_json::to_string(&msg).unwrap());
            HttpResponse::Ok().json(serde_json::json!({"ok": true}))
        }
        None => HttpResponse::NotFound().finish(),
    }
}

// ── History ────────────────────────────────────────────────────────

pub async fn get_history(
    state: web::Data<AppState>,
    body: web::Json<HistoryRequest>,
) -> HttpResponse {
    if body.password != state.config.history_password {
        return HttpResponse::Unauthorized().finish();
    }

    state.with_db(|db| {
        let mut entries: Vec<HistoryEntry> = db
            .events
            .iter()
            .map(|e| HistoryEntry {
                id: e.id.clone(),
                name: e.name.clone(),
                admin_mail: e.admin_mail.clone(),
                num_participants: e.participants.len(),
                message: e.message.clone(),
                creation_time: e.creation_time,
            })
            .collect();
        entries.sort_by(|a, b| b.creation_time.cmp(&a.creation_time));
        HttpResponse::Ok().json(entries)
    })
}

// ── Health ──────────────────────────────────────────────────────────

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

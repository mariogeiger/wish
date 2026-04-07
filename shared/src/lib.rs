use serde::{Deserialize, Serialize};

// ── Core data model ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub name: String,
    pub vmin: u32,
    pub vmax: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantStatus {
    New,
    UpdatePending,
    MailError,
    Mailed,
    Visited,
    Done,
    Modified,
}

impl Default for ParticipantStatus {
    fn default() -> Self {
        Self::New
    }
}

impl ParticipantStatus {
    pub fn as_i32(self) -> i32 {
        match self {
            Self::New => 0,
            Self::UpdatePending => 10,
            Self::MailError => -10,
            Self::Mailed => 20,
            Self::Visited => 30,
            Self::Done => 35,
            Self::Modified => 40,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::New => "mail not sent",
            Self::UpdatePending => "update mail not sent",
            Self::MailError => "mail error",
            Self::Mailed => "mail sent, no activity",
            Self::Visited => "visited wish page",
            Self::Done => "completed wishes",
            Self::Modified => "modified wishes",
        }
    }

    /// Whether this participant still needs to fill their wishes.
    pub fn needs_reminder(self) -> bool {
        self.as_i32() < 35
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub mail: String,
    pub wish: Vec<i32>,
    pub event: String,
    #[serde(default)]
    pub status: ParticipantStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub admin_mail: String,
    pub slots: Vec<Slot>,
    pub url: String,
    pub message: String,
    pub participants: Vec<String>,
    pub creation_time: i64,
}

// ── API request/response types ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub name: String,
    pub admin_mail: String,
    pub mails: Vec<String>,
    pub slots: Vec<Slot>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventResponse {
    pub event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminData {
    pub name: String,
    pub slots: Vec<Slot>,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDataRequest {
    pub slots: Vec<Slot>,
    pub participants: Vec<ParticipantInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInput {
    pub mail: String,
    pub wish: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishData {
    pub name: String,
    pub mail: String,
    pub slots: Vec<Slot>,
    pub wish: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetWishRequest {
    pub wish: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResultsRequest {
    pub results: Vec<ResultEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultEntry {
    pub mail: String,
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub name: String,
    pub admin_mail: String,
    pub num_participants: usize,
    pub message: String,
    pub creation_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailsResponse {
    pub total: usize,
}

// ── Utilities ──────────────────────────────────────────────────────

/// Escape HTML special characters to prevent XSS in email templates.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

// ── Validation ─────────────────────────────────────────────────────

/// Check whether a wish vector satisfies the fairness rule.
/// Rule: when sorted ascending, sorted[i] <= i for all i.
pub fn is_fair_wish(wish: &[i32]) -> bool {
    let mut sorted: Vec<i32> = wish.to_vec();
    sorted.sort();
    sorted.iter().enumerate().all(|(i, &v)| v <= i as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ParticipantStatus ──────────────────────────────────────────

    #[test]
    fn status_as_i32_round_trips() {
        let cases = [
            (ParticipantStatus::New, 0),
            (ParticipantStatus::UpdatePending, 10),
            (ParticipantStatus::MailError, -10),
            (ParticipantStatus::Mailed, 20),
            (ParticipantStatus::Visited, 30),
            (ParticipantStatus::Done, 35),
            (ParticipantStatus::Modified, 40),
        ];
        for (status, expected) in cases {
            assert_eq!(status.as_i32(), expected, "{status:?}");
        }
    }

    #[test]
    fn status_labels_not_empty() {
        let all = [
            ParticipantStatus::New,
            ParticipantStatus::UpdatePending,
            ParticipantStatus::MailError,
            ParticipantStatus::Mailed,
            ParticipantStatus::Visited,
            ParticipantStatus::Done,
            ParticipantStatus::Modified,
        ];
        for s in all {
            assert!(!s.label().is_empty(), "{s:?} has empty label");
        }
    }

    #[test]
    fn needs_reminder_before_done() {
        assert!(ParticipantStatus::New.needs_reminder());
        assert!(ParticipantStatus::Mailed.needs_reminder());
        assert!(ParticipantStatus::Visited.needs_reminder());
        assert!(!ParticipantStatus::Done.needs_reminder());
        assert!(!ParticipantStatus::Modified.needs_reminder());
    }

    #[test]
    fn default_status_is_new() {
        assert_eq!(ParticipantStatus::default(), ParticipantStatus::New);
    }

    // ── is_fair_wish ───────────────────────────────────────────────

    #[test]
    fn fair_wish_all_zeros() {
        assert!(is_fair_wish(&[0, 0, 0, 0]));
    }

    #[test]
    fn fair_wish_strict_ordering() {
        // 0,1,2,3 — perfectly ordered, fair
        assert!(is_fair_wish(&[0, 1, 2, 3]));
    }

    #[test]
    fn fair_wish_reversed() {
        // 3,2,1,0 — sorted is 0,1,2,3 — fair
        assert!(is_fair_wish(&[3, 2, 1, 0]));
    }

    #[test]
    fn fair_wish_some_ties() {
        // 0,0,1,2 — sorted: 0,0,1,2 — all sorted[i] <= i — fair
        assert!(is_fair_wish(&[0, 0, 1, 2]));
    }

    #[test]
    fn unfair_wish_too_many_high() {
        // 3,3,3,0 — sorted: 0,3,3,3 — sorted[1]=3 > 1 — unfair
        assert!(!is_fair_wish(&[3, 3, 3, 0]));
    }

    #[test]
    fn unfair_wish_single_high() {
        // 2,0,0 — sorted: 0,0,2 — sorted[2]=2 <= 2 — fair actually
        assert!(is_fair_wish(&[2, 0, 0]));
        // 2,2,0 — sorted: 0,2,2 — sorted[1]=2 > 1 — unfair
        assert!(!is_fair_wish(&[2, 2, 0]));
    }

    #[test]
    fn fair_wish_empty() {
        assert!(is_fair_wish(&[]));
    }

    #[test]
    fn fair_wish_single() {
        assert!(is_fair_wish(&[0]));
        assert!(!is_fair_wish(&[1]));
    }
}

// ── WebSocket messages ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMsg {
    NewWish { mail: String },
    MailProgress { sent: usize, total: usize, errors: Vec<String> },
    Feedback { title: String, html: String, msg_type: String },
}

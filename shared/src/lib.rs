use serde::{Deserialize, Serialize};

// ── Language ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Fr,
    It,
    De,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::It => "it",
            Self::De => "de",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::En),
            "fr" => Some(Self::Fr),
            "it" => Some(Self::It),
            "de" => Some(Self::De),
            _ => None,
        }
    }

    /// Best-effort match from a browser-style tag like "fr-CA" or "en-US".
    pub fn from_browser_tag(tag: &str) -> Option<Self> {
        let prefix = tag.split(['-', '_']).next().unwrap_or(tag).to_lowercase();
        Self::from_code(&prefix)
    }
}

// ── Core data model ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub name: String,
    pub vmin: u32,
    pub vmax: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParticipantStatus {
    #[default]
    New,
    UpdatePending,
    MailError,
    Mailed,
    Visited,
    Done,
    Modified,
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
        match self {
            Self::New | Self::UpdatePending | Self::MailError | Self::Mailed | Self::Visited => {
                true
            }
            Self::Done | Self::Modified => false,
        }
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
    pub participants: Vec<String>,
    pub creation_time: i64,
    #[serde(default)]
    pub templates: EmailTemplates,
}

// ── Email templates ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplates {
    pub invite: String,
    pub update: String,
    pub reminder: String,
    pub results: String,
}

impl Default for EmailTemplates {
    fn default() -> Self {
        Self::for_lang(Lang::En)
    }
}

impl EmailTemplates {
    pub fn for_lang(lang: Lang) -> Self {
        let (invite, update, reminder, results) = match lang {
            Lang::En => (INVITE_EN, UPDATE_EN, REMINDER_EN, RESULTS_EN),
            Lang::Fr => (INVITE_FR, UPDATE_FR, REMINDER_FR, RESULTS_FR),
            Lang::It => (INVITE_IT, UPDATE_IT, REMINDER_IT, RESULTS_IT),
            Lang::De => (INVITE_DE, UPDATE_DE, REMINDER_DE, RESULTS_DE),
        };
        Self {
            invite: invite.to_string(),
            update: update.to_string(),
            reminder: reminder.to_string(),
            results: results.to_string(),
        }
    }
}

/// Variable names recognized by the template renderer. Any `$word` token that
/// matches one of these is substituted at send time; anything else renders
/// literally. For per-template availability (used by the editor highlighter),
/// see `INVITE_VARS`, `UPDATE_VARS`, `REMINDER_VARS`, `RESULTS_VARS`.
pub const TEMPLATE_VARIABLES: &[&str] = &["event_name", "admin_mail", "url", "slot"];

pub const INVITE_VARS: &[&str] = &["event_name", "admin_mail", "url"];
pub const UPDATE_VARS: &[&str] = &["event_name", "admin_mail", "url"];
pub const REMINDER_VARS: &[&str] = &["event_name", "admin_mail", "url"];
pub const RESULTS_VARS: &[&str] = &["event_name", "slot"];

/// Variables a template must contain — save is blocked if any are missing.
pub const INVITE_REQUIRED: &[&str] = &["url"];
pub const UPDATE_REQUIRED: &[&str] = &["url"];
pub const REMINDER_REQUIRED: &[&str] = &["url"];
pub const RESULTS_REQUIRED: &[&str] = &["slot"];

/// Return the subset of `required` var names not present anywhere in `template`.
pub fn missing_required_vars<'a>(template: &str, required: &'a [&'a str]) -> Vec<&'a str> {
    required
        .iter()
        .filter(|&&v| !template_contains_var(template, v))
        .copied()
        .collect()
}

fn template_contains_var(template: &str, var: &str) -> bool {
    let mut found = false;
    scan_template(template, |span| {
        if let TemplateSpan::Var { name, .. } = span
            && name == var
        {
            found = true;
        }
    });
    found
}

// English

const INVITE_EN: &str = "Hi,\n\
     \n\
     You have been invited by $admin_mail to give your wishes about the event: $event_name\n\
     \n\
     To set your wishes, go to: $url\n\
     \n\
     Have a nice day,\n\
     The Wish team";

const UPDATE_EN: &str = "Hi,\n\
     \n\
     The administrator ($admin_mail) of the event $event_name has modified the slots.\n\
     \n\
     Please look at your wish: $url\n\
     \n\
     Have a nice day,\n\
     The Wish team";

const REMINDER_EN: &str = "Hi,\n\
     \n\
     Don't forget to fill your wish for the event $event_name.\n\
     \n\
     Go to: $url\n\
     \n\
     Have a nice day,\n\
     The Wish team";

const RESULTS_EN: &str = "Hi,\n\
     \n\
     You have been put in the slot $slot for the event $event_name.\n\
     \n\
     Have a nice day,\n\
     The Wish team";

// French

const INVITE_FR: &str = "Bonjour,\n\
     \n\
     Vous avez été invité·e par $admin_mail à donner vos souhaits pour l'événement : $event_name\n\
     \n\
     Pour renseigner vos souhaits, rendez-vous sur : $url\n\
     \n\
     Bonne journée,\n\
     L'équipe Wish";

const UPDATE_FR: &str = "Bonjour,\n\
     \n\
     L'administrateur ($admin_mail) de l'événement $event_name a modifié les créneaux.\n\
     \n\
     Merci de revoir vos souhaits : $url\n\
     \n\
     Bonne journée,\n\
     L'équipe Wish";

const REMINDER_FR: &str = "Bonjour,\n\
     \n\
     N'oubliez pas de renseigner vos souhaits pour l'événement $event_name.\n\
     \n\
     Rendez-vous sur : $url\n\
     \n\
     Bonne journée,\n\
     L'équipe Wish";

const RESULTS_FR: &str = "Bonjour,\n\
     \n\
     Vous avez été placé·e dans le créneau $slot pour l'événement $event_name.\n\
     \n\
     Bonne journée,\n\
     L'équipe Wish";

// Italian

const INVITE_IT: &str = "Ciao,\n\
     \n\
     Sei stato/a invitato/a da $admin_mail a esprimere le tue preferenze per l'evento: $event_name\n\
     \n\
     Per indicare le tue preferenze, vai su: $url\n\
     \n\
     Buona giornata,\n\
     Il team Wish";

const UPDATE_IT: &str = "Ciao,\n\
     \n\
     L'amministratore ($admin_mail) dell'evento $event_name ha modificato le fasce orarie.\n\
     \n\
     Ti preghiamo di rivedere le tue preferenze: $url\n\
     \n\
     Buona giornata,\n\
     Il team Wish";

const REMINDER_IT: &str = "Ciao,\n\
     \n\
     Non dimenticare di indicare le tue preferenze per l'evento $event_name.\n\
     \n\
     Vai su: $url\n\
     \n\
     Buona giornata,\n\
     Il team Wish";

const RESULTS_IT: &str = "Ciao,\n\
     \n\
     Sei stato/a assegnato/a alla fascia $slot per l'evento $event_name.\n\
     \n\
     Buona giornata,\n\
     Il team Wish";

// German

const INVITE_DE: &str = "Hallo,\n\
     \n\
     Du wurdest von $admin_mail eingeladen, deine Wünsche für die Veranstaltung anzugeben: $event_name\n\
     \n\
     Um deine Wünsche einzutragen, gehe zu: $url\n\
     \n\
     Einen schönen Tag,\n\
     Das Wish-Team";

const UPDATE_DE: &str = "Hallo,\n\
     \n\
     Der Administrator ($admin_mail) der Veranstaltung $event_name hat die Zeitfenster geändert.\n\
     \n\
     Bitte überprüfe deine Wünsche: $url\n\
     \n\
     Einen schönen Tag,\n\
     Das Wish-Team";

const REMINDER_DE: &str = "Hallo,\n\
     \n\
     Vergiss nicht, deine Wünsche für die Veranstaltung $event_name anzugeben.\n\
     \n\
     Gehe zu: $url\n\
     \n\
     Einen schönen Tag,\n\
     Das Wish-Team";

const RESULTS_DE: &str = "Hallo,\n\
     \n\
     Du wurdest dem Zeitfenster $slot für die Veranstaltung $event_name zugeteilt.\n\
     \n\
     Einen schönen Tag,\n\
     Das Wish-Team";

/// Scan `$name` tokens in `template`. `emit` receives each span as either
/// plain text or a variable reference (name, whether known). Used by both the
/// renderer and the editor highlighter.
pub fn scan_template<F: FnMut(TemplateSpan<'_>)>(template: &str, mut emit: F) {
    let mut iter = template.char_indices().peekable();
    let mut plain_start = 0usize;

    while let Some(&(i, c)) = iter.peek() {
        if c == '$' {
            let name_start = i + 1;
            let mut name_end = name_start;
            let mut probe = iter.clone();
            probe.next();
            if let Some(&(_, ch)) = probe.peek()
                && (ch.is_ascii_alphabetic() || ch == '_')
            {
                probe.next();
                name_end += ch.len_utf8();
                while let Some(&(j, ch2)) = probe.peek() {
                    if ch2.is_ascii_alphanumeric() || ch2 == '_' {
                        name_end = j + ch2.len_utf8();
                        probe.next();
                    } else {
                        break;
                    }
                }
                if plain_start < i {
                    emit(TemplateSpan::Text(&template[plain_start..i]));
                }
                let name = &template[name_start..name_end];
                let known = TEMPLATE_VARIABLES.contains(&name);
                emit(TemplateSpan::Var {
                    raw: &template[i..name_end],
                    name,
                    known,
                });
                iter = probe;
                plain_start = name_end;
                continue;
            }
        }
        iter.next();
    }
    if plain_start < template.len() {
        emit(TemplateSpan::Text(&template[plain_start..]));
    }
}

pub enum TemplateSpan<'a> {
    Text(&'a str),
    Var {
        raw: &'a str,
        name: &'a str,
        known: bool,
    },
}

/// Substitute `$name` tokens in `template` using `vars`. Unknown tokens render
/// literally (including the `$`). A token is `$` followed by one or more ASCII
/// letters, digits, or underscores (the first char must be a letter or `_`).
pub fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    scan_template(template, |span| match span {
        TemplateSpan::Text(t) => out.push_str(t),
        TemplateSpan::Var { raw, name, .. } => {
            if let Some((_, v)) = vars.iter().find(|(k, _)| *k == name) {
                out.push_str(v);
            } else {
                out.push_str(raw);
            }
        }
    });
    out
}

/// Convert a plain-text email body into minimal HTML: HTML-escape, split on
/// blank lines into paragraphs, and turn remaining line breaks into `<br/>`.
pub fn text_to_html(text: &str) -> String {
    let escaped = escape_html(text);
    let paragraphs: Vec<String> = escaped
        .split("\n\n")
        .map(|p| p.trim_matches('\n').replace('\n', "<br/>"))
        .filter(|p| !p.is_empty())
        .map(|p| format!("<p>{p}</p>"))
        .collect();
    paragraphs.join("")
}

// ── API request/response types ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub name: String,
    pub admin_mail: String,
    pub mails: Vec<String>,
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub lang: Lang,
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
    #[serde(default)]
    pub templates: EmailTemplates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDataRequest {
    pub slots: Vec<Slot>,
    pub participants: Vec<ParticipantInput>,
    #[serde(default)]
    pub send_mails: bool,
    #[serde(default)]
    pub templates: EmailTemplates,
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
    pub creation_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailsResponse {
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEmailRequest {
    pub password: String,
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
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

    // ── escape_html ───────────────────────────────────────────────

    #[test]
    fn escape_html_no_special_chars() {
        assert_eq!(escape_html("hello world"), "hello world");
    }

    #[test]
    fn escape_html_all_special_chars() {
        assert_eq!(
            escape_html(r#"<script>alert("x&y")</script>"#),
            "&lt;script&gt;alert(&quot;x&amp;y&quot;)&lt;/script&gt;"
        );
    }

    #[test]
    fn escape_html_single_quotes() {
        assert_eq!(escape_html("it's"), "it&#x27;s");
    }

    #[test]
    fn escape_html_empty() {
        assert_eq!(escape_html(""), "");
    }

    // ── render_template ───────────────────────────────────────────

    #[test]
    fn render_template_substitutes_known_vars() {
        let out = render_template(
            "Hi $event_name, go to $url.",
            &[("event_name", "Party"), ("url", "https://x/y")],
        );
        assert_eq!(out, "Hi Party, go to https://x/y.");
    }

    #[test]
    fn render_template_leaves_unknown_vars_literal() {
        let out = render_template("Start $ur end", &[("url", "https://x")]);
        assert_eq!(out, "Start $ur end");
    }

    #[test]
    fn render_template_stops_at_word_boundary() {
        // $url_X is one token (underscore/alnum extends the name); $url. stops at '.'
        let out = render_template("$url. and $url_x.", &[("url", "A")]);
        assert_eq!(out, "A. and $url_x.");
    }

    #[test]
    fn render_template_bare_dollar_is_literal() {
        assert_eq!(render_template("price: $5", &[]), "price: $5");
        assert_eq!(render_template("end$", &[]), "end$");
    }

    #[test]
    fn render_template_handles_unicode_text() {
        let out = render_template("héllo $name ✓", &[("name", "wörld")]);
        assert_eq!(out, "héllo wörld ✓");
    }

    // ── text_to_html ──────────────────────────────────────────────

    #[test]
    fn text_to_html_wraps_paragraphs() {
        let out = text_to_html("Hi,\n\nLine two.");
        assert_eq!(out, "<p>Hi,</p><p>Line two.</p>");
    }

    #[test]
    fn text_to_html_single_newline_is_br() {
        let out = text_to_html("one\ntwo");
        assert_eq!(out, "<p>one<br/>two</p>");
    }

    #[test]
    fn text_to_html_escapes_html() {
        let out = text_to_html("<script>a&b</script>");
        assert_eq!(out, "<p>&lt;script&gt;a&amp;b&lt;/script&gt;</p>");
    }

    // ── scan_template ─────────────────────────────────────────────

    #[test]
    fn scan_template_marks_known_and_unknown() {
        let mut spans = Vec::new();
        scan_template("x $url y $ur z", |span| match span {
            TemplateSpan::Text(t) => spans.push(("t", t.to_string(), false)),
            TemplateSpan::Var { name, known, .. } => spans.push(("v", name.to_string(), known)),
        });
        assert_eq!(
            spans,
            vec![
                ("t", "x ".into(), false),
                ("v", "url".into(), true),
                ("t", " y ".into(), false),
                ("v", "ur".into(), false),
                ("t", " z".into(), false),
            ]
        );
    }

    // ── Slot serialization ────────────────────────────────────────

    #[test]
    fn slot_round_trip_json() {
        let slot = Slot {
            name: "Morning".into(),
            vmin: 2,
            vmax: 5,
        };
        let json = serde_json::to_string(&slot).unwrap();
        let back: Slot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Morning");
        assert_eq!(back.vmin, 2);
        assert_eq!(back.vmax, 5);
    }

    #[test]
    fn participant_status_default_serialization() {
        let p = Participant {
            id: "abc".into(),
            mail: "test@x".into(),
            wish: vec![0, 1],
            event: "ev1".into(),
            status: ParticipantStatus::default(),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Participant = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, ParticipantStatus::New);
    }

    #[test]
    fn participant_missing_status_defaults_to_new() {
        let json = r#"{"id":"a","mail":"b@c","wish":[0],"event":"e"}"#;
        let p: Participant = serde_json::from_str(json).unwrap();
        assert_eq!(p.status, ParticipantStatus::New);
    }

    // ── WsMsg serialization ───────────────────────────────────────

    #[test]
    fn ws_msg_new_wish_round_trip() {
        let msg = WsMsg::NewWish { mail: "a@b".into() };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"NewWish\""));
        let back: WsMsg = serde_json::from_str(&json).unwrap();
        match back {
            WsMsg::NewWish { mail } => assert_eq!(mail, "a@b"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn ws_msg_mail_progress_round_trip() {
        let msg = WsMsg::MailProgress {
            sent: 3,
            total: 10,
            mail: "ok@x".into(),
            error: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: WsMsg = serde_json::from_str(&json).unwrap();
        match back {
            WsMsg::MailProgress {
                sent,
                total,
                mail,
                error,
            } => {
                assert_eq!(sent, 3);
                assert_eq!(total, 10);
                assert_eq!(mail, "ok@x");
                assert_eq!(error, None);
            }
            _ => panic!("wrong variant"),
        }
    }
}

// ── WebSocket messages ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMsg {
    NewWish {
        mail: String,
    },
    MailProgress {
        sent: usize,
        total: usize,
        mail: String,
        error: Option<String>,
    },
    Feedback {
        title: String,
        html: String,
        msg_type: String,
    },
}

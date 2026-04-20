# Wish

Scheduling tool: an admin creates an event with slots (each with min/max quotas) and
invites participants by email. Each participant rates slots 0..n-1 (n = number
of slots) under a fairness constraint: at most (n - v) slots may have rating ≥ v,
so e.g. only one slot can be rated the max. The server then assigns people to
slots minimizing total penalty via the Hungarian algorithm.

## Workspace

- `shared/` — types shared between client and server (`WsMsg`, `AdminData`,
  `EmailTemplates`, HTML-escape helper, Hungarian solver inputs).
- `server/` — Actix-web backend. `handlers.rs` owns all routes, `db.rs` is a
  JSON-file store (`db.json`), `email.rs` wraps the Resend HTTP API.
- `client/` — Leptos WASM SPA. Pages: `home` (create event), `admin` (manage +
  send mails, subscribes to a per-event WebSocket), `wish` (participant rates
  slots), `history`, `email` (debug), `offline`, `help`.

## Conventions

- Real-time updates: the admin page opens `/api/events/{id}/ws` and reacts to
  `WsMsg` broadcasts (`NewWish`, `MailProgress`, `Feedback`).
- i18n: four languages defined as static structs in `client/src/i18n.rs` — add
  a field to `Translations` *and* to all four language tables.
- Emails go through Resend; templates are per-event (`EmailTemplates`) with
  simple `{var}` placeholders rendered server-side.

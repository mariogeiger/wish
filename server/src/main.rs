use actix_files::{Files, NamedFile};
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use futures_util::StreamExt as _;
use std::collections::HashMap;
use std::net::TcpListener;
use std::os::unix::io::FromRawFd;
use std::sync::Mutex;

mod db;
mod email;
mod handlers;

use db::{Db, load_db, save_db};
use email::ResendClient;

// ── Config ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub history_password: String,
    pub base_url: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            port: std::env::var("WISH_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
            history_password: std::env::var("WISH_HISTORY_PASSWORD")
                .unwrap_or_else(|_| "changeme".to_string()),
            base_url: std::env::var("WISH_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
        }
    }
}

// ── App state ──────────────────────────────────────────────────────

pub struct AppState {
    db: Mutex<Db>,
    rooms: Mutex<HashMap<String, tokio::sync::broadcast::Sender<String>>>,
    resend: ResendClient,
    config: Config,
}

impl AppState {
    fn with_db<R>(&self, f: impl FnOnce(&mut Db) -> R) -> R {
        f(&mut self.db.lock().unwrap())
    }

    fn with_db_save<R>(&self, f: impl FnOnce(&mut Db) -> R) -> R {
        let mut db = self.db.lock().unwrap();
        let result = f(&mut db);
        save_db(&db);
        result
    }

    fn subscribe(&self, event_id: &str) -> tokio::sync::broadcast::Receiver<String> {
        self.rooms
            .lock()
            .unwrap()
            .entry(event_id.to_string())
            .or_insert_with(|| tokio::sync::broadcast::channel(64).0)
            .subscribe()
    }

    fn broadcast(&self, event_id: &str, msg: &str) {
        let rooms = self.rooms.lock().unwrap();
        if let Some(tx) = rooms.get(event_id) {
            let _ = tx.send(msg.to_string());
        }
    }

    pub fn get_broadcast(&self, event_id: &str) -> tokio::sync::broadcast::Sender<String> {
        self.rooms
            .lock()
            .unwrap()
            .entry(event_id.to_string())
            .or_insert_with(|| tokio::sync::broadcast::channel(64).0)
            .clone()
    }
}

pub fn gen_id() -> String {
    let bytes: [u8; 20] = rand::random();
    hex::encode(bytes)
}

// ── WebSocket ──────────────────────────────────────────────────────

async fn event_ws(
    req: actix_web::HttpRequest,
    body: web::Payload,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, msg_stream) = actix_ws::handle(&req, body)?;
    let event_id = path.into_inner();
    let mut rx = state.subscribe(&event_id);
    let mut ws = msg_stream;

    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {
                msg = ws.next() => {
                    match msg {
                        Some(Ok(actix_ws::Message::Ping(data))) => {
                            let _ = session.pong(&data).await;
                        }
                        Some(Ok(actix_ws::Message::Text(_))) => {
                            // Client messages not used currently
                        }
                        _ => break,
                    }
                }
                bcast = rx.recv() => {
                    match bcast {
                        Ok(text) => {
                            if session.text(text).await.is_err() { break; }
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });

    Ok(response)
}

// ── SPA fallback ───────────────────────────────────────────────────

async fn spa_fallback() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./client/dist/index.html")?)
}

// ── Main ───────────────────────────────────────────────────────────

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let config = Config::from_env();
    let port = config.port;

    let resend_key =
        std::env::var("RESEND_API_KEY").unwrap_or_else(|_| "re_test_key".to_string());
    let resend_from = std::env::var("WISH_SENDER_EMAIL")
        .unwrap_or_else(|_| "Wish <wish@geiger.ink>".to_string());

    let state = web::Data::new(AppState {
        db: Mutex::new(load_db()),
        rooms: Mutex::new(HashMap::new()),
        resend: ResendClient::new(resend_key, resend_from),
        config,
    });

    log::info!("Server running at http://localhost:{port}");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::new("%a \"%r\" %s %b %Dms"))
            .route("/health", web::get().to(handlers::health))
            // Event CRUD
            .route("/api/events", web::post().to(handlers::create_event))
            .route("/api/events/{id}", web::get().to(handlers::get_admin_data))
            .route("/api/events/{id}", web::put().to(handlers::set_admin_data))
            // Email actions
            .route(
                "/api/events/{id}/send-mails",
                web::post().to(handlers::send_mails),
            )
            .route(
                "/api/events/{id}/remind",
                web::post().to(handlers::send_reminders),
            )
            .route(
                "/api/events/{id}/results",
                web::post().to(handlers::send_results),
            )
            // WebSocket
            .route("/api/events/{id}/ws", web::get().to(event_ws))
            // Wish endpoints
            .route("/api/wish/{pid}", web::get().to(handlers::get_wish))
            .route("/api/wish/{pid}", web::put().to(handlers::set_wish))
            // History
            .route("/api/history", web::post().to(handlers::get_history))
            // Static files (client dist)
            .service(Files::new("/", "./client/dist").index_file("index.html"))
            // SPA fallback: serve index.html for client-side routes
            .default_service(web::get().to(spa_fallback))
    })
    .shutdown_timeout(1);

    let server = if std::env::var("LISTEN_FDS")
        .map(|v| v.parse::<u32>().unwrap_or(0))
        .unwrap_or(0)
        >= 1
    {
        let listener = unsafe { TcpListener::from_raw_fd(3) };
        server.listen(listener)?
    } else {
        server.bind(format!("0.0.0.0:{port}"))?
    };

    server.run().await
}

use std::path::PathBuf;
use wish_shared::{Event, Participant};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Db {
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub participants: Vec<Participant>,
}

pub fn db_path() -> PathBuf {
    PathBuf::from("db.json")
}

pub fn load_db() -> Db {
    std::fs::read_to_string(db_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_db(db: &Db) {
    let json = match serde_json::to_string_pretty(db) {
        Ok(j) => j,
        Err(e) => {
            log::error!("Failed to serialize db: {e}");
            return;
        }
    };
    let tmp = db_path().with_extension("tmp");
    if let Err(e) = std::fs::write(&tmp, &json) {
        log::error!("Failed to write {}: {e}", tmp.display());
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, db_path()) {
        log::error!(
            "Failed to rename {} -> {}: {e}",
            tmp.display(),
            db_path().display()
        );
        return;
    }
    backup_db();
}

const BACKUP_DIR: &str = "backups";

fn backup_db() {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (year, month, day, hour) = unix_secs_to_ymdh(secs);
    let name = format!("{BACKUP_DIR}/db_{year:04}{month:02}{day:02}_{hour:02}.json");

    if std::path::Path::new(&name).exists() {
        return;
    }

    std::fs::create_dir_all(BACKUP_DIR).ok();
    if std::fs::copy(db_path(), &name).is_ok() {
        log::info!("backup: {name}");
    }
}

fn unix_secs_to_ymdh(secs: u64) -> (u64, u64, u64, u64) {
    let hour = (secs / 3600) % 24;
    let days = secs / 86400;
    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let days_in_year =
            if y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400)) {
                366
            } else {
                365
            };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let leap = y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0;
    while m < 12 && remaining >= month_days[m] {
        remaining -= month_days[m];
        m += 1;
    }
    (y, (m + 1) as u64, (remaining + 1), hour)
}

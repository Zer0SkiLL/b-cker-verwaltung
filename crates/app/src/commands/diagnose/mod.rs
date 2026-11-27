//! Frühwarnung in der Einführungsphase: Diagnose-Ordner neben der Datei und befristetes Senden
//! an einen Discord-Webhook. Die Webhook-Adresse kommt nur beim Release-Build hinein
//! (`MV_DIAGNOSE_WEBHOOK`) und erscheint nie in Protokoll, Bericht oder Oberfläche.

pub mod discord;
pub mod outbox;
pub mod settings;

use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use settings::DiagnoseSettings;

/// Letzter Tag, an dem gesendet wird.
pub const DIAGNOSE_UNTIL: NaiveDate = match NaiveDate::from_ymd_opt(2027, 5, 31) {
    Some(d) => d,
    None => panic!("ungültiges Enddatum"),
};
/// Vorgabe des Schalters „Analysedaten senden“; per neuer Version änderbar.
pub const DIAGNOSE_DEFAULT: bool = true;
pub const MAX_PER_DAY: u32 = 10;
pub const MAX_OUTBOX: usize = 20;
pub const HEARTBEAT_DAYS: i64 = 7;
pub const FOLDER_NAME: &str = "Mitgliederverwaltung-Diagnose";

/// Protokollzeilen aus diesen Modulen lösen keine Frühwarnung aus (sonst Schleife beim Senden).
const QUIET_TARGETS: [&str; 5] = [
    "verwaltung_app::commands::diagnose",
    "reqwest",
    "hyper",
    "rustls",
    "h2",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Absturz,
    Warnung,
    Selbsttest,
    Lebenszeichen,
}

impl Kind {
    pub fn label(&self) -> &'static str {
        match self {
            Kind::Absturz => "Absturz",
            Kind::Warnung => "Warnung",
            Kind::Selbsttest => "Selbsttest",
            Kind::Lebenszeichen => "Lebenszeichen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub kind: Kind,
    pub host: String,
    pub version: String,
    pub status_block: String,
    pub report: String,
    pub at: DateTime<Local>,
}

/// Im Release-Build eingebaute Webhook-Adresse; sonst `None` (Senden still aus).
pub fn webhook() -> Option<&'static str> {
    option_env!("MV_DIAGNOSE_WEBHOOK").filter(|s| !s.trim().is_empty())
}

/// Ob gesendet wird: Adresse vorhanden, Enddatum nicht überschritten, Schalter ein.
pub fn active_with(url: Option<&str>, s: &DiagnoseSettings, today: NaiveDate) -> bool {
    url.is_some_and(|u| !u.trim().is_empty())
        && today <= DIAGNOSE_UNTIL
        && s.enabled.unwrap_or(DIAGNOSE_DEFAULT)
}

pub fn active(s: &DiagnoseSettings, today: NaiveDate) -> bool {
    active_with(webhook(), s, today)
}

/// Lebenszeichen fällig: noch nie oder vor mehr als 7 Tagen gesendet.
pub fn heartbeat_due(s: &DiagnoseSettings, now: DateTime<Local>) -> bool {
    s.last_heartbeat
        .as_deref()
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .is_none_or(|last| now.signed_duration_since(last) > Duration::days(HEARTBEAT_DAYS))
}

/// Welche Protokollzeile eine Frühwarnung auslöst.
pub fn event_kind(level: log::Level, target: &str, message: &str) -> Option<Kind> {
    if level > log::Level::Warn || QUIET_TARGETS.iter().any(|t| target.starts_with(t)) {
        return None;
    }
    Some(if message.starts_with("Absturz") {
        Kind::Absturz
    } else if message.starts_with("Selbsttest") {
        Kind::Selbsttest
    } else {
        Kind::Warnung
    })
}

/// Nur Buchstaben, Ziffern und Bindestrich (für Dateinamen); alles andere wird zu „_“.
pub fn safe_name(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// `<Ordner der Datei>/Mitgliederverwaltung-Diagnose/<PC>.txt`; Fehler nur ins Protokoll.
pub fn write_folder_file(db_path: &Path, host: &str, report: &str) {
    let Some(dir) = db_path.parent().map(|d| d.join(FOLDER_NAME)) else {
        return;
    };
    let result = std::fs::create_dir_all(&dir)
        .and_then(|_| std::fs::write(dir.join(format!("{}.txt", safe_name(host))), report));
    if let Err(e) = result {
        log::info!("Diagnose-Ordner nicht beschreibbar: {}", e.kind());
    }
}

/// Sendet, was im Ausgang liegt (der Reihe nach, höchstens 10 pro Tag). Läuft nie doppelt.
/// Gibt die Anzahl gesendeter Nachrichten zurück.
pub fn flush(outbox: &Path, url: &str, s: &mut DiagnoseSettings, now: DateTime<Local>) -> usize {
    static FLUSHING: Mutex<()> = Mutex::new(());
    let Ok(_guard) = FLUSHING.lock() else {
        return 0;
    };
    let today = now.date_naive().to_string();
    if s.sent_day.as_deref() != Some(today.as_str()) {
        s.sent_day = Some(today);
        s.sent_count = 0;
    }
    let mut sent = 0;
    for (path, msg) in outbox::pending(outbox) {
        if s.sent_count >= MAX_PER_DAY {
            log::info!("Analysedaten: Tageshöchstzahl erreicht, Nachricht verworfen");
            outbox::remove(&path);
            continue;
        }
        match discord::send(url, &msg) {
            Ok(()) => {
                outbox::remove(&path);
                s.sent_count += 1;
                s.last_sent = Some(now.to_rfc3339());
                if msg.kind == Kind::Lebenszeichen {
                    s.last_heartbeat = Some(now.to_rfc3339());
                }
                sent += 1;
            }
            Err(reason) => {
                log::info!("Analysedaten nicht gesendet: {reason}");
                break;
            }
        }
    }
    sent
}

/// Ordner-Datei immer; bei aktivem Senden in den Ausgang legen und im Hintergrund senden.
/// Gibt den Sende-Thread zurück (für Tests), sonst `None`.
pub fn publish_with(
    url: Option<&str>,
    config_dir: &Path,
    outbox_dir: &Path,
    db_path: Option<&Path>,
    msg: Message,
) -> Option<std::thread::JoinHandle<()>> {
    if let Some(db) = db_path {
        write_folder_file(db, &msg.host, &msg.report);
    }
    let settings = settings::load(config_dir);
    if !active_with(url, &settings, msg.at.date_naive()) {
        return None;
    }
    if let Err(e) = outbox::enqueue(outbox_dir, &msg) {
        log::info!("Analysedaten nicht abgelegt: {e}");
        return None;
    }
    Some(spawn_flush(
        url?.to_string(),
        config_dir.to_path_buf(),
        outbox_dir.to_path_buf(),
    ))
}

/// Sendet den Ausgang in einem eigenen Thread (nie im Thread der Oberfläche oder von Tokio).
pub fn spawn_flush(
    url: String,
    config_dir: std::path::PathBuf,
    outbox_dir: std::path::PathBuf,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut s = settings::load(&config_dir);
        flush(&outbox_dir, &url, &mut s, Local::now());
        if let Err(e) = settings::save(&config_dir, &s) {
            log::info!("Analysedaten: Einstellungen nicht gespeichert: {e}");
        }
    })
}

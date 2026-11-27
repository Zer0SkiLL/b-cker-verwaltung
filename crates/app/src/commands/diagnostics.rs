//! Angaben für „Problem melden“: relevante Protokollzeilen und System. Enthält keine Vereinsdaten.

use std::path::Path;

use serde::Serialize;

use chrono::{Local, NaiveDate, TimeZone, Utc};
use verwaltung_core::repo::memberships;
use verwaltung_core::storage::backup;

use super::location::location_of;
use super::logfile::{
    excerpt_indices, log_excerpt, problems_today, read_log_lines, sanitize, start_history, StartDto,
};
use super::resources::{measure, ResourcesDto};
use crate::error::AppResult;
use crate::session::Session;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SystemInfoDto {
    pub os: String,
    pub arch: String,
    pub version: String,
}

pub fn system_info() -> SystemInfoDto {
    let info = os_info::get();
    SystemInfoDto {
        os: format!("{} {}", info.os_type(), info.version()),
        arch: std::env::consts::ARCH.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Letzte Warnungen/Fehler (höchstens 40) und die letzten 10 Zeilen, roh und zeitlich geordnet.
pub fn log_tail(log_dir: &Path) -> AppResult<Vec<String>> {
    let lines = read_log_lines(log_dir)?;
    Ok(excerpt_indices(&lines)
        .into_iter()
        .map(|i| lines[i].clone())
        .collect())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileFactsDto {
    /// Art des Ortes, z.B. „Netzlaufwerk (S:)“ – nie der Pfad.
    pub location: String,
    pub reachable: bool,
    pub size: Option<u64>,
    pub schema_version: Option<i64>,
    pub organisations: Option<i64>,
    pub persons: Option<i64>,
    pub active_members: Option<i64>,
    /// RFC 3339 (lokal) der neuesten Sicherung neben der Datei.
    pub last_local_backup: Option<String>,
    /// PC, der die Sperre hält (nur bei „nur lesend“); der Benutzername bleibt draussen.
    pub lock_host: Option<String>,
    pub lock_since: Option<String>,
    pub lock_likely_crash: bool,
}

fn count(s: &Session, table: &str) -> Option<i64> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    s.read(|c| Ok(c.query_row(&sql, [], |r| r.get(0))?)).ok()
}

/// Angaben zur geöffneten Datei; `None` ohne Datei. Jeder Einzelwert darf fehlen.
pub fn file_facts(s: &mut Session, today: NaiveDate) -> Option<FileFactsDto> {
    let path = s.workspace_mut().ok()?.path().to_path_buf();
    let lock = s.info(Utc::now()).ok().and_then(|i| i.held_by);
    let size = std::fs::metadata(&path).ok().map(|m| m.len());
    let last_local_backup = backup::list(&backup::local_backup_dir(&path), &backup::db_stem(&path))
        .ok()
        .and_then(|files| files.into_iter().map(|f| f.created).max())
        .and_then(|t| Local.from_local_datetime(&t).earliest())
        .map(|t| t.to_rfc3339());
    Some(FileFactsDto {
        location: location_of(&path).label(),
        reachable: size.is_some(),
        size,
        schema_version: s.read(verwaltung_core::db::schema_version).ok(),
        organisations: count(s, "organisations"),
        persons: count(s, "persons"),
        active_members: s.read(|c| memberships::count_active_members(c, today)).ok(),
        last_local_backup,
        lock_likely_crash: lock.as_ref().is_some_and(|l| l.likely_crash),
        lock_since: lock.as_ref().map(|l| l.since.clone()),
        lock_host: lock.map(|l| l.host),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub system: SystemInfoDto,
    pub host: String,
    pub resources: ResourcesDto,
    pub file: Option<FileFactsDto>,
    pub starts: Vec<StartDto>,
    pub problems_today: usize,
    /// Aufbereiteter Protokoll-Auszug, älteste zuerst.
    pub log: Vec<String>,
}

/// Wie viele Programmstarts der Bericht zeigt.
const STARTS: usize = 5;

/// Alles für den Problembericht in einem Aufruf; scheitert nie.
pub fn snapshot(s: &mut Session, log_dir: &Path, host: &str) -> SnapshotDto {
    let file = file_facts(s, Local::now().date_naive());
    snapshot_with(file, log_dir, host)
}

/// Wie [`snapshot`], mit bereits gelesenen Datei-Angaben (die Messung braucht die Sitzung nicht).
pub fn snapshot_with(file: Option<FileFactsDto>, log_dir: &Path, host: &str) -> SnapshotDto {
    let today = Local::now().date_naive();
    let lines = read_log_lines(log_dir).unwrap_or_default();
    SnapshotDto {
        system: system_info(),
        host: host.to_string(),
        resources: measure(),
        file,
        starts: start_history(&lines, STARTS),
        problems_today: problems_today(&lines, today),
        log: log_excerpt(&lines),
    }
}

/// Längste Nachricht aus der Oberfläche im Protokoll.
const MAX_UI_MESSAGE: usize = 500;

/// `Absturz: <Meldung> (<Datei>:<Zeile>)`.
pub fn panic_message(info: &std::panic::PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let text = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unbekannter Fehler".into());
    let place = info
        .location()
        .map(|l| format!(" ({}:{})", l.file(), l.line()))
        .unwrap_or_default();
    sanitize(&format!("Absturz: {text}{place}"))
}

/// Schreibt Abstürze (Panics) ins Protokoll, danach wie gewohnt. Mehrfacher Aufruf schadet nicht.
pub fn install_panic_hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            log::error!("{}", panic_message(info));
            previous(info);
        }));
    });
}

/// Protokollzeile aus der Oberfläche: gekürzt und mit maskierten Benutzerordnern.
pub fn log_from_ui(level: &str, message: &str) {
    let short: String = message.chars().take(MAX_UI_MESSAGE).collect();
    let text = sanitize(&short);
    match level {
        "warn" => log::warn!("{text}"),
        "error" => log::error!("{text}"),
        _ => log::info!("{text}"),
    }
}

/// Speichert den Problembericht als Textdatei (UTF-8).
pub fn save_report(path: &Path, text: &str) -> AppResult<()> {
    std::fs::write(path, text)?;
    Ok(())
}

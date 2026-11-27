//! Selbsttest nach dem Öffnen der Datei. Befunde landen als Warnung im Protokoll (und damit in
//! der Frühwarnung); die Oberfläche zeigt nur, was der Nutzer selbst beheben kann.

use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use verwaltung_core::storage::backup;

use crate::error::AppResult;
use crate::session::Session;

/// Weniger freier Platz auf dem Laufwerk der Datei gilt als knapp.
pub const LOW_SPACE_BYTES: u64 = 500 * 1024 * 1024;
/// Älter darf die neueste Sicherung neben der Datei nicht sein.
pub const BACKUP_MAX_AGE_DAYS: i64 = 7;
/// Länger darf das Öffnen nicht dauern.
pub const SLOW_OPEN_MS: u128 = 3000;

/// Gemessene Werte, getrennt von der Bewertung (testbar ohne Laufwerk).
#[derive(Debug, Clone)]
pub struct Facts {
    pub integrity: String,
    pub free_bytes: Option<u64>,
    pub newest_backup_age_days: Option<i64>,
    pub writable: bool,
    pub open_ms: Option<u128>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfTestDto {
    pub integrity_problem: bool,
    pub low_space: bool,
    pub backup_stale: bool,
    pub slow_open: bool,
}

impl SelfTestDto {
    pub fn any(&self) -> bool {
        self.integrity_problem || self.low_space || self.backup_stale || self.slow_open
    }
}

pub fn evaluate(f: &Facts) -> SelfTestDto {
    SelfTestDto {
        integrity_problem: f.integrity.trim() != "ok",
        low_space: f.free_bytes.is_some_and(|b| b < LOW_SPACE_BYTES),
        backup_stale: f.writable
            && f.newest_backup_age_days
                .is_none_or(|d| d > BACKUP_MAX_AGE_DAYS),
        slow_open: f.open_ms.is_some_and(|ms| ms > SLOW_OPEN_MS),
    }
}

fn log_findings(f: &Facts, r: &SelfTestDto) {
    if !r.any() {
        log::info!("Selbsttest: in Ordnung");
        return;
    }
    if r.integrity_problem {
        log::warn!("Selbsttest: Datei fehlerhaft (quick_check)");
    }
    if r.low_space {
        let mb = f.free_bytes.unwrap_or(0) / (1024 * 1024);
        log::warn!("Selbsttest: wenig Platz auf dem Laufwerk ({mb} MB frei)");
    }
    if r.backup_stale {
        match f.newest_backup_age_days {
            Some(d) => log::warn!("Selbsttest: neueste Sicherung neben der Datei ist {d} Tage alt"),
            None => log::warn!("Selbsttest: keine Sicherung neben der Datei"),
        }
    }
    if r.slow_open {
        log::warn!("Selbsttest: Öffnen dauerte {} ms", f.open_ms.unwrap_or(0));
    }
}

/// Misst und bewertet; scheitert nur ohne geöffnete Datei.
pub fn self_test(s: &mut Session, now: DateTime<Local>) -> AppResult<SelfTestDto> {
    let path = s.workspace_mut()?.path().to_path_buf();
    let writable = !s.info(now.with_timezone(&Utc))?.read_only;
    let integrity = s
        .read(|c| Ok(c.query_row("PRAGMA quick_check", [], |r| r.get::<_, String>(0))?))
        .unwrap_or_else(|e| format!("Fehler: {e}"));
    let free_bytes = path.parent().and_then(|dir| fs4::available_space(dir).ok());
    let newest_backup_age_days =
        backup::list(&backup::local_backup_dir(&path), &backup::db_stem(&path))
            .ok()
            .and_then(|files| files.into_iter().map(|f| f.created).max())
            .map(|created| (now.naive_local() - created).num_days());
    let facts = Facts {
        integrity,
        free_bytes,
        newest_backup_age_days,
        writable,
        open_ms: s.open_ms(),
    };
    let result = evaluate(&facts);
    log_findings(&facts, &result);
    Ok(result)
}

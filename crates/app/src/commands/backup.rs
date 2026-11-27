//! Datensicherung, Wiederherstellen und Gesamtexport (Spec §8).

use std::path::Path;

use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use verwaltung_core::export_full::full_export;
use verwaltung_core::repo::settings::{self, SECONDARY_BACKUP_DIR};
use verwaltung_core::storage::backup::{self, db_stem, local_backup_dir};
use verwaltung_core::storage::retention::{load_policy, save_policy, RetentionPolicy};
use verwaltung_core::storage::workspace::BackupOutcome;

use crate::dto::{BackupStatusDto, SecondaryResultDto};
use crate::error::{AppError, AppResult};
use crate::session::Session;

pub const DIR_MISSING: &str = "Der Ordner existiert nicht.";
const STAMP_FORMAT: &str = "%d.%m.%Y %H:%M";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackupFileDto {
    pub path: String,
    /// TT.MM.JJJJ HH:MM
    pub created: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupOverview {
    pub status: BackupStatusDto,
    pub local_dir: String,
    pub files: Vec<BackupFileDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackupNowDto {
    pub local: String,
    pub secondary: SecondaryResultDto,
}

/// Stand der Sicherungen und lokale Sicherungen, neueste zuerst.
pub fn backup_overview(s: &mut Session, now: DateTime<Utc>) -> AppResult<BackupOverview> {
    let ws = s.workspace_mut()?;
    let dir = local_backup_dir(ws.path());
    let files = backup::list(&dir, &db_stem(ws.path()))?
        .into_iter()
        .map(|f| BackupFileDto {
            path: f.path.display().to_string(),
            created: f.created.format(STAMP_FORMAT).to_string(),
        })
        .collect();
    Ok(BackupOverview {
        status: backup::status(&ws.conn, now)?.into(),
        local_dir: dir.display().to_string(),
        files,
    })
}

/// Setzt oder entfernt (`None`) den zweiten Sicherungsort.
pub fn set_secondary_dir(s: &mut Session, dir: Option<&Path>) -> AppResult<()> {
    let result = match dir {
        Some(dir) if !dir.is_dir() => Err(AppError::new(DIR_MISSING)),
        Some(dir) => {
            let value = dir.display().to_string();
            s.write(|c| settings::set(c, SECONDARY_BACKUP_DIR, &value))
        }
        None => s.write(|c| settings::remove(c, SECONDARY_BACKUP_DIR)),
    };
    if result.is_ok() {
        s.clear_secondary_error();
    }
    result
}

pub fn backup_now(s: &mut Session, now: DateTime<Local>) -> AppResult<BackupNowDto> {
    let outcome = s.writable()?.backup_now(now)?;
    Ok(outcome_dto(s, outcome))
}

/// Sicherung während der Arbeit: nur wenn seit der letzten Sicherung etwas geändert wurde.
pub fn backup_if_changed(s: &mut Session, now: DateTime<Local>) -> AppResult<Option<BackupNowDto>> {
    let outcome = s.workspace_mut()?.backup_if_changed(now)?;
    Ok(outcome.map(|o| outcome_dto(s, o)))
}

fn outcome_dto(s: &mut Session, outcome: BackupOutcome) -> BackupNowDto {
    s.note_secondary(&outcome.secondary);
    BackupNowDto {
        local: outcome.local.display().to_string(),
        secondary: outcome.secondary.into(),
    }
}

pub fn get_retention(s: &Session) -> AppResult<RetentionPolicy> {
    s.read(|c| Ok(load_policy(c)))
}

pub fn set_retention(s: &mut Session, policy: &RetentionPolicy) -> AppResult<()> {
    s.write(|c| save_policy(c, policy))
}

/// Stellt eine Sicherung wieder her; der aktuelle Stand wird vorher gesichert (Core).
pub fn restore_backup(s: &mut Session, backup: &Path, now: DateTime<Local>) -> AppResult<()> {
    Ok(s.writable()?.restore(backup, now)?)
}

/// Alle Daten als Excel-Datei (Exit-Strategie, auch ohne Programm lesbar).
pub fn export_all(s: &Session, path: &Path) -> AppResult<()> {
    let started = std::time::Instant::now();
    s.read(|c| full_export(c, path))?;
    log::info!("Gesamtexport: {} ms", started.elapsed().as_millis());
    Ok(())
}

//! Serialisierbare Abbilder von Core-Typen ohne serde (camelCase für TypeScript).

use chrono::{DateTime, Utc};
use serde::Serialize;
use verwaltung_core::storage::backup::BackupStatus;
use verwaltung_core::storage::lock::{is_stale, Identity, LockInfo};
use verwaltung_core::storage::workspace::SecondaryResult;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub path: String,
    pub read_only: bool,
    pub held_by: Option<LockDto>,
    pub backup: BackupStatusDto,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockDto {
    pub user: String,
    pub host: String,
    /// RFC 3339.
    pub since: String,
    /// Eigene Sperre (gleiche Person, gleicher Rechner) oder älter als 24 h: vermutlich nach Absturz.
    pub likely_crash: bool,
}

impl LockDto {
    pub fn new(info: &LockInfo, own: Option<&Identity>, now: DateTime<Utc>) -> LockDto {
        let mine = own.is_some_and(|me| me.user == info.user && me.host == info.host);
        LockDto {
            user: info.user.clone(),
            host: info.host.clone(),
            since: info.created_at.to_rfc3339(),
            likely_crash: mine || is_stale(info, now),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatusDto {
    pub secondary_dir: Option<String>,
    /// RFC 3339.
    pub last_secondary: Option<String>,
    pub reminder_due: bool,
}

impl From<BackupStatus> for BackupStatusDto {
    fn from(status: BackupStatus) -> BackupStatusDto {
        BackupStatusDto {
            secondary_dir: status.secondary_dir,
            last_secondary: status.last_secondary.map(|d| d.to_rfc3339()),
            reminder_due: status.reminder_due,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SecondaryResultDto {
    NotConfigured,
    Done { path: String },
    Unavailable { reason: String },
}

impl From<SecondaryResult> for SecondaryResultDto {
    fn from(result: SecondaryResult) -> SecondaryResultDto {
        match result {
            SecondaryResult::NotConfigured => SecondaryResultDto::NotConfigured,
            SecondaryResult::Done(path) => SecondaryResultDto::Done {
                path: path.display().to_string(),
            },
            SecondaryResult::Unavailable(reason) => SecondaryResultDto::Unavailable { reason },
        }
    }
}

//! Serialisierbare Abbilder von Core-Typen ohne serde (camelCase für TypeScript).

use serde::Serialize;
use verwaltung_core::storage::backup::BackupStatus;
use verwaltung_core::storage::lock::LockInfo;
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
}

impl From<&LockInfo> for LockDto {
    fn from(info: &LockInfo) -> LockDto {
        LockDto {
            user: info.user.clone(),
            host: info.host.clone(),
            since: info.created_at.to_rfc3339(),
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Inhalt einer Sperrdatei: wer die Datenbank geöffnet hat.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LockInfo {
    pub user: String,
    pub host: String,
    pub created_at: DateTime<Utc>,
}

/// Identität der Person, die eine Datenbank öffnet.
#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    pub user: String,
    pub host: String,
}

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{CoreError, Result};

const STALE_HOURS: i64 = 24;
const UNKNOWN: &str = "unbekannt";

/// Pfad der Sperrdatei: `<db_path>.lock`.
pub fn lock_path(db_path: &Path) -> PathBuf {
    let mut name = db_path.as_os_str().to_os_string();
    name.push(".lock");
    PathBuf::from(name)
}

/// Eine Sperre gilt als veraltet (nur Hinweis), wenn sie älter als 24 Stunden ist.
pub fn is_stale(info: &LockInfo, now: DateTime<Utc>) -> bool {
    now - info.created_at > chrono::Duration::hours(STALE_HOURS)
}

/// Gehaltene Sperre. Beim Freigeben wird die Datei nur gelöscht, wenn sie noch uns gehört.
#[derive(Debug)]
pub struct LockGuard {
    path: PathBuf,
    info: LockInfo,
}

impl LockGuard {
    /// Legt die Sperrdatei atomar an. Existiert sie, wird `LockHeld` mit dem Inhaber geliefert.
    pub fn acquire(db_path: &Path, who: &Identity) -> Result<LockGuard> {
        let path = lock_path(db_path);
        let info = new_info(who);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(&serde_json::to_vec(&info).expect("serialisierbar"))?;
                file.sync_all()?;
                Ok(LockGuard { path, info })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(CoreError::LockHeld(read_info(&path)))
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Übernimmt eine bestehende Sperre (nach Rückfrage in der Oberfläche).
    pub fn take_over(db_path: &Path, who: &Identity) -> Result<LockGuard> {
        let path = lock_path(db_path);
        let info = new_info(who);
        fs::write(&path, serde_json::to_vec(&info).expect("serialisierbar"))?;
        Ok(LockGuard { path, info })
    }

    pub fn info(&self) -> &LockInfo {
        &self.info
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let still_ours = fs::read(&self.path)
            .ok()
            .and_then(|b| serde_json::from_slice::<LockInfo>(&b).ok())
            .is_some_and(|i| i == self.info);
        if still_ours {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn new_info(who: &Identity) -> LockInfo {
    LockInfo {
        user: who.user.clone(),
        host: who.host.clone(),
        created_at: Utc::now(),
    }
}

fn read_info(path: &Path) -> LockInfo {
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<LockInfo>(&b).ok())
        .unwrap_or_else(|| LockInfo {
            user: UNKNOWN.into(),
            host: UNKNOWN.into(),
            created_at: fs::metadata(path)
                .and_then(|m| m.modified())
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(|_| Utc::now()),
        })
}

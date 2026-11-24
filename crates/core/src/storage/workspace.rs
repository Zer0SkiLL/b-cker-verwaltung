use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

use super::backup::{backup_to_dir, db_stem, local_backup_dir, prune, BACKUP_KEEP};
use super::lock::{current_holder, Identity, LockGuard, LockInfo};
use crate::db;
use crate::error::{CoreError, Result};
use crate::repo::settings::{self, LAST_SECONDARY_BACKUP, SECONDARY_BACKUP_DIR};

/// Zugriffsart auf die geöffnete Datenbank.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessMode {
    ReadWrite,
    ReadOnly { held_by: LockInfo },
}

/// Ergebnis der Sicherung am zweiten Ort.
#[derive(Debug, Clone, PartialEq)]
pub enum SecondaryResult {
    NotConfigured,
    Done(PathBuf),
    /// Ort nicht erreichbar (z.B. USB-Stick nicht eingesteckt); deutsche Meldung.
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackupOutcome {
    pub local: PathBuf,
    pub secondary: SecondaryResult,
}

/// Geöffnete Datenbank inklusive Sperre. Felder werden in Deklarationsreihenfolge freigegeben:
/// zuerst die Verbindung, dann die Sperre.
pub struct Workspace {
    pub conn: Connection,
    pub mode: AccessMode,
    path: PathBuf,
    lock: Option<LockGuard>,
}

impl Workspace {
    /// Öffnet schreibend. Ist die Datei gesperrt, wird sie nur lesend geöffnet.
    pub fn open(db_path: &Path, who: &Identity) -> Result<Workspace> {
        match LockGuard::acquire(db_path, who) {
            Ok(lock) => Self::open_read_write(db_path, lock),
            Err(CoreError::LockHeld(info)) if db_path.exists() => {
                Self::open_read_only(db_path, info)
            }
            Err(err) => Err(err),
        }
    }

    /// Übernimmt eine fremde Sperre und öffnet schreibend.
    pub fn open_taking_over(db_path: &Path, who: &Identity) -> Result<Workspace> {
        let lock = LockGuard::take_over(db_path, who)?;
        Self::open_read_write(db_path, lock)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_read_write(&self) -> bool {
        matches!(self.mode, AccessMode::ReadWrite)
    }

    /// Sichert lokal und (falls eingerichtet) am zweiten Ort. Nur bei schreibendem Zugriff.
    pub fn backup_now(&self, now: DateTime<Local>) -> Result<BackupOutcome> {
        self.require_read_write()?;
        let dir = local_backup_dir(&self.path);
        let stem = db_stem(&self.path);
        let local = backup_to_dir(&self.conn, &dir, &stem, now)?;
        prune(&dir, &stem, BACKUP_KEEP)?;
        Ok(BackupOutcome {
            local,
            secondary: self.backup_secondary(now)?,
        })
    }

    /// Prüft, ob die eigene Sperre noch besteht. Wurde sie übernommen (z.B. durch die
    /// Ferienvertretung), wird die Datenbank nur noch lesend geöffnet. Vor dem Schreiben aufrufen.
    pub fn check_lock(&mut self) -> Result<&AccessMode> {
        let lost = self.lock.as_ref().is_some_and(|lock| !lock.still_held());
        if lost {
            let held_by = current_holder(&self.path).unwrap_or_else(|| LockInfo {
                user: "unbekannt".into(),
                host: "unbekannt".into(),
                created_at: Utc::now(),
            });
            self.conn = db::open_connection(&self.path, true)?;
            self.mode = AccessMode::ReadOnly { held_by };
            self.lock = None;
        }
        Ok(&self.mode)
    }

    /// Schliesst die Datenbank: Sicherung am zweiten Ort, danach wird die Sperre freigegeben.
    pub fn close(mut self, now: DateTime<Local>) -> Result<SecondaryResult> {
        self.check_lock()?;
        if !self.is_read_write() {
            return Ok(SecondaryResult::NotConfigured);
        }
        self.backup_secondary(now)
    }

    /// Schreibend geöffnet und die eigene Sperre besteht noch.
    pub(crate) fn require_read_write(&self) -> Result<()> {
        let lock_ok = self.lock.as_ref().is_none_or(|lock| lock.still_held());
        if self.is_read_write() && lock_ok {
            Ok(())
        } else {
            Err(CoreError::ReadOnly)
        }
    }

    fn from_parts(conn: Connection, path: PathBuf, lock: LockGuard) -> Workspace {
        Workspace {
            conn,
            mode: AccessMode::ReadWrite,
            path,
            lock: Some(lock),
        }
    }

    fn backup_secondary(&self, now: DateTime<Local>) -> Result<SecondaryResult> {
        let Some(dir) = settings::get(&self.conn, SECONDARY_BACKUP_DIR)? else {
            return Ok(SecondaryResult::NotConfigured);
        };
        let dir = PathBuf::from(dir);
        let stem = db_stem(&self.path);
        let attempt = backup_to_dir(&self.conn, &dir, &stem, now)
            .and_then(|path| prune(&dir, &stem, BACKUP_KEEP).map(|_| path));
        match attempt {
            Ok(path) => {
                settings::set(
                    &self.conn,
                    LAST_SECONDARY_BACKUP,
                    &now.with_timezone(&Utc).to_rfc3339(),
                )?;
                Ok(SecondaryResult::Done(path))
            }
            Err(e) => Ok(SecondaryResult::Unavailable(format!(
                "Zweiter Sicherungsort nicht erreichbar: {e}"
            ))),
        }
    }

    fn open_read_write(db_path: &Path, lock: LockGuard) -> Result<Workspace> {
        let existed = db_path.exists();
        let conn = db::open_unmigrated(db_path, false)?;
        if existed {
            let dir = local_backup_dir(db_path);
            let stem = db_stem(db_path);
            backup_to_dir(&conn, &dir, &stem, Local::now())?;
            prune(&dir, &stem, BACKUP_KEEP)?;
        }
        db::migrate(&conn)?;
        Ok(Self::from_parts(conn, db_path.to_path_buf(), lock))
    }

    fn open_read_only(db_path: &Path, held_by: LockInfo) -> Result<Workspace> {
        let conn = db::open_connection(db_path, true)?;
        Ok(Workspace {
            conn,
            mode: AccessMode::ReadOnly { held_by },
            path: db_path.to_path_buf(),
            lock: None,
        })
    }
}

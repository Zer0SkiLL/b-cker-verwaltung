use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;

use super::backup::{backup_to_dir, db_stem, local_backup_dir};
use super::lock::{current_holder, Identity, LockGuard, LockInfo};
use super::retention::{apply_retention, load_policy};
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
    /// Änderungszähler der Verbindung nach der letzten Sicherung (für „sichern, falls geändert“).
    changes_at_backup: u64,
}

impl Workspace {
    /// Öffnet schreibend. Ist die Datei gesperrt, wird sie nur lesend geöffnet.
    pub fn open(db_path: &Path, who: &Identity) -> Result<Workspace> {
        let ws = match LockGuard::acquire(db_path, who) {
            Ok(lock) => Self::open_read_write(db_path, lock),
            Err(CoreError::LockHeld(info)) if db_path.exists() => {
                Self::open_read_only(db_path, info)
            }
            Err(err) => Err(err),
        }?;
        if ws.is_read_write() {
            log::info!("Datei geöffnet (schreibend)");
        } else {
            log::info!("Datei geöffnet (nur lesend)");
        }
        Ok(ws)
    }

    /// Übernimmt eine fremde Sperre und öffnet schreibend.
    pub fn open_taking_over(db_path: &Path, who: &Identity) -> Result<Workspace> {
        let lock = LockGuard::take_over(db_path, who)?;
        log::info!("Sperre übernommen");
        Self::open_read_write(db_path, lock)
    }

    /// Öffnet ausdrücklich nur lesend, ohne die Sperre anzufassen (Rückfall nach Fehlern).
    pub fn open_read_only_at(db_path: &Path) -> Result<Workspace> {
        let held_by = current_holder(db_path).unwrap_or_else(|| LockInfo {
            user: "unbekannt".into(),
            host: "unbekannt".into(),
            created_at: Utc::now(),
        });
        Self::open_read_only(db_path, held_by)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_read_write(&self) -> bool {
        matches!(self.mode, AccessMode::ReadWrite)
    }

    /// Sichert lokal und (falls eingerichtet) am zweiten Ort. Nur bei schreibendem Zugriff.
    /// Danach gilt der Stand als gesichert (keine weitere Sicherung während der Arbeit nötig).
    pub fn backup_now(&mut self, now: DateTime<Local>) -> Result<BackupOutcome> {
        self.require_read_write()?;
        let dir = local_backup_dir(&self.path);
        let stem = db_stem(&self.path);
        let local = backup_to_dir(&self.conn, &dir, &stem, now)?;
        log::info!("Sicherung neben der Datei erstellt");
        apply_retention(&dir, &stem, &local, &load_policy(&self.conn))?;
        let secondary = self.backup_secondary(now)?;
        self.mark_backed_up();
        Ok(BackupOutcome { local, secondary })
    }

    /// Nur am zweiten Ort sichern (z.B. nach dem Übernehmen: die lokale Sicherung entstand
    /// schon beim Öffnen).
    pub fn backup_secondary_now(&mut self, now: DateTime<Local>) -> Result<SecondaryResult> {
        self.require_read_write()?;
        let result = self.backup_secondary(now)?;
        self.mark_backed_up();
        Ok(result)
    }

    /// Merkt sich den aktuellen Änderungszähler als „gesichert“. Wird nach der Sicherung
    /// gelesen, weil die Sicherung selbst den Zeitpunkt am zweiten Ort speichert.
    pub(crate) fn mark_backed_up(&mut self) {
        self.changes_at_backup = self.conn.total_changes();
    }

    /// Sichert lokal und am zweiten Ort, aber nur wenn seit der letzten Sicherung etwas geändert
    /// wurde und schreibend geöffnet ist.
    pub fn backup_if_changed(&mut self, now: DateTime<Local>) -> Result<Option<BackupOutcome>> {
        if self.check_lock()? != &AccessMode::ReadWrite
            || self.conn.total_changes() == self.changes_at_backup
        {
            return Ok(None);
        }
        self.backup_now(now).map(Some)
    }

    /// Prüft, ob die eigene Sperre noch besteht. Wurde sie übernommen (z.B. durch die
    /// Ferienvertretung), wird die Datenbank nur noch lesend geöffnet. Vor dem Schreiben aufrufen.
    pub fn check_lock(&mut self) -> Result<&AccessMode> {
        let lost = self.lock.as_ref().is_some_and(|lock| !lock.still_held());
        if lost {
            log::warn!("Sperre verloren: Datei ist jetzt nur lesend geöffnet");
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
        let result = if self.is_read_write() {
            self.backup_secondary(now)
        } else {
            Ok(SecondaryResult::NotConfigured)
        };
        log::info!("Datei geschlossen");
        result
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
        let changes_at_backup = conn.total_changes();
        Workspace {
            conn,
            mode: AccessMode::ReadWrite,
            path,
            lock: Some(lock),
            changes_at_backup,
        }
    }

    fn backup_secondary(&self, now: DateTime<Local>) -> Result<SecondaryResult> {
        let Some(dir) = settings::get(&self.conn, SECONDARY_BACKUP_DIR)? else {
            return Ok(SecondaryResult::NotConfigured);
        };
        let dir = PathBuf::from(dir);
        let stem = db_stem(&self.path);
        let attempt = backup_to_dir(&self.conn, &dir, &stem, now).and_then(|path| {
            apply_retention(&dir, &stem, &path, &load_policy(&self.conn)).map(|_| path)
        });
        match attempt {
            Ok(path) => {
                settings::set(
                    &self.conn,
                    LAST_SECONDARY_BACKUP,
                    &now.with_timezone(&Utc).to_rfc3339(),
                )?;
                log::info!("Sicherung am zweiten Ort erstellt");
                Ok(SecondaryResult::Done(path))
            }
            Err(e) => {
                log::warn!("Zweiter Sicherungsort nicht erreichbar: {e}");
                Ok(SecondaryResult::Unavailable(format!(
                    "Zweiter Sicherungsort nicht erreichbar: {e}"
                )))
            }
        }
    }

    fn open_read_write(db_path: &Path, lock: LockGuard) -> Result<Workspace> {
        let existed = db_path.exists();
        let conn = db::open_unmigrated(db_path, false)?;
        if existed {
            let dir = local_backup_dir(db_path);
            let stem = db_stem(db_path);
            let saved = backup_to_dir(&conn, &dir, &stem, Local::now())?;
            // Vor der Migration: fehlt `settings` noch, gilt der Standard.
            apply_retention(&dir, &stem, &saved, &load_policy(&conn))?;
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
            changes_at_backup: 0,
        })
    }
}

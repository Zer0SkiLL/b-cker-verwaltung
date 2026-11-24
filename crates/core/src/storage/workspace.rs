use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::Connection;

use super::backup::{backup_to_dir, db_stem, local_backup_dir, prune, BACKUP_KEEP};
use super::lock::{Identity, LockGuard, LockInfo};
use crate::db;
use crate::error::{CoreError, Result};

/// Zugriffsart auf die geöffnete Datenbank.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessMode {
    ReadWrite,
    ReadOnly { held_by: LockInfo },
}

/// Geöffnete Datenbank inklusive Sperre. Felder werden in Deklarationsreihenfolge freigegeben:
/// zuerst die Verbindung, dann die Sperre.
pub struct Workspace {
    pub conn: Connection,
    pub mode: AccessMode,
    path: PathBuf,
    #[allow(dead_code)] // hält die Sperre, bis der Workspace freigegeben wird
    lock: Option<LockGuard>,
}

impl Workspace {
    /// Öffnet schreibend. Ist die Datei gesperrt, wird sie nur lesend geöffnet.
    pub fn open(db_path: &Path, who: &Identity) -> Result<Workspace> {
        match LockGuard::acquire(db_path, who) {
            Ok(lock) => Self::open_read_write(db_path, lock),
            Err(CoreError::LockHeld(info)) if db_path.exists() => Self::open_read_only(db_path, info),
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

    pub(crate) fn from_parts(conn: Connection, path: PathBuf, lock: LockGuard) -> Workspace {
        Workspace { conn, mode: AccessMode::ReadWrite, path, lock: Some(lock) }
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

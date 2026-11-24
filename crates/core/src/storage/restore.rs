//! Wiederherstellen einer Sicherung: prüfen, aktuellen Stand sichern, dann ersetzen.
//! Schlägt etwas fehl, bleibt die bisherige Datei geöffnet und die Sperre gehalten.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use rusqlite::Connection;

use super::backup::{backup_to_dir, db_stem, local_backup_dir, prune, verify, BACKUP_KEEP};
use super::workspace::Workspace;
use crate::db::{self, SCHEMA_VERSION};
use crate::error::{CoreError, Result};

/// Tabelle, an der eine Sicherung der Mitgliederverwaltung erkannt wird.
const MARKER_TABLE: &str = "organisations";

impl Workspace {
    /// Ersetzt die Datenbank durch `backup`. Die Sperre bleibt während des ganzen Vorgangs gehalten.
    pub fn restore(&mut self, backup: &Path, now: DateTime<Local>) -> Result<()> {
        self.require_read_write()?;
        validate(backup)?;
        let path = self.path().to_path_buf();
        let stem = db_stem(&path);
        let dir = local_backup_dir(&path);
        backup_to_dir(&self.conn, &dir, &stem, now)?;
        prune(&dir, &stem, BACKUP_KEEP)?;
        // Verbindung schliessen, damit die Datei ersetzt werden kann.
        drop(std::mem::replace(
            &mut self.conn,
            Connection::open_in_memory()?,
        ));
        match replace_file(backup, &path) {
            Ok(()) => {
                let conn = db::open_unmigrated(&path, false)?;
                db::migrate(&conn)?;
                self.conn = conn;
                Ok(())
            }
            Err(err) => {
                self.conn = db::open_connection(&path, false)?;
                Err(err)
            }
        }
    }
}

fn validate(backup: &Path) -> Result<()> {
    if verify(backup).is_err() {
        return Err(CoreError::Validation("Sicherung ist beschädigt".into()));
    }
    let conn = db::open_unmigrated(backup, true)?;
    if db::schema_version(&conn)? > SCHEMA_VERSION {
        return Err(CoreError::Validation(
            "Sicherung stammt von einer neueren Programmversion".into(),
        ));
    }
    let has_marker: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [MARKER_TABLE],
        |r| r.get(0),
    )?;
    if has_marker == 0 {
        return Err(CoreError::Validation(
            "Die Datei ist keine Sicherung der Mitgliederverwaltung".into(),
        ));
    }
    Ok(())
}

/// Kopiert in eine temporäre Datei (auf Platte geschrieben) und benennt sie dann um.
fn replace_file(backup: &Path, target: &Path) -> Result<()> {
    let mut tmp = target.as_os_str().to_os_string();
    tmp.push(".restore-tmp");
    let tmp = PathBuf::from(tmp);
    let result = fs::copy(backup, &tmp)
        .and_then(|_| File::open(&tmp)?.sync_all())
        .and_then(|_| fs::rename(&tmp, target));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    Ok(result?)
}

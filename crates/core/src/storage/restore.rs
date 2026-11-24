//! Wiederherstellen einer Sicherung: prüfen, aktuellen Stand sichern, dann ersetzen.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

use super::backup::{backup_to_dir, db_stem, local_backup_dir, prune, verify, BACKUP_KEEP};
use super::workspace::Workspace;
use crate::db::{self, SCHEMA_VERSION};
use crate::error::{CoreError, Result};

impl Workspace {
    /// Ersetzt die Datenbank durch `backup`. Die Sperre bleibt während des ganzen Vorgangs gehalten.
    pub fn restore(self, backup: &Path, now: DateTime<Local>) -> Result<Workspace> {
        self.require_read_write()?;
        validate(backup)?;
        let dir = local_backup_dir(self.path());
        let stem = db_stem(self.path());
        backup_to_dir(&self.conn, &dir, &stem, now)?;
        prune(&dir, &stem, BACKUP_KEEP)?;
        let (conn, path, lock) = self.into_parts();
        drop(conn);
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".restore-tmp");
        fs::copy(backup, &tmp)?;
        fs::rename(&tmp, &path)?;
        let conn = db::open_unmigrated(&path, false)?;
        db::migrate(&conn)?;
        let lock = lock.ok_or(CoreError::ReadOnly)?;
        Ok(Workspace::from_parts(conn, path, lock))
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
    Ok(())
}

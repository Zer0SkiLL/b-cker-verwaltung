use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use rusqlite::Connection;

use crate::db;
use crate::error::{CoreError, Result};

/// Anzahl Sicherungen, die pro Ort behalten werden.
pub const BACKUP_KEEP: usize = 20;
const BACKUP_EXT: &str = ".sqlite";

/// Ordner für automatische Sicherungen: `<Ordner der Datenbank>/backups`.
pub fn local_backup_dir(db_path: &Path) -> PathBuf {
    db_path.parent().unwrap_or_else(|| Path::new(".")).join("backups")
}

/// Dateistamm der Datenbank (z.B. `daten` für `daten.sqlite`), Basis für Sicherungsnamen.
pub fn db_stem(db_path: &Path) -> String {
    db_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "daten".into())
}

/// Erstellt eine konsistente, geprüfte Kopie mit `VACUUM INTO`.
pub fn backup_to_dir(conn: &Connection, dir: &Path, stem: &str, now: DateTime<Local>) -> Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let target = free_name(dir, stem, now);
    let target_str = target.to_string_lossy().into_owned();
    conn.execute("VACUUM INTO ?1", [&target_str])?;
    if let Err(problem) = verify(&target) {
        let _ = fs::remove_file(&target);
        return Err(CoreError::Validation(format!("Sicherung fehlerhaft: {problem}")));
    }
    Ok(target)
}

/// Löscht die ältesten `<stem>-*.sqlite` über `keep` hinaus. Fremde Dateien bleiben unberührt.
pub fn prune(dir: &Path, stem: &str, keep: usize) -> Result<()> {
    let mut names = backup_names(dir, stem)?;
    names.sort();
    let excess = names.len().saturating_sub(keep);
    for name in names.into_iter().take(excess) {
        fs::remove_file(dir.join(name))?;
    }
    Ok(())
}

/// Dateinamen aller Sicherungen von `stem` im Ordner (unsortiert).
pub fn backup_names(dir: &Path, stem: &str) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let prefix = format!("{stem}-");
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if name.starts_with(&prefix) && name.ends_with(BACKUP_EXT) {
            names.push(name);
        }
    }
    Ok(names)
}

/// Prüft eine Datenbankdatei mit `PRAGMA integrity_check`.
pub fn verify(path: &Path) -> std::result::Result<(), String> {
    let conn = db::open_unmigrated(path, true).map_err(|e| e.to_string())?;
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if result == "ok" {
        Ok(())
    } else {
        Err(result)
    }
}

fn free_name(dir: &Path, stem: &str, now: DateTime<Local>) -> PathBuf {
    let base = format!("{stem}-{}", now.format("%Y%m%d-%H%M%S"));
    let first = dir.join(format!("{base}{BACKUP_EXT}"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{base}-{n}{BACKUP_EXT}")))
        .find(|p| !p.exists())
        .expect("unendlicher Iterator")
}

//! Einstellungen, die mit der Datenbank reisen (gelten auch für die Ferienvertretung).

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;

/// Ordner des zweiten Sicherungsorts.
pub const SECONDARY_BACKUP_DIR: &str = "backup.secondary_dir";
/// Zeitpunkt der letzten Sicherung am zweiten Ort (RFC 3339).
pub const LAST_SECONDARY_BACKUP: &str = "backup.last_secondary_at";

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })
        .optional()?)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn remove(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
    Ok(())
}

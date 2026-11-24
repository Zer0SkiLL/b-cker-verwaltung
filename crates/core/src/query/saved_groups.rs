use rusqlite::{params, Connection, OptionalExtension};

use super::filter::Filter;
use crate::error::{CoreError, Result};
use crate::repo::require_name;

/// Speichert einen Filter unter einem Namen; ein bestehender Name wird überschrieben.
pub fn save(conn: &Connection, name: &str, filter: &Filter) -> Result<i64> {
    let name = require_name(name)?;
    let json = serde_json::to_string(filter).map_err(|e| CoreError::Validation(e.to_string()))?;
    conn.execute(
        "INSERT INTO saved_groups(name, filter_json) VALUES (?1, ?2) \
         ON CONFLICT(name) DO UPDATE SET filter_json = excluded.filter_json",
        params![name, json],
    )?;
    Ok(conn.query_row("SELECT id FROM saved_groups WHERE name = ?1", [&name], |r| r.get(0))?)
}

pub fn list(conn: &Connection) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, name FROM saved_groups ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn load(conn: &Connection, id: i64) -> Result<Filter> {
    let json: String = conn
        .query_row("SELECT filter_json FROM saved_groups WHERE id = ?1", [id], |r| r.get(0))
        .optional()?
        .ok_or(CoreError::NotFound { entity: "Gespeicherte Gruppe", id })?;
    serde_json::from_str(&json)
        .map_err(|e| CoreError::Validation(format!("Gespeicherte Gruppe ist ungültig: {e}")))
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM saved_groups WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: "Gespeicherte Gruppe", id });
    }
    Ok(())
}

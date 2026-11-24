use rusqlite::{params, Connection, OptionalExtension, Row};

use super::lookups::invalid_text;
use super::{guard_in_use, require_name};
use crate::error::{CoreError, Result};
use crate::model::{AppliesTo, Label, LabelGroup};

pub fn create_group(conn: &Connection, name: &str, applies_to: AppliesTo) -> Result<LabelGroup> {
    let name = require_name(name)?;
    conn.execute(
        "INSERT INTO label_groups(name, applies_to) VALUES (?1, ?2)",
        params![name, applies_to.as_str()],
    )?;
    get_group(conn, conn.last_insert_rowid())
}

pub fn get_group(conn: &Connection, id: i64) -> Result<LabelGroup> {
    conn.query_row("SELECT id, name, applies_to FROM label_groups WHERE id = ?1", [id], group_from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: "Label-Gruppe", id })
}

pub fn list_groups(conn: &Connection) -> Result<Vec<LabelGroup>> {
    let mut stmt =
        conn.prepare("SELECT id, name, applies_to FROM label_groups ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], group_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_label(conn: &Connection, group_id: i64, name: &str) -> Result<Label> {
    let name = require_name(name)?;
    conn.execute("INSERT INTO labels(group_id, name) VALUES (?1, ?2)", params![group_id, name])?;
    get_label(conn, conn.last_insert_rowid())
}

pub fn get_label(conn: &Connection, id: i64) -> Result<Label> {
    conn.query_row("SELECT id, group_id, name FROM labels WHERE id = ?1", [id], label_from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: "Label", id })
}

pub fn list_labels(conn: &Connection, group_id: i64) -> Result<Vec<Label>> {
    let mut stmt = conn.prepare(
        "SELECT id, group_id, name FROM labels WHERE group_id = ?1 ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([group_id], label_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn rename_label(conn: &Connection, id: i64, name: &str) -> Result<Label> {
    let name = require_name(name)?;
    let changed = conn.execute("UPDATE labels SET name = ?1 WHERE id = ?2", params![name, id])?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: "Label", id });
    }
    get_label(conn, id)
}

/// Sucht ein Label nach Gruppen- und Labelname; legt Gruppe und Label bei Bedarf an.
pub fn find_or_create_label(
    conn: &Connection,
    group_name: &str,
    applies_to: AppliesTo,
    label_name: &str,
) -> Result<Label> {
    let group_name = require_name(group_name)?;
    let label_name = require_name(label_name)?;
    let group_id: Option<i64> = conn
        .query_row("SELECT id FROM label_groups WHERE name = ?1", [&group_name], |r| r.get(0))
        .optional()?;
    let group_id = match group_id {
        Some(id) => id,
        None => create_group(conn, &group_name, applies_to)?.id,
    };
    let existing = conn
        .query_row(
            "SELECT id, group_id, name FROM labels WHERE group_id = ?1 AND name = ?2",
            params![group_id, label_name],
            label_from_row,
        )
        .optional()?;
    match existing {
        Some(label) => Ok(label),
        None => create_label(conn, group_id, &label_name),
    }
}

pub fn label_usage_count(conn: &Connection, id: i64) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM label_assignments WHERE label_id = ?1", [id], |r| r.get(0))?)
}

pub fn delete_label(conn: &Connection, id: i64, confirm: bool) -> Result<()> {
    get_label(conn, id)?;
    guard_in_use(label_usage_count(conn, id)?, confirm)?;
    conn.execute("DELETE FROM labels WHERE id = ?1", [id])?;
    Ok(())
}

pub fn delete_group(conn: &Connection, id: i64, confirm: bool) -> Result<()> {
    get_group(conn, id)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM label_assignments a JOIN labels l ON l.id = a.label_id WHERE l.group_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    guard_in_use(count, confirm)?;
    conn.execute("DELETE FROM label_groups WHERE id = ?1", [id])?;
    Ok(())
}

fn group_from_row(row: &Row) -> rusqlite::Result<LabelGroup> {
    let applies: String = row.get(2)?;
    Ok(LabelGroup {
        id: row.get(0)?,
        name: row.get(1)?,
        applies_to: AppliesTo::parse(&applies).ok_or_else(|| invalid_text(2, applies))?,
    })
}

fn label_from_row(row: &Row) -> rusqlite::Result<Label> {
    Ok(Label { id: row.get(0)?, group_id: row.get(1)?, name: row.get(2)? })
}

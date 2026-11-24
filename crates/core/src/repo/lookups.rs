use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{guard_in_use, require_name};
use crate::error::{CoreError, Result};
use crate::model::{LookupKind, LookupValue};

const ENTITY: &str = "Listeneintrag";

/// Spalten, die auf optionale Listeneinträge verweisen (werden beim Löschen geleert).
const NULLABLE_REFS: &[(&str, &str)] = &[
    ("organisations", "org_type_id"),
    ("organisations", "cantonal_association_id"),
    ("memberships", "category_id"),
];

pub fn create(conn: &Connection, kind: LookupKind, name: &str) -> Result<LookupValue> {
    let name = require_name(name)?;
    conn.execute(
        "INSERT INTO lookup_values(kind, name) VALUES (?1, ?2)",
        params![kind.as_str(), name],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn get(conn: &Connection, id: i64) -> Result<LookupValue> {
    conn.query_row("SELECT id, kind, name, sort FROM lookup_values WHERE id = ?1", [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn list(conn: &Connection, kind: LookupKind) -> Result<Vec<LookupValue>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, name, sort FROM lookup_values WHERE kind = ?1 ORDER BY sort, name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([kind.as_str()], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<LookupValue> {
    let name = require_name(name)?;
    let changed = conn.execute("UPDATE lookup_values SET name = ?1 WHERE id = ?2", params![name, id])?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn find_or_create(conn: &Connection, kind: LookupKind, name: &str) -> Result<LookupValue> {
    let name = require_name(name)?;
    let existing = conn
        .query_row(
            "SELECT id, kind, name, sort FROM lookup_values WHERE kind = ?1 AND name = ?2",
            params![kind.as_str(), name],
            from_row,
        )
        .optional()?;
    match existing {
        Some(value) => Ok(value),
        None => create(conn, kind, &name),
    }
}

/// Anzahl Verweise auf diesen Listeneintrag.
pub fn usage_count(conn: &Connection, id: i64) -> Result<i64> {
    let mut total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM affiliations WHERE function_id = ?1",
        [id],
        |r| r.get(0),
    )?;
    for (table, column) in NULLABLE_REFS {
        let sql = format!("SELECT COUNT(*) FROM {table} WHERE {column} = ?1");
        total += conn.query_row(&sql, [id], |r| r.get::<_, i64>(0))?;
    }
    Ok(total)
}

/// Löscht einen Listeneintrag. Verwendete Einträge nur mit `confirm`; Funktionen in Gebrauch nie.
pub fn delete(conn: &Connection, id: i64, confirm: bool) -> Result<()> {
    let value = get(conn, id)?;
    let count = usage_count(conn, id)?;
    if value.kind == LookupKind::Function && count > 0 {
        return Err(CoreError::Validation("Funktion wird noch verwendet".into()));
    }
    guard_in_use(count, confirm)?;
    let tx = conn.unchecked_transaction()?;
    for (table, column) in NULLABLE_REFS {
        let sql = format!("UPDATE {table} SET {column} = NULL WHERE {column} = ?1");
        tx.execute(&sql, [id])?;
    }
    tx.execute("DELETE FROM lookup_values WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

fn from_row(row: &Row) -> rusqlite::Result<LookupValue> {
    let kind: String = row.get(1)?;
    Ok(LookupValue {
        id: row.get(0)?,
        kind: LookupKind::parse(&kind).ok_or_else(|| invalid_text(1, kind))?,
        name: row.get(2)?,
        sort: row.get(3)?,
    })
}

pub(crate) fn invalid_text(col: usize, value: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        col,
        rusqlite::types::Type::Text,
        format!("unbekannter Wert: {value}").into(),
    )
}

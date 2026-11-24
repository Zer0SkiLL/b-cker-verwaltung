use chrono::NaiveDate;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::labels;
use crate::error::{CoreError, Result};
use crate::model::{AppliesTo, MemberRef};
use crate::text::clean;

const COLUMNS: &str = "id, label_id, organisation_id, person_id, assigned_on, note";

/// Zuweisung eines Labels an Betrieb oder Person, optional mit Datum und Notiz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentInput {
    pub label_id: i64,
    pub target: MemberRef,
    pub assigned_on: Option<NaiveDate>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub label_id: i64,
    pub target: MemberRef,
    pub assigned_on: Option<NaiveDate>,
    pub note: Option<String>,
}

pub fn assign(conn: &Connection, input: &AssignmentInput) -> Result<Assignment> {
    let label = labels::get_label(conn, input.label_id)?;
    let group = labels::get_group(conn, label.group_id)?;
    if !applies(group.applies_to, input.target) {
        return Err(CoreError::Validation("Label gilt nicht für diesen Eintragstyp".into()));
    }
    let (org, person) = input.target.columns();
    conn.execute(
        "INSERT INTO label_assignments(label_id, organisation_id, person_id, assigned_on, note) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![input.label_id, org, person, input.assigned_on, clean(&input.note)],
    )?;
    let id = conn.last_insert_rowid();
    let sql = format!("SELECT {COLUMNS} FROM label_assignments WHERE id = ?1");
    Ok(conn.query_row(&sql, [id], from_row)?)
}

pub fn unassign(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM label_assignments WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: "Label-Zuweisung", id });
    }
    Ok(())
}

pub fn for_target(conn: &Connection, target: &MemberRef) -> Result<Vec<Assignment>> {
    let (column, id) = match *target {
        MemberRef::Organisation(id) => ("organisation_id", id),
        MemberRef::Person(id) => ("person_id", id),
    };
    let sql = format!("SELECT {COLUMNS} FROM label_assignments WHERE {column} = ?1 ORDER BY id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([id], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn applies(applies_to: AppliesTo, target: MemberRef) -> bool {
    matches!(
        (applies_to, target),
        (AppliesTo::Both, _)
            | (AppliesTo::Person, MemberRef::Person(_))
            | (AppliesTo::Organisation, MemberRef::Organisation(_))
    )
}

fn from_row(row: &Row) -> rusqlite::Result<Assignment> {
    let target = MemberRef::from_columns(row.get(2)?, row.get(3)?).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Null, "Ziel fehlt".into())
    })?;
    Ok(Assignment {
        id: row.get(0)?,
        label_id: row.get(1)?,
        target,
        assigned_on: row.get(4)?,
        note: row.get(5)?,
    })
}

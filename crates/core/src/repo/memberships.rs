use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::check_date_order;
use crate::error::{CoreError, Result};
use crate::model::MemberRef;

const ENTITY: &str = "Mitgliedschaft";
const COLUMNS: &str = "id, organisation_id, person_id, category_id, joined_on, left_on";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MembershipInput {
    pub member: MemberRef,
    pub category_id: Option<i64>,
    pub joined_on: Option<NaiveDate>,
    pub left_on: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Membership {
    pub id: i64,
    pub member: MemberRef,
    pub category_id: Option<i64>,
    pub joined_on: Option<NaiveDate>,
    pub left_on: Option<NaiveDate>,
}

impl Membership {
    /// Aktiv, solange kein Austritt erfasst ist oder der Austritt in der Zukunft liegt.
    pub fn is_active(&self, today: NaiveDate) -> bool {
        self.left_on.is_none_or(|left| left > today)
    }
}

pub fn create(conn: &Connection, input: &MembershipInput) -> Result<Membership> {
    validate(input)?;
    let (org, person) = input.member.columns();
    conn.execute(
        "INSERT INTO memberships(organisation_id, person_id, category_id, joined_on, left_on) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            org,
            person,
            input.category_id,
            input.joined_on,
            input.left_on
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &MembershipInput) -> Result<Membership> {
    validate(input)?;
    let (org, person) = input.member.columns();
    let changed = conn.execute(
        "UPDATE memberships SET organisation_id = ?1, person_id = ?2, category_id = ?3, joined_on = ?4, \
         left_on = ?5 WHERE id = ?6",
        params![org, person, input.category_id, input.joined_on, input.left_on, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn get(conn: &Connection, id: i64) -> Result<Membership> {
    let sql = format!("SELECT {COLUMNS} FROM memberships WHERE id = ?1");
    conn.query_row(&sql, [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM memberships WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    Ok(())
}

pub fn for_member(conn: &Connection, member: &MemberRef) -> Result<Vec<Membership>> {
    let (column, id) = match *member {
        MemberRef::Organisation(id) => ("organisation_id", id),
        MemberRef::Person(id) => ("person_id", id),
    };
    let sql = format!("SELECT {COLUMNS} FROM memberships WHERE {column} = ?1 ORDER BY id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([id], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn validate(input: &MembershipInput) -> Result<()> {
    check_date_order(
        input.joined_on,
        input.left_on,
        "Austritt liegt vor Eintritt",
    )
}

fn from_row(row: &Row) -> rusqlite::Result<Membership> {
    let member = MemberRef::from_columns(row.get(1)?, row.get(2)?).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Null,
            "Mitglied fehlt".into(),
        )
    })?;
    Ok(Membership {
        id: row.get(0)?,
        member,
        category_id: row.get(3)?,
        joined_on: row.get(4)?,
        left_on: row.get(5)?,
    })
}

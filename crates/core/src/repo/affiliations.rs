use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::check_date_order;
use crate::error::{CoreError, Result};

const ENTITY: &str = "Zugehörigkeit";
const COLUMNS: &str = "id, person_id, organisation_id, function_id, valid_from, valid_to";
/// SQL-Bedingung "aktuell": kein Bis-Datum oder Bis-Datum ab heute (Parameter :today).
pub(crate) const CURRENT_SQL: &str = "(valid_to IS NULL OR valid_to >= :today)";

/// Verbindung Person ↔ Betrieb mit Funktion und Gültigkeit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffiliationInput {
    pub person_id: i64,
    pub organisation_id: i64,
    pub function_id: i64,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Affiliation {
    pub id: i64,
    pub person_id: i64,
    pub organisation_id: i64,
    pub function_id: i64,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}

impl Affiliation {
    pub fn is_current(&self, today: NaiveDate) -> bool {
        self.valid_to.is_none_or(|to| to >= today)
    }
}

pub fn create(conn: &Connection, input: &AffiliationInput) -> Result<Affiliation> {
    validate(input)?;
    conn.execute(
        "INSERT INTO affiliations(person_id, organisation_id, function_id, valid_from, valid_to) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.person_id,
            input.organisation_id,
            input.function_id,
            input.valid_from,
            input.valid_to
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &AffiliationInput) -> Result<Affiliation> {
    validate(input)?;
    let changed = conn.execute(
        "UPDATE affiliations SET person_id = ?1, organisation_id = ?2, function_id = ?3, valid_from = ?4, \
         valid_to = ?5 WHERE id = ?6",
        params![input.person_id, input.organisation_id, input.function_id, input.valid_from, input.valid_to, id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn get(conn: &Connection, id: i64) -> Result<Affiliation> {
    let sql = format!("SELECT {COLUMNS} FROM affiliations WHERE id = ?1");
    conn.query_row(&sql, [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM affiliations WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    Ok(())
}

pub fn for_person(
    conn: &Connection,
    person_id: i64,
    include_past: bool,
    today: NaiveDate,
) -> Result<Vec<Affiliation>> {
    list_where(conn, "person_id", person_id, include_past, today)
}

pub fn for_organisation(
    conn: &Connection,
    org_id: i64,
    include_past: bool,
    today: NaiveDate,
) -> Result<Vec<Affiliation>> {
    list_where(conn, "organisation_id", org_id, include_past, today)
}

fn list_where(
    conn: &Connection,
    column: &str,
    id: i64,
    include_past: bool,
    today: NaiveDate,
) -> Result<Vec<Affiliation>> {
    let base = format!("SELECT {COLUMNS} FROM affiliations WHERE {column} = :id");
    let rows = if include_past {
        let mut stmt = conn.prepare(&format!("{base} ORDER BY id"))?;
        let rows = stmt.query_map(rusqlite::named_params! { ":id": id }, from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let mut stmt = conn.prepare(&format!("{base} AND {CURRENT_SQL} ORDER BY id"))?;
        let rows = stmt.query_map(
            rusqlite::named_params! { ":id": id, ":today": today },
            from_row,
        )?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    Ok(rows)
}

fn validate(input: &AffiliationInput) -> Result<()> {
    check_date_order(
        input.valid_from,
        input.valid_to,
        "Bis-Datum liegt vor Von-Datum",
    )
}

fn from_row(row: &Row) -> rusqlite::Result<Affiliation> {
    Ok(Affiliation {
        id: row.get(0)?,
        person_id: row.get(1)?,
        organisation_id: row.get(2)?,
        function_id: row.get(3)?,
        valid_from: row.get(4)?,
        valid_to: row.get(5)?,
    })
}

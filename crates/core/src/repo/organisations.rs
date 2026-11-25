use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::require_name;
use crate::error::{CoreError, Result};
use crate::text::{clean, normalize_search};

const ENTITY: &str = "Betrieb";
const COLUMNS: &str =
    "id, name, addition, street, po_box, zip, city, phone, email, website, branches, \
     apprentice_count, notes, legacy_id, org_type_id, cantonal_association_id, \
     commercial_register_no, cost_center_no";

/// Eingabedaten für einen Betrieb (juristische Person).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrganisationInput {
    pub name: String,
    pub addition: Option<String>,
    pub street: Option<String>,
    pub po_box: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub branches: Option<i64>,
    pub apprentice_count: Option<i64>,
    pub notes: Option<String>,
    pub legacy_id: Option<String>,
    pub org_type_id: Option<i64>,
    pub cantonal_association_id: Option<i64>,
    pub commercial_register_no: Option<String>,
    pub cost_center_no: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Organisation {
    pub id: i64,
    pub name: String,
    pub addition: Option<String>,
    pub street: Option<String>,
    pub po_box: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub branches: Option<i64>,
    pub apprentice_count: Option<i64>,
    pub notes: Option<String>,
    pub legacy_id: Option<String>,
    pub org_type_id: Option<i64>,
    pub cantonal_association_id: Option<i64>,
    pub commercial_register_no: Option<String>,
    pub cost_center_no: Option<String>,
}

pub fn create(conn: &Connection, input: &OrganisationInput) -> Result<Organisation> {
    let c = cleaned(input)?;
    conn.execute(
        "INSERT INTO organisations(name, addition, street, po_box, zip, city, phone, email, website, \
         branches, apprentice_count, notes, legacy_id, org_type_id, cantonal_association_id, search_text, \
         commercial_register_no, cost_center_no) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        params![
            c.name, c.addition, c.street, c.po_box, c.zip, c.city, c.phone, c.email, c.website,
            c.branches, c.apprentice_count, c.notes, c.legacy_id, c.org_type_id,
            c.cantonal_association_id, search_text(&c), c.commercial_register_no, c.cost_center_no
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &OrganisationInput) -> Result<Organisation> {
    let c = cleaned(input)?;
    let changed = conn.execute(
        "UPDATE organisations SET name = ?1, addition = ?2, street = ?3, po_box = ?4, zip = ?5, city = ?6, \
         phone = ?7, email = ?8, website = ?9, branches = ?10, apprentice_count = ?11, notes = ?12, \
         legacy_id = ?13, org_type_id = ?14, cantonal_association_id = ?15, search_text = ?16, \
         commercial_register_no = ?17, cost_center_no = ?18 WHERE id = ?19",
        params![
            c.name, c.addition, c.street, c.po_box, c.zip, c.city, c.phone, c.email, c.website,
            c.branches, c.apprentice_count, c.notes, c.legacy_id, c.org_type_id,
            c.cantonal_association_id, search_text(&c), c.commercial_register_no, c.cost_center_no, id
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn get(conn: &Connection, id: i64) -> Result<Organisation> {
    let sql = format!("SELECT {COLUMNS} FROM organisations WHERE id = ?1");
    conn.query_row(&sql, [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn list(conn: &Connection) -> Result<Vec<Organisation>> {
    let sql = format!("SELECT {COLUMNS} FROM organisations ORDER BY name COLLATE NOCASE, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Löscht den Betrieb; Zugehörigkeiten, Mitgliedschaften und Labels fallen weg, Personen bleiben.
pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM organisations WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    Ok(())
}

fn cleaned(input: &OrganisationInput) -> Result<OrganisationInput> {
    Ok(OrganisationInput {
        name: require_name(&input.name)?,
        addition: clean(&input.addition),
        street: clean(&input.street),
        po_box: clean(&input.po_box),
        zip: clean(&input.zip),
        city: clean(&input.city),
        phone: clean(&input.phone),
        email: clean(&input.email),
        website: clean(&input.website),
        notes: clean(&input.notes),
        legacy_id: clean(&input.legacy_id),
        commercial_register_no: clean(&input.commercial_register_no),
        cost_center_no: clean(&input.cost_center_no),
        ..input.clone()
    })
}

fn search_text(c: &OrganisationInput) -> String {
    normalize_search(&[
        Some(&c.name),
        c.addition.as_deref(),
        c.city.as_deref(),
        c.email.as_deref(),
        c.commercial_register_no.as_deref(),
    ])
}

fn from_row(row: &Row) -> rusqlite::Result<Organisation> {
    Ok(Organisation {
        id: row.get(0)?,
        name: row.get(1)?,
        addition: row.get(2)?,
        street: row.get(3)?,
        po_box: row.get(4)?,
        zip: row.get(5)?,
        city: row.get(6)?,
        phone: row.get(7)?,
        email: row.get(8)?,
        website: row.get(9)?,
        branches: row.get(10)?,
        apprentice_count: row.get(11)?,
        notes: row.get(12)?,
        legacy_id: row.get(13)?,
        org_type_id: row.get(14)?,
        cantonal_association_id: row.get(15)?,
        commercial_register_no: row.get(16)?,
        cost_center_no: row.get(17)?,
    })
}

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::text::{clean, normalize_search};

const ENTITY: &str = "Person";
const COLUMNS: &str =
    "id, salutation, first_name, last_name, street, zip, city, phone, mobile, email, compensation, notes";

/// Eingabedaten für eine natürliche Person.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PersonInput {
    pub salutation: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    /// Honorar/Lohn als Freitext, z.B. "CHF 300 / Prüfungstag".
    pub compensation: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id: i64,
    pub salutation: Option<String>,
    pub first_name: Option<String>,
    pub last_name: String,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub compensation: Option<String>,
    pub notes: Option<String>,
}

impl Person {
    /// "Vorname Nachname" bzw. nur Nachname.
    pub fn display_name(&self) -> String {
        match &self.first_name {
            Some(first) => format!("{first} {}", self.last_name),
            None => self.last_name.clone(),
        }
    }
}

pub fn create(conn: &Connection, input: &PersonInput) -> Result<Person> {
    let c = cleaned(input)?;
    conn.execute(
        "INSERT INTO persons(salutation, first_name, last_name, street, zip, city, phone, mobile, email, \
         compensation, notes, search_text) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            c.salutation, c.first_name, c.last_name, c.street, c.zip, c.city, c.phone, c.mobile,
            c.email, c.compensation, c.notes, search_text(&c)
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, input: &PersonInput) -> Result<Person> {
    let c = cleaned(input)?;
    let changed = conn.execute(
        "UPDATE persons SET salutation = ?1, first_name = ?2, last_name = ?3, street = ?4, zip = ?5, \
         city = ?6, phone = ?7, mobile = ?8, email = ?9, compensation = ?10, notes = ?11, search_text = ?12 \
         WHERE id = ?13",
        params![
            c.salutation, c.first_name, c.last_name, c.street, c.zip, c.city, c.phone, c.mobile,
            c.email, c.compensation, c.notes, search_text(&c), id
        ],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn get(conn: &Connection, id: i64) -> Result<Person> {
    let sql = format!("SELECT {COLUMNS} FROM persons WHERE id = ?1");
    conn.query_row(&sql, [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn list(conn: &Connection) -> Result<Vec<Person>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM persons ORDER BY last_name COLLATE NOCASE, first_name COLLATE NOCASE, id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM persons WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    Ok(())
}

fn cleaned(input: &PersonInput) -> Result<PersonInput> {
    let last_name = input.last_name.trim();
    if last_name.is_empty() {
        return Err(CoreError::Validation(
            "Nachname darf nicht leer sein".into(),
        ));
    }
    Ok(PersonInput {
        salutation: clean(&input.salutation),
        first_name: clean(&input.first_name),
        last_name: last_name.to_string(),
        street: clean(&input.street),
        zip: clean(&input.zip),
        city: clean(&input.city),
        phone: clean(&input.phone),
        mobile: clean(&input.mobile),
        email: clean(&input.email),
        compensation: clean(&input.compensation),
        notes: clean(&input.notes),
    })
}

fn search_text(c: &PersonInput) -> String {
    normalize_search(&[
        c.first_name.as_deref(),
        Some(&c.last_name),
        c.city.as_deref(),
        c.email.as_deref(),
    ])
}

fn from_row(row: &Row) -> rusqlite::Result<Person> {
    Ok(Person {
        id: row.get(0)?,
        salutation: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        street: row.get(4)?,
        zip: row.get(5)?,
        city: row.get(6)?,
        phone: row.get(7)?,
        mobile: row.get(8)?,
        email: row.get(9)?,
        compensation: row.get(10)?,
        notes: row.get(11)?,
    })
}

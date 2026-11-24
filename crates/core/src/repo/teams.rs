use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use super::require_name;
use crate::error::{CoreError, Result};
use crate::text::clean;

const ENTITY: &str = "Team";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamMember {
    pub person_id: i64,
    pub role: Option<String>,
}

pub fn create(conn: &Connection, name: &str, description: Option<&str>) -> Result<Team> {
    let name = require_name(name)?;
    conn.execute(
        "INSERT INTO teams(name, description) VALUES (?1, ?2)",
        params![name, clean(&description.map(String::from))],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, name: &str, description: Option<&str>) -> Result<Team> {
    let name = require_name(name)?;
    let changed = conn.execute(
        "UPDATE teams SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, clean(&description.map(String::from)), id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    get(conn, id)
}

pub fn get(conn: &Connection, id: i64) -> Result<Team> {
    conn.query_row("SELECT id, name, description FROM teams WHERE id = ?1", [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { entity: ENTITY, id })
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    if conn.execute("DELETE FROM teams WHERE id = ?1", [id])? == 0 {
        return Err(CoreError::NotFound { entity: ENTITY, id });
    }
    Ok(())
}

pub fn list(conn: &Connection) -> Result<Vec<Team>> {
    let mut stmt = conn.prepare("SELECT id, name, description FROM teams ORDER BY name COLLATE NOCASE")?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Fügt eine Person hinzu; ist sie schon Mitglied, wird nur die Rolle aktualisiert.
pub fn add_member(conn: &Connection, team_id: i64, person_id: i64, role: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO team_members(team_id, person_id, role) VALUES (?1, ?2, ?3) \
         ON CONFLICT(team_id, person_id) DO UPDATE SET role = excluded.role",
        params![team_id, person_id, clean(&role.map(String::from))],
    )?;
    Ok(())
}

pub fn remove_member(conn: &Connection, team_id: i64, person_id: i64) -> Result<()> {
    conn.execute("DELETE FROM team_members WHERE team_id = ?1 AND person_id = ?2", [team_id, person_id])?;
    Ok(())
}

pub fn members(conn: &Connection, team_id: i64) -> Result<Vec<TeamMember>> {
    let mut stmt = conn.prepare("SELECT person_id, role FROM team_members WHERE team_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([team_id], |r| Ok(TeamMember { person_id: r.get(0)?, role: r.get(1)? }))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn from_row(row: &Row) -> rusqlite::Result<Team> {
    Ok(Team { id: row.get(0)?, name: row.get(1)?, description: row.get(2)? })
}

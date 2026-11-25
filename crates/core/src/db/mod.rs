use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::error::{CoreError, Result};

const MIGRATIONS: &[&str] = &[
    include_str!("0001_init.sql"),
    include_str!("0002_settings.sql"),
    include_str!("0003_org_numbers.sql"),
];

/// Aktuelle Schema-Version (Anzahl Migrationen).
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

/// Öffnet die Datenbank ohne Migration. Setzt Journal-Modus DELETE (nur schreibend) und Fremdschlüssel.
pub fn open_unmigrated(path: &Path, read_only: bool) -> Result<Connection> {
    let flags = if read_only {
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
    };
    let conn = Connection::open_with_flags(path, flags)?;
    configure(&conn, read_only)?;
    Ok(conn)
}

/// Öffnet die Datenbank und migriert sie (ausser schreibgeschützt).
pub fn open_connection(path: &Path, read_only: bool) -> Result<Connection> {
    let conn = open_unmigrated(path, read_only)?;
    if !read_only {
        migrate(&conn)?;
    }
    Ok(conn)
}

/// Migrierte In-Memory-Datenbank (für Tests).
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn schema_version(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

/// Wendet alle noch fehlenden Migrationen an, jede in einer eigenen Transaktion.
pub fn migrate(conn: &Connection) -> Result<()> {
    let current = schema_version(conn)?;
    if current > SCHEMA_VERSION {
        return Err(CoreError::Validation(
            "Die Datei stammt von einer neueren Programmversion. Bitte das Programm aktualisieren."
                .into(),
        ));
    }
    for (index, sql) in MIGRATIONS.iter().enumerate() {
        let version = index as i64 + 1;
        if version <= current {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

fn configure(conn: &Connection, read_only: bool) -> Result<()> {
    if !read_only {
        let _mode: String = conn.query_row("PRAGMA journal_mode = DELETE", [], |r| r.get(0))?;
    }
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

#![allow(dead_code)]
use rusqlite::Connection;

pub fn test_conn() -> Connection {
    verwaltung_core::db::open_in_memory().expect("in-memory db")
}

/// Fügt eine Organisation per SQL ein (für Tests vor bzw. unabhängig vom Repository).
pub fn raw_org(conn: &Connection, name: &str) -> i64 {
    conn.execute("INSERT INTO organisations(name, search_text) VALUES (?1, ?1)", [name]).unwrap();
    conn.last_insert_rowid()
}

/// Fügt eine Person per SQL ein.
pub fn raw_person(conn: &Connection, last_name: &str) -> i64 {
    conn.execute("INSERT INTO persons(last_name, search_text) VALUES (?1, ?1)", [last_name]).unwrap();
    conn.last_insert_rowid()
}

pub fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0)).unwrap()
}

pub fn d(y: i32, m: u32, day: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

pub fn function(conn: &Connection, name: &str) -> i64 {
    verwaltung_core::repo::lookups::find_or_create(conn, verwaltung_core::model::LookupKind::Function, name)
        .unwrap()
        .id
}

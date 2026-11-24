#![allow(dead_code)]
use rusqlite::Connection;

pub fn test_conn() -> Connection {
    verwaltung_core::db::open_in_memory().expect("in-memory db")
}

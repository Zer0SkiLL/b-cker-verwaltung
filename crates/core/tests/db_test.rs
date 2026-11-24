use verwaltung_core::db::{open_connection, schema_version, SCHEMA_VERSION};
use verwaltung_core::error::CoreError;

fn tmp_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("d.sqlite");
    (dir, path)
}

#[test]
fn creates_file_with_current_schema_version() {
    let (_d, path) = tmp_db();
    let conn = open_connection(&path, false).unwrap();
    assert!(path.exists());
    assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
}

#[test]
fn uses_delete_journal_and_foreign_keys() {
    let (_d, path) = tmp_db();
    let conn = open_connection(&path, false).unwrap();
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0)).unwrap();
    let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
    assert_eq!(mode, "delete");
    assert_eq!(fk, 1);
}

#[test]
fn migrate_is_idempotent() {
    let (_d, path) = tmp_db();
    drop(open_connection(&path, false).unwrap());
    let conn = open_connection(&path, false).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
}

#[test]
fn read_only_connection_rejects_writes() {
    let (_d, path) = tmp_db();
    drop(open_connection(&path, false).unwrap());
    let conn = open_connection(&path, true).unwrap();
    let err = conn
        .execute("INSERT INTO teams(name) VALUES ('A')", [])
        .map_err(CoreError::from)
        .unwrap_err();
    assert!(matches!(err, CoreError::ReadOnly), "got {err:?}");
}

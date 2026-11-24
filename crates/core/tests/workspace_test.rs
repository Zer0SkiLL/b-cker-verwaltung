use chrono::{Local, TimeZone};
use verwaltung_core::error::CoreError;
use verwaltung_core::storage::backup::{backup_to_dir, local_backup_dir, prune, BACKUP_KEEP};
use verwaltung_core::storage::lock::{Identity, LockGuard};
use verwaltung_core::storage::workspace::{AccessMode, Workspace};

fn who(user: &str) -> Identity {
    Identity {
        user: user.into(),
        host: "pc1".into(),
    }
}

fn backups(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn first_open_creates_db_without_backup() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let ws = Workspace::open(&db, &who("reto")).unwrap();
    assert!(matches!(ws.mode, AccessMode::ReadWrite));
    assert!(db.exists());
    assert!(backups(&local_backup_dir(&db)).is_empty());
}

#[test]
fn second_open_creates_backup() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    drop(Workspace::open(&db, &who("reto")).unwrap());
    drop(Workspace::open(&db, &who("reto")).unwrap());
    let names = backups(&local_backup_dir(&db));
    assert_eq!(names.len(), 1);
    assert!(names[0].starts_with("daten-") && names[0].ends_with(".sqlite"));
}

#[test]
fn backup_is_valid_sqlite_with_same_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let ws = Workspace::open(&db, &who("reto")).unwrap();
    ws.conn
        .execute("INSERT INTO teams(name) VALUES ('Prüfungen')", [])
        .unwrap();
    let out = backup_to_dir(&ws.conn, &dir.path().join("b"), "daten", Local::now()).unwrap();
    let copy = rusqlite::Connection::open(&out).unwrap();
    let name: String = copy
        .query_row("SELECT name FROM teams", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Prüfungen");
}

#[test]
fn backup_same_second_gets_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let conn = rusqlite::Connection::open(dir.path().join("x.sqlite")).unwrap();
    conn.execute_batch("CREATE TABLE t(a)").unwrap();
    let now = Local.with_ymd_and_hms(2026, 11, 24, 10, 0, 0).unwrap();
    let target = dir.path().join("b");
    let a = backup_to_dir(&conn, &target, "x", now).unwrap();
    let b = backup_to_dir(&conn, &target, "x", now).unwrap();
    assert!(a.ends_with("x-20261124-100000.sqlite"));
    assert!(b.ends_with("x-20261124-100000-2.sqlite"));
}

#[test]
fn prune_keeps_newest_20() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..22 {
        std::fs::write(
            dir.path().join(format!("daten-20260101-1000{i:02}.sqlite")),
            b"x",
        )
        .unwrap();
    }
    std::fs::write(dir.path().join("fremd.txt"), b"x").unwrap();
    prune(dir.path(), "daten", BACKUP_KEEP).unwrap();
    let names = backups(dir.path());
    assert_eq!(names.len(), 21); // 20 Sicherungen + fremde Datei
    assert!(!names.contains(&"daten-20260101-100000.sqlite".to_string()));
    assert!(!names.contains(&"daten-20260101-100001.sqlite".to_string()));
    assert!(names.contains(&"daten-20260101-100002.sqlite".to_string()));
    assert!(names.contains(&"fremd.txt".to_string()));
}

#[test]
fn open_while_locked_is_read_only_with_holder() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let _first = Workspace::open(&db, &who("anna")).unwrap();
    let second = Workspace::open(&db, &who("reto")).unwrap();
    match second.mode {
        AccessMode::ReadOnly { held_by } => assert_eq!(held_by.user, "anna"),
        AccessMode::ReadWrite => panic!("expected read-only"),
    }
}

#[test]
fn read_only_workspace_rejects_writes() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let _first = Workspace::open(&db, &who("anna")).unwrap();
    let second = Workspace::open(&db, &who("reto")).unwrap();
    let err = second
        .conn
        .execute("INSERT INTO teams(name) VALUES ('A')", [])
        .map_err(CoreError::from)
        .unwrap_err();
    assert!(matches!(err, CoreError::ReadOnly), "got {err:?}");
}

#[test]
fn take_over_gives_read_write() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let _first = Workspace::open(&db, &who("anna")).unwrap();
    let second = Workspace::open_taking_over(&db, &who("reto")).unwrap();
    assert!(matches!(second.mode, AccessMode::ReadWrite));
    second
        .conn
        .execute("INSERT INTO teams(name) VALUES ('A')", [])
        .unwrap();
}

#[test]
fn locked_and_missing_db_returns_lock_held() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let _lock = LockGuard::acquire(&db, &who("anna")).unwrap();
    assert!(matches!(
        Workspace::open(&db, &who("reto")),
        Err(CoreError::LockHeld(_))
    ));
}

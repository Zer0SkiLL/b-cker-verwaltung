use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use verwaltung_core::db::{schema_version, SCHEMA_VERSION};
use verwaltung_core::error::CoreError;
use verwaltung_core::repo::settings::{self, LAST_SECONDARY_BACKUP, SECONDARY_BACKUP_DIR};
use verwaltung_core::storage::backup::{self, local_backup_dir};
use verwaltung_core::storage::lock::{Identity, LockGuard};
use verwaltung_core::storage::workspace::{SecondaryResult, Workspace};

fn who(user: &str) -> Identity {
    Identity {
        user: user.into(),
        host: "pc1".into(),
    }
}

fn setup() -> (tempfile::TempDir, PathBuf, Workspace) {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let ws = Workspace::open(&db, &who("reto")).unwrap();
    (dir, db, ws)
}

fn set_secondary(ws: &Workspace, dir: &Path) {
    settings::set(&ws.conn, SECONDARY_BACKUP_DIR, &dir.to_string_lossy()).unwrap();
}

fn team_names(ws: &Workspace) -> Vec<String> {
    let mut stmt = ws
        .conn
        .prepare("SELECT name FROM teams ORDER BY name")
        .unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

#[test]
fn schema_version_is_current_and_settings_roundtrip() {
    let (_d, _db, ws) = setup();
    assert_eq!(SCHEMA_VERSION, 3);
    assert_eq!(schema_version(&ws.conn).unwrap(), 3);
    assert_eq!(settings::get(&ws.conn, "x").unwrap(), None);
    settings::set(&ws.conn, "x", "1").unwrap();
    settings::set(&ws.conn, "x", "2").unwrap();
    assert_eq!(settings::get(&ws.conn, "x").unwrap().as_deref(), Some("2"));
    settings::remove(&ws.conn, "x").unwrap();
    assert_eq!(settings::get(&ws.conn, "x").unwrap(), None);
}

#[test]
fn backup_now_without_secondary_reports_not_configured() {
    let (_d, db, mut ws) = setup();
    let out = ws.backup_now(Local::now()).unwrap();
    assert_eq!(out.secondary, SecondaryResult::NotConfigured);
    assert!(out.local.starts_with(local_backup_dir(&db)));
    assert!(out.local.exists());
}

#[test]
fn backup_now_writes_secondary_and_sets_last_time() {
    let (dir, _db, mut ws) = setup();
    let usb = dir.path().join("usb");
    set_secondary(&ws, &usb);
    let out = ws.backup_now(Local::now()).unwrap();
    match out.secondary {
        SecondaryResult::Done(path) => assert!(path.starts_with(&usb) && path.exists()),
        other => panic!("expected Done, got {other:?}"),
    }
    assert!(settings::get(&ws.conn, LAST_SECONDARY_BACKUP)
        .unwrap()
        .is_some());
}

#[test]
fn unavailable_secondary_does_not_fail() {
    let (dir, _db, mut ws) = setup();
    let file = dir.path().join("datei.txt");
    std::fs::write(&file, b"x").unwrap();
    set_secondary(&ws, &file.join("unterordner"));
    let out = ws.backup_now(Local::now()).unwrap();
    assert!(matches!(out.secondary, SecondaryResult::Unavailable(_)));
    assert!(out.local.exists());
    assert_eq!(
        settings::get(&ws.conn, LAST_SECONDARY_BACKUP).unwrap(),
        None
    );
}

#[test]
fn secondary_thins_out_like_local() {
    let (dir, _db, mut ws) = setup();
    let usb = dir.path().join("usb");
    set_secondary(&ws, &usb);
    let start = Local.with_ymd_and_hms(2026, 11, 25, 12, 0, 0).unwrap();
    // Drei Sicherungen an einem älteren Tag, eine am Tag danach.
    let old = start - Duration::days(20);
    for h in 0..3 {
        ws.backup_now(old + Duration::hours(h)).unwrap();
    }
    ws.backup_now(old + Duration::days(1)).unwrap();
    // Viele Sicherungen kurz hintereinander am selben Tag bleiben alle.
    for i in 0..22 {
        ws.backup_now(start + Duration::seconds(i)).unwrap();
    }
    let files = backup::list(&usb, "daten").unwrap();
    let on = |d: DateTime<Local>| {
        files
            .iter()
            .filter(|f| f.created.date() == d.date_naive())
            .count()
    };
    assert_eq!(on(start), 22);
    // Der ältere Tag ist der dritte Tag mit Sicherungen: nur die neueste bleibt.
    assert_eq!(on(old), 1);
}

#[test]
fn reminder_due_rules() {
    let (dir, _db, ws) = setup();
    let now = Utc::now();
    assert!(backup::status(&ws.conn, now).unwrap().reminder_due);
    set_secondary(&ws, &dir.path().join("usb"));
    assert!(backup::status(&ws.conn, now).unwrap().reminder_due);
    settings::set(
        &ws.conn,
        LAST_SECONDARY_BACKUP,
        &(now - Duration::days(29)).to_rfc3339(),
    )
    .unwrap();
    let status = backup::status(&ws.conn, now).unwrap();
    assert!(!status.reminder_due);
    assert!(status.secondary_dir.is_some() && status.last_secondary.is_some());
    settings::set(
        &ws.conn,
        LAST_SECONDARY_BACKUP,
        &(now - Duration::days(31)).to_rfc3339(),
    )
    .unwrap();
    assert!(backup::status(&ws.conn, now).unwrap().reminder_due);
}

#[test]
fn list_backups_newest_first_ignores_foreign_files() {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "daten-20260101-100000.sqlite",
        "daten-20260301-100000.sqlite",
        "daten-20260201-100000-2.sqlite",
        "fremd.sqlite",
        "daten-kaputt.sqlite",
    ] {
        std::fs::write(dir.path().join(name), b"x").unwrap();
    }
    let names: Vec<String> = backup::list(dir.path(), "daten")
        .unwrap()
        .into_iter()
        .map(|b| b.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec![
            "daten-20260301-100000.sqlite",
            "daten-20260201-100000-2.sqlite",
            "daten-20260101-100000.sqlite"
        ]
    );
}

#[test]
fn restore_replaces_data_and_keeps_pre_restore_backup() {
    let (dir, db, mut ws) = setup();
    ws.conn
        .execute("INSERT INTO teams(name) VALUES ('A')", [])
        .unwrap();
    let saved = ws.backup_now(Local::now()).unwrap().local;
    ws.conn
        .execute("INSERT INTO teams(name) VALUES ('B')", [])
        .unwrap();
    ws.restore(&saved, Local::now() + Duration::seconds(5))
        .unwrap();
    assert_eq!(team_names(&ws), vec!["A"]);
    assert!(ws.is_read_write());
    let has_b = backup::list(&local_backup_dir(&db), "daten")
        .unwrap()
        .iter()
        .any(|b| {
            let c = rusqlite::Connection::open(&b.path).unwrap();
            c.query_row("SELECT COUNT(*) FROM teams WHERE name = 'B'", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
                == 1
        });
    assert!(has_b);
    assert!(matches!(
        LockGuard::acquire(&db, &who("x")),
        Err(CoreError::LockHeld(_))
    ));
    drop(dir);
}

#[test]
fn restore_rejects_corrupt_file_and_keeps_current() {
    let (dir, _db, mut ws) = setup();
    ws.conn
        .execute("INSERT INTO teams(name) VALUES ('A')", [])
        .unwrap();
    let junk = dir.path().join("kaputt.sqlite");
    std::fs::write(
        &junk,
        b"das ist keine datenbank, aber lang genug um wie eine auszusehen?",
    )
    .unwrap();
    match ws.restore(&junk, Local::now()) {
        Err(CoreError::Validation(m)) => assert_eq!(m, "Sicherung ist beschädigt"),
        other => panic!("expected Validation, got {:?}", other.map(|_| ())),
    }
    assert_eq!(team_names(&ws), vec!["A"]);
    drop(dir);
}

#[test]
fn restore_rejects_newer_schema() {
    let (_dir, _db, mut ws) = setup();
    let copy = ws.backup_now(Local::now()).unwrap().local;
    rusqlite::Connection::open(&copy)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    match ws.restore(&copy, Local::now()) {
        Err(CoreError::Validation(m)) => {
            assert_eq!(m, "Sicherung stammt von einer neueren Programmversion")
        }
        other => panic!("expected Validation, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn restore_in_read_only_returns_read_only() {
    let (_dir, db, mut ws) = setup();
    let copy = ws.backup_now(Local::now()).unwrap().local;
    let mut second = Workspace::open(&db, &who("anna")).unwrap();
    assert!(matches!(
        second.restore(&copy, Local::now()),
        Err(CoreError::ReadOnly)
    ));
    assert!(matches!(ws_backup_read_only(&db), Err(CoreError::ReadOnly)));
}

fn ws_backup_read_only(db: &Path) -> Result<(), CoreError> {
    let mut ro = Workspace::open(db, &who("anna"))?;
    ro.backup_now(Local::now()).map(|_| ())
}

#[test]
fn close_writes_secondary_and_releases_lock() {
    let (dir, db, ws) = setup();
    let usb = dir.path().join("usb");
    set_secondary(&ws, &usb);
    assert!(matches!(
        ws.close(Local::now()).unwrap(),
        SecondaryResult::Done(_)
    ));
    assert_eq!(backup::list(&usb, "daten").unwrap().len(), 1);
    assert!(LockGuard::acquire(&db, &who("x")).is_ok());
}

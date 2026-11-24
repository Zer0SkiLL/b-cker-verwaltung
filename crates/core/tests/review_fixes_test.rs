//! Regressionstests für die Befunde des abschliessenden Reviews.
mod common;
use chrono::{Duration, Local};
use common::d;
use verwaltung_core::storage::backup::{self, local_backup_dir, prune, BACKUP_KEEP};
use verwaltung_core::storage::lock::Identity;
use verwaltung_core::storage::workspace::Workspace;

fn who(user: &str) -> Identity {
    Identity { user: user.into(), host: "pc1".into() }
}

// C1
#[test]
fn prune_ignores_backups_of_similar_named_database() {
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("verband.sqlite");
    let copy = dir.path().join("verband-kopie.sqlite");
    let start = Local::now();
    let ws_copy = Workspace::open(&copy, &who("reto")).unwrap();
    for i in 0..BACKUP_KEEP as i64 {
        ws_copy.backup_now(start + Duration::seconds(i)).unwrap();
    }
    drop(ws_copy);
    let ws = Workspace::open(&main, &who("reto")).unwrap();
    let out = ws.backup_now(start + Duration::seconds(100)).unwrap();
    assert!(out.local.exists(), "eigene Sicherung wurde gelöscht");
    let backups = local_backup_dir(&main);
    assert_eq!(backup::list(&backups, "verband-kopie").unwrap().len(), BACKUP_KEEP);
}

// C1
#[test]
fn prune_keeps_foreign_file_with_matching_prefix() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("daten-kaputt.sqlite"), b"x").unwrap();
    for i in 0..3 {
        std::fs::write(dir.path().join(format!("daten-20260101-10000{i}.sqlite")), b"x").unwrap();
    }
    prune(dir.path(), "daten", 1).unwrap();
    assert!(dir.path().join("daten-kaputt.sqlite").exists());
    assert!(dir.path().join("daten-20260101-100002.sqlite").exists());
    assert!(!dir.path().join("daten-20260101-100001.sqlite").exists());
}

// C1 / Minor 5
#[test]
fn prune_keeps_newer_same_second_suffix() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("daten-20260101-100000.sqlite"), b"x").unwrap();
    std::fs::write(dir.path().join("daten-20260101-100000-2.sqlite"), b"x").unwrap();
    prune(dir.path(), "daten", 1).unwrap();
    assert!(dir.path().join("daten-20260101-100000-2.sqlite").exists());
    assert!(!dir.path().join("daten-20260101-100000.sqlite").exists());
}

#[allow(dead_code)]
fn unused() -> chrono::NaiveDate {
    d(2026, 1, 1)
}

// I1
#[test]
fn check_lock_switches_to_read_only_after_takeover() {
    use verwaltung_core::error::CoreError;
    use verwaltung_core::storage::workspace::AccessMode;
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let mut anna = Workspace::open(&db, &who("anna")).unwrap();
    assert_eq!(anna.check_lock().unwrap(), &AccessMode::ReadWrite);
    let _vertretung = Workspace::open_taking_over(&db, &who("vertretung")).unwrap();
    match anna.check_lock().unwrap() {
        AccessMode::ReadOnly { held_by } => assert_eq!(held_by.user, "vertretung"),
        AccessMode::ReadWrite => panic!("Sperre verloren, aber weiterhin schreibend"),
    }
    let err = anna.conn.execute("INSERT INTO teams(name) VALUES ('X')", []).map_err(CoreError::from).unwrap_err();
    assert!(matches!(err, CoreError::ReadOnly));
}

// I1
#[test]
fn close_after_takeover_writes_no_secondary_backup() {
    use verwaltung_core::repo::settings::{self, SECONDARY_BACKUP_DIR};
    use verwaltung_core::storage::workspace::SecondaryResult;
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let usb = dir.path().join("usb");
    let anna = Workspace::open(&db, &who("anna")).unwrap();
    settings::set(&anna.conn, SECONDARY_BACKUP_DIR, &usb.to_string_lossy()).unwrap();
    let _vertretung = Workspace::open_taking_over(&db, &who("vertretung")).unwrap();
    assert_eq!(anna.close(Local::now()).unwrap(), SecondaryResult::NotConfigured);
    assert!(backup::list(&usb, "daten").unwrap().is_empty());
}

// I1
#[test]
fn backup_now_after_takeover_is_refused() {
    use verwaltung_core::error::CoreError;
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("daten.sqlite");
    let anna = Workspace::open(&db, &who("anna")).unwrap();
    let _vertretung = Workspace::open_taking_over(&db, &who("vertretung")).unwrap();
    assert!(matches!(anna.backup_now(Local::now()), Err(CoreError::ReadOnly)));
}

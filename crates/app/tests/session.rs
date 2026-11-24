mod common;

use chrono::{Local, Utc};
use common::{db_path, fresh_session, org_input, who};
use verwaltung_app::dto::SecondaryResultDto;
use verwaltung_app::session::{Session, NO_FILE, READ_ONLY};
use verwaltung_core::repo::organisations;

#[test]
fn create_then_write_and_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = s
        .write(|c| organisations::create(c, &org_input("Beck AG")))
        .unwrap();
    let back = s.read(|c| organisations::get(c, org.id)).unwrap();
    assert_eq!(back.name, "Beck AG");
    let info = s.info(Utc::now()).unwrap();
    assert!(!info.read_only);
    assert!(info.held_by.is_none());
}

#[test]
fn create_refuses_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.close(Local::now()).unwrap();
    let err = Session::default()
        .create(&db_path(dir.path()), &who("anna"))
        .unwrap_err();
    assert!(err.0.contains("existiert bereits"), "{}", err.0);
}

#[test]
fn second_session_is_read_only_and_refuses_writes() {
    let dir = tempfile::tempdir().unwrap();
    let _anna = fresh_session(dir.path());
    let mut ben = Session::default();
    let info = ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    assert!(info.read_only);
    assert_eq!(info.held_by.unwrap().user, "anna");
    let err = ben
        .write(|c| organisations::create(c, &org_input("X")))
        .unwrap_err();
    assert_eq!(err.0, READ_ONLY);
}

#[test]
fn lost_lock_refuses_next_write() {
    let dir = tempfile::tempdir().unwrap();
    let mut anna = fresh_session(dir.path());
    let mut ben = Session::default();
    ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    ben.take_over(&who("ben")).unwrap();
    let err = anna
        .write(|c| organisations::create(c, &org_input("X")))
        .unwrap_err();
    assert_eq!(err.0, READ_ONLY);
    let info = anna.info(Utc::now()).unwrap();
    assert!(info.read_only);
    assert_eq!(info.held_by.unwrap().user, "ben");
}

#[test]
fn take_over_makes_writable() {
    let dir = tempfile::tempdir().unwrap();
    let _anna = fresh_session(dir.path());
    let mut ben = Session::default();
    ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    let info = ben.take_over(&who("ben")).unwrap();
    assert!(!info.read_only);
    ben.write(|c| organisations::create(c, &org_input("Ben AG")))
        .unwrap();
}

#[test]
fn calls_without_open_file_fail() {
    let s = Session::default();
    let err = s.read(organisations::list).unwrap_err();
    assert_eq!(err.0, NO_FILE);
    let err = Session::default().take_over(&who("ben")).unwrap_err();
    assert_eq!(err.0, NO_FILE);
}

#[test]
fn close_without_secondary_reports_not_configured() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    assert_eq!(
        s.close(Local::now()).unwrap(),
        Some(SecondaryResultDto::NotConfigured)
    );
    assert_eq!(s.close(Local::now()).unwrap(), None);
}

#[test]
fn reopen_closes_previous_and_releases_lock() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let info = s.open(&db_path(dir.path()), &who("anna")).unwrap();
    assert!(
        !info.read_only,
        "eigene alte Sperre muss vorher freigegeben werden"
    );
}

#[test]
fn new_file_reminds_about_secondary_backup() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let info = s.info(Utc::now()).unwrap();
    assert!(info.backup.reminder_due);
    assert!(info.backup.secondary_dir.is_none());
}

#[test]
fn session_info_serializes_camel_case() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let json = serde_json::to_value(s.info(Utc::now()).unwrap()).unwrap();
    assert_eq!(json["readOnly"], false);
    assert_eq!(json["backup"]["reminderDue"], true);
    let closed = serde_json::to_value(SecondaryResultDto::NotConfigured).unwrap();
    assert_eq!(closed["kind"], "notConfigured");
}

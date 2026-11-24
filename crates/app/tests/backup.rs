mod common;

use chrono::{Local, Utc};
use common::{db_path, fresh_session, org_input, who};
use verwaltung_app::commands::backup::{
    backup_now, backup_overview, export_all, restore_backup, set_secondary_dir,
};
use verwaltung_app::dto::SecondaryResultDto;
use verwaltung_app::session::{Session, READ_ONLY};
use verwaltung_core::repo::organisations;

#[test]
fn backup_now_creates_local_file_and_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let done = backup_now(&mut s, Local::now()).unwrap();
    assert!(std::path::Path::new(&done.local).exists());
    assert_eq!(done.secondary, SecondaryResultDto::NotConfigured);
    let overview = backup_overview(&mut s, Utc::now()).unwrap();
    assert_eq!(overview.files.len(), 1);
    assert_eq!(overview.files[0].path, done.local);
    assert_eq!(overview.files[0].created.len(), "24.11.2026 20:15".len());
}

#[test]
fn secondary_dir_must_exist() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let missing = dir.path().join("gibt-es-nicht");
    let err = set_secondary_dir(&mut s, Some(&missing)).unwrap_err();
    assert_eq!(err.0, "Der Ordner existiert nicht.");
    let stick = tempfile::tempdir().unwrap();
    set_secondary_dir(&mut s, Some(stick.path())).unwrap();
    let done = backup_now(&mut s, Local::now()).unwrap();
    assert!(matches!(done.secondary, SecondaryResultDto::Done { .. }));
    let overview = backup_overview(&mut s, Utc::now()).unwrap();
    assert!(!overview.status.reminder_due);
    set_secondary_dir(&mut s, None).unwrap();
    assert!(backup_overview(&mut s, Utc::now())
        .unwrap()
        .status
        .secondary_dir
        .is_none());
}

#[test]
fn restore_brings_back_old_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| organisations::create(c, &org_input("A")))
        .unwrap();
    let saved = backup_now(&mut s, Local::now()).unwrap();
    s.write(|c| organisations::create(c, &org_input("B")))
        .unwrap();
    restore_backup(&mut s, std::path::Path::new(&saved.local), Local::now()).unwrap();
    let names: Vec<String> = s
        .read(organisations::list)
        .unwrap()
        .into_iter()
        .map(|o| o.name)
        .collect();
    assert_eq!(names, vec!["A"]);
    assert!(!s.info(Utc::now()).unwrap().read_only);
}

#[test]
fn restore_refused_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let mut anna = fresh_session(dir.path());
    let saved = backup_now(&mut anna, Local::now()).unwrap();
    let mut ben = Session::default();
    ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    let err =
        restore_backup(&mut ben, std::path::Path::new(&saved.local), Local::now()).unwrap_err();
    assert_eq!(err.0, READ_ONLY);
    assert_eq!(backup_now(&mut ben, Local::now()).unwrap_err().0, READ_ONLY);
}

#[test]
fn export_all_writes_xlsx() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| organisations::create(c, &org_input("A")))
        .unwrap();
    let target = dir.path().join("alles.xlsx");
    export_all(&s, &target).unwrap();
    assert!(target.exists());
}

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

mod during_work {
    use super::*;
    use verwaltung_app::commands::backup::{backup_if_changed, get_retention, set_retention};
    use verwaltung_core::storage::retention::RetentionPolicy;

    #[test]
    fn no_backup_without_change_and_one_after_change() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = fresh_session(dir.path());
        // Die Vorgaben einer neuen Datei zählen als Änderung: einmal sichern, danach ist Ruhe.
        backup_if_changed(&mut s, Local::now()).unwrap();
        assert!(backup_if_changed(&mut s, Local::now()).unwrap().is_none());
        s.write(|c| organisations::create(c, &org_input("Beck AG")))
            .unwrap();
        assert!(backup_if_changed(&mut s, Local::now()).unwrap().is_some());
        assert!(backup_if_changed(&mut s, Local::now()).unwrap().is_none());
    }

    #[test]
    fn own_secondary_bookkeeping_does_not_count_as_change() {
        let dir = tempfile::tempdir().unwrap();
        let usb = dir.path().join("usb");
        std::fs::create_dir(&usb).unwrap();
        let mut s = fresh_session(dir.path());
        set_secondary_dir(&mut s, Some(&usb)).unwrap();
        s.write(|c| organisations::create(c, &org_input("Beck AG")))
            .unwrap();
        let first = backup_if_changed(&mut s, Local::now()).unwrap().unwrap();
        assert!(matches!(first.secondary, SecondaryResultDto::Done { .. }));
        assert!(backup_if_changed(&mut s, Local::now()).unwrap().is_none());
    }

    #[test]
    fn read_only_never_backs_up() {
        let dir = tempfile::tempdir().unwrap();
        let _anna = fresh_session(dir.path());
        let mut ben = Session::default();
        ben.open(&db_path(dir.path()), &who("ben")).unwrap();
        assert!(backup_if_changed(&mut ben, Local::now()).unwrap().is_none());
    }

    #[test]
    fn secondary_error_is_set_and_cleared() {
        let dir = tempfile::tempdir().unwrap();
        let usb = dir.path().join("usb");
        std::fs::create_dir(&usb).unwrap();
        let mut s = fresh_session(dir.path());
        set_secondary_dir(&mut s, Some(&usb)).unwrap();
        std::fs::remove_dir(&usb).unwrap();
        // Ordner fehlt: wird beim Sichern neu angelegt? Nur wenn das Laufwerk da ist; hier
        // simulieren wir „nicht erreichbar“ mit einer Datei an seiner Stelle.
        std::fs::write(&usb, b"kein Ordner").unwrap();
        backup_now(&mut s, Local::now()).unwrap();
        let info = s.info(Utc::now()).unwrap();
        assert!(info.backup.secondary_error.is_some(), "{info:?}");
        std::fs::remove_file(&usb).unwrap();
        std::fs::create_dir(&usb).unwrap();
        backup_now(&mut s, Local::now()).unwrap();
        assert_eq!(s.info(Utc::now()).unwrap().backup.secondary_error, None);
    }

    #[test]
    fn take_over_backs_up_at_secondary() {
        let dir = tempfile::tempdir().unwrap();
        let usb = dir.path().join("usb");
        std::fs::create_dir(&usb).unwrap();
        let mut anna = fresh_session(dir.path());
        set_secondary_dir(&mut anna, Some(&usb)).unwrap();
        let mut ben = Session::default();
        ben.open(&db_path(dir.path()), &who("ben")).unwrap();
        ben.take_over(&who("ben")).unwrap();
        assert_eq!(std::fs::read_dir(&usb).unwrap().count(), 1);
    }

    #[test]
    fn retention_roundtrip_validation_and_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = fresh_session(dir.path());
        assert_eq!(get_retention(&s).unwrap(), RetentionPolicy::default());
        let p = RetentionPolicy {
            interval_minutes: 30,
            ..Default::default()
        };
        set_retention(&mut s, &p).unwrap();
        assert_eq!(get_retention(&s).unwrap(), p);
        let err = set_retention(
            &mut s,
            &RetentionPolicy {
                interval_minutes: 5,
                ..p
            },
        )
        .unwrap_err();
        assert!(err.0.contains("Während der Arbeit sichern"), "{}", err.0);
        let mut ben = Session::default();
        ben.open(&db_path(dir.path()), &who("ben")).unwrap();
        assert_eq!(set_retention(&mut ben, &p).unwrap_err().0, READ_ONLY);
    }
}

mod secondary_error_reset {
    use super::*;

    fn session_with_error(dir: &std::path::Path) -> Session {
        let usb = dir.join("usb");
        std::fs::create_dir(&usb).unwrap();
        let mut s = fresh_session(dir);
        set_secondary_dir(&mut s, Some(&usb)).unwrap();
        std::fs::remove_dir(&usb).unwrap();
        std::fs::write(&usb, b"kein Ordner").unwrap();
        backup_now(&mut s, Local::now()).unwrap();
        assert!(s.info(Utc::now()).unwrap().backup.secondary_error.is_some());
        s
    }

    #[test]
    fn removing_secondary_dir_clears_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session_with_error(dir.path());
        set_secondary_dir(&mut s, None).unwrap();
        assert_eq!(s.info(Utc::now()).unwrap().backup.secondary_error, None);
    }

    #[test]
    fn opening_another_file_clears_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session_with_error(dir.path());
        let other = tempfile::tempdir().unwrap();
        s.create(&db_path(other.path()), &who("anna")).unwrap();
        assert_eq!(s.info(Utc::now()).unwrap().backup.secondary_error, None);
    }
}

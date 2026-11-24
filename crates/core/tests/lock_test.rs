use chrono::{Duration, Utc};
use verwaltung_core::error::CoreError;
use verwaltung_core::storage::lock::{is_stale, lock_path, Identity, LockGuard, LockInfo};

fn who(user: &str) -> Identity {
    Identity {
        user: user.into(),
        host: "pc1".into(),
    }
}

fn db_in(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("daten.sqlite")
}

#[test]
fn acquire_creates_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_in(&dir);
    let _g = LockGuard::acquire(&db, &who("reto")).unwrap();
    assert!(lock_path(&db).exists());
    assert_eq!(lock_path(&db), dir.path().join("daten.sqlite.lock"));
}

#[test]
fn second_acquire_returns_lock_held_with_holder() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_in(&dir);
    let _g = LockGuard::acquire(&db, &who("anna")).unwrap();
    match LockGuard::acquire(&db, &who("reto")) {
        Err(CoreError::LockHeld(info)) => assert_eq!(info.user, "anna"),
        other => panic!("expected LockHeld, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn drop_removes_own_lock() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_in(&dir);
    drop(LockGuard::acquire(&db, &who("reto")).unwrap());
    assert!(!lock_path(&db).exists());
}

#[test]
fn take_over_replaces_holder_and_old_drop_keeps_new_lock() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_in(&dir);
    let old = LockGuard::acquire(&db, &who("anna")).unwrap();
    let _new = LockGuard::take_over(&db, &who("reto")).unwrap();
    drop(old);
    assert!(lock_path(&db).exists());
    match LockGuard::acquire(&db, &who("x")) {
        Err(CoreError::LockHeld(info)) => assert_eq!(info.user, "reto"),
        other => panic!("expected LockHeld, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn corrupt_lock_file_reports_unbekannt() {
    let dir = tempfile::tempdir().unwrap();
    let db = db_in(&dir);
    std::fs::write(lock_path(&db), b"kein json").unwrap();
    match LockGuard::acquire(&db, &who("reto")) {
        Err(CoreError::LockHeld(info)) => {
            assert_eq!(info.user, "unbekannt");
            assert_eq!(info.host, "unbekannt");
        }
        other => panic!("expected LockHeld, got {:?}", other.map(|_| ())),
    }
}

#[test]
fn is_stale_after_24h() {
    let now = Utc::now();
    let info = |h: i64| LockInfo {
        user: "a".into(),
        host: "b".into(),
        created_at: now - Duration::hours(h),
    };
    assert!(!is_stale(&info(23), now));
    assert!(is_stale(&info(25), now));
}

//! Selbsttest nach dem Öffnen: Datei intakt, Platz, Sicherung, Dauer des Öffnens.
mod common;

use chrono::{Duration, Local};
use common::{db_path, fresh_session, who};
use verwaltung_app::commands::selftest::{evaluate, self_test, Facts, SLOW_OPEN_MS};

fn facts() -> Facts {
    Facts {
        integrity: "ok".into(),
        free_bytes: Some(10 * 1024 * 1024 * 1024),
        newest_backup_age_days: Some(1),
        writable: true,
        open_ms: Some(200),
    }
}

#[test]
fn healthy_facts_have_no_findings() {
    let r = evaluate(&facts());
    assert!(!r.integrity_problem && !r.low_space && !r.backup_stale && !r.slow_open);
    assert!(!r.any());
}

#[test]
fn each_finding_is_detected() {
    assert!(
        evaluate(&Facts {
            integrity: "*** in database main ***".into(),
            ..facts()
        })
        .integrity_problem
    );
    assert!(
        evaluate(&Facts {
            free_bytes: Some(100 * 1024 * 1024),
            ..facts()
        })
        .low_space
    );
    assert!(
        !evaluate(&Facts {
            free_bytes: None,
            ..facts()
        })
        .low_space
    );
    assert!(
        evaluate(&Facts {
            newest_backup_age_days: Some(8),
            ..facts()
        })
        .backup_stale
    );
    assert!(
        evaluate(&Facts {
            newest_backup_age_days: None,
            ..facts()
        })
        .backup_stale
    );
    // nur lesend: Sicherungen macht der andere PC
    assert!(
        !evaluate(&Facts {
            newest_backup_age_days: None,
            writable: false,
            ..facts()
        })
        .backup_stale
    );
    assert!(
        evaluate(&Facts {
            open_ms: Some(SLOW_OPEN_MS + 1),
            ..facts()
        })
        .slow_open
    );
    assert!(evaluate(&Facts {
        open_ms: Some(SLOW_OPEN_MS + 1),
        ..facts()
    })
    .any());
}

#[test]
fn self_test_on_fresh_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let r = self_test(&mut s, Local::now()).unwrap();
    assert!(!r.integrity_problem);
    assert!(!r.low_space);
    // neue Datei, noch keine Sicherung neben der Datei
    assert!(r.backup_stale);
    // nach einer Sicherung in Ordnung
    verwaltung_app::commands::backup::backup_now(&mut s, Local::now()).unwrap();
    assert!(!self_test(&mut s, Local::now()).unwrap().backup_stale);
    assert!(
        self_test(&mut s, Local::now() + Duration::days(9))
            .unwrap()
            .backup_stale
    );
}

#[test]
fn opening_records_its_duration() {
    let dir = tempfile::tempdir().unwrap();
    let _anna = fresh_session(dir.path());
    let mut ben = verwaltung_app::session::Session::default();
    let config = tempfile::tempdir().unwrap();
    verwaltung_app::commands::session::open_and_remember(
        &mut ben,
        config.path(),
        &db_path(dir.path()),
        &who("ben"),
        false,
    )
    .unwrap();
    assert!(ben.open_ms().is_some());
}

#[test]
fn self_test_without_file_fails_cleanly() {
    let mut s = verwaltung_app::session::Session::default();
    assert!(self_test(&mut s, Local::now()).is_err());
}

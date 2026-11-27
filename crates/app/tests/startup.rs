use std::path::PathBuf;
use verwaltung_app::startup::{db_from_env, open_startup_db};
use verwaltung_app::tauri_commands::AppState;
use verwaltung_core::storage::lock::Identity;

#[test]
fn db_from_env_ignores_missing_and_blank() {
    assert_eq!(db_from_env(None), None);
    assert_eq!(db_from_env(Some("  ".into())), None);
    assert_eq!(
        db_from_env(Some(" /tmp/x.sqlite ".into())),
        Some(PathBuf::from("/tmp/x.sqlite"))
    );
}

#[test]
fn startup_db_is_created_then_reopened() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("e2e.sqlite");
    let who = Identity {
        user: "e2e".into(),
        host: "ci".into(),
    };
    let state = AppState::new(dir.path().join("config"));
    open_startup_db(&state, &db, &who).unwrap();
    assert!(db.exists());
    let again = AppState::new(dir.path().join("config2"));
    let err = open_startup_db(&again, &db, &who).map(|_| ());
    assert!(
        err.is_ok(),
        "bestehende Datei wird geöffnet (nur lesend, weil gesperrt)"
    );
}

#[test]
fn single_instance_can_only_be_lifted_in_debug_builds() {
    use verwaltung_app::startup::single_instance;
    assert!(single_instance(false, None));
    assert!(single_instance(true, None));
    assert!(single_instance(true, Some("".into())));
    // nur Debug-Builds (E2E: zweites Programm simuliert einen zweiten PC)
    assert!(!single_instance(true, Some("1".into())));
    assert!(single_instance(false, Some("1".into())));
}

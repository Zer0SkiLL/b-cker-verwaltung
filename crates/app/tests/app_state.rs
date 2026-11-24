use verwaltung_app::tauri_commands::{close_session, AppState};
use verwaltung_core::storage::lock::{lock_path, Identity};

#[test]
fn close_session_releases_lock_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("verband.sqlite");
    let state = AppState::new(dir.path().join("config"));
    let who = Identity {
        user: "anna".into(),
        host: "pc1".into(),
    };
    state.with(|s| s.create(&db, &who)).unwrap();
    assert!(lock_path(&db).exists());
    close_session(&state);
    assert!(
        !lock_path(&db).exists(),
        "Sperre muss beim Beenden freigegeben werden"
    );
    close_session(&state);
}

use verwaltung_app::error::AppError;
use verwaltung_app::identity::identity_from_env;
use verwaltung_core::error::CoreError;

#[test]
fn identity_is_never_empty() {
    let who = identity_from_env();
    assert!(!who.user.trim().is_empty());
    assert!(!who.host.trim().is_empty());
}

#[test]
fn app_error_serializes_as_plain_text() {
    let err = AppError::from(CoreError::InUse { count: 2 });
    assert_eq!(err.to_string(), "Der Eintrag wird noch 2-mal verwendet.");
    assert_eq!(
        serde_json::to_string(&err).unwrap(),
        "\"Der Eintrag wird noch 2-mal verwendet.\""
    );
    let io = AppError::from(std::io::Error::new(std::io::ErrorKind::NotFound, "weg"));
    assert!(io.0.starts_with("Dateifehler"));
}

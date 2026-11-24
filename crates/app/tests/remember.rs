mod common;

use common::{db_path, who};
use verwaltung_app::commands::session::{open_and_remember, REMEMBER_FAILED};
use verwaltung_app::config::load_last_path;
use verwaltung_app::session::Session;

#[test]
fn successful_create_is_remembered() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let mut s = Session::default();
    let result =
        open_and_remember(&mut s, &config, &db_path(dir.path()), &who("anna"), true).unwrap();
    assert!(result.warning.is_none());
    assert_eq!(load_last_path(&config), Some(db_path(dir.path())));
}

#[test]
fn remember_failure_is_only_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("keine-ordner-datei");
    std::fs::write(&config, "x").unwrap();
    let mut s = Session::default();
    let result =
        open_and_remember(&mut s, &config, &db_path(dir.path()), &who("anna"), true).unwrap();
    assert_eq!(result.warning.as_deref(), Some(REMEMBER_FAILED));
    assert!(!result.info.read_only);
}

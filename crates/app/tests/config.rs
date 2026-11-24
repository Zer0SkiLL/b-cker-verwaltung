use std::path::PathBuf;
use verwaltung_app::config::{load_last_path, save_last_path};

#[test]
fn last_path_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    save_last_path(dir.path(), &PathBuf::from("/x/verband.sqlite")).unwrap();
    assert_eq!(load_last_path(dir.path()), Some(PathBuf::from("/x/verband.sqlite")));
}

#[test]
fn missing_config_is_none() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(load_last_path(dir.path()), None);
}

#[test]
fn corrupt_config_is_none() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("zuletzt.json"), "{kaputt").unwrap();
    assert_eq!(load_last_path(dir.path()), None);
}

#[test]
fn save_creates_dir() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b");
    save_last_path(&nested, &PathBuf::from("/y.sqlite")).unwrap();
    assert_eq!(load_last_path(&nested), Some(PathBuf::from("/y.sqlite")));
}

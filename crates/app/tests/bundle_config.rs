//! Installer- und Update-Konfiguration (tauri.conf.json).

fn config() -> serde_json::Value {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn bundles_for_all_platforms_with_updater_artifacts() {
    let c = config();
    let targets: Vec<&str> = c["bundle"]["targets"].as_array().unwrap().iter().filter_map(|t| t.as_str()).collect();
    assert_eq!(targets, vec!["nsis", "appimage", "deb", "dmg", "app"]);
    assert_eq!(c["bundle"]["createUpdaterArtifacts"], true);
}

#[test]
fn windows_installs_per_user_without_admin() {
    let c = config();
    assert_eq!(c["bundle"]["windows"]["nsis"]["installMode"], "currentUser");
    assert_eq!(c["plugins"]["updater"]["windows"]["installMode"], "passive");
}

#[test]
fn updater_points_to_github_releases_with_key() {
    let c = config();
    let endpoints = c["plugins"]["updater"]["endpoints"].as_array().unwrap();
    assert_eq!(
        endpoints[0],
        "https://github.com/Zer0SkiLL/b-cker-verwaltung/releases/latest/download/latest.json"
    );
    assert!(c["plugins"]["updater"]["pubkey"].as_str().unwrap().len() > 40);
}

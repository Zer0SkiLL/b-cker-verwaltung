//! Die Rechte des Fensters müssen das Öffnen des Mailprogramms (mailto:) erlauben.

#[test]
fn opener_allows_mailto_links() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/capabilities/default.json"
    ))
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let opener = json["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["identifier"] == "opener:allow-open-url")
        .expect("opener:allow-open-url mit Bereich fehlt");
    let urls: Vec<&str> = opener["allow"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a["url"].as_str())
        .collect();
    assert_eq!(
        urls,
        vec![
            "mailto:*",
            "https://github.com/Zer0SkiLL/b-cker-verwaltung*"
        ]
    );
}

#[test]
fn updater_and_restart_are_allowed() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/capabilities/default.json"
    ))
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    let ids: Vec<&str> = json["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p.as_str())
        .collect();
    assert!(ids.contains(&"updater:default"));
    assert!(ids.contains(&"process:allow-restart"));
}

//! Alle Befehle laufen ausserhalb des Hauptthreads, damit das Fenster bei langsamen
//! Netzlaufwerken (Sichern, Import, Export) nicht einfriert.

#[test]
fn every_tauri_command_is_async() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tauri_commands");
    let mut checked = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            !text.contains("#[tauri::command]\n"),
            "{} enthält einen Befehl ohne (async)",
            path.display()
        );
        checked += text.matches("#[tauri::command(async)]").count();
    }
    assert!(checked >= 50, "nur {checked} Befehle gefunden");
}

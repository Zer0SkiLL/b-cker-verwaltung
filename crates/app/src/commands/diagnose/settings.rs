//! Einstellungen der Analysedaten pro PC (`diagnose.json` im Konfigurationsordner).

use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const FILE_NAME: &str = "diagnose.json";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiagnoseSettings {
    /// `None`: Vorgabe der Version gilt.
    pub enabled: Option<bool>,
    pub last_sent: Option<String>,
    pub last_heartbeat: Option<String>,
    pub sent_day: Option<String>,
    pub sent_count: u32,
}

/// Fehlende oder unlesbare Datei ergibt die Vorgaben.
pub fn load(config_dir: &Path) -> DiagnoseSettings {
    std::fs::read_to_string(config_dir.join(FILE_NAME))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Alle Änderungen laufen nacheinander (Laden → Ändern → Speichern).
static LOCK: Mutex<()> = Mutex::new(());

/// Schreibt über eine Hilfsdatei und Umbenennen, damit nie eine halbe Datei entsteht
/// (eine kaputte Datei würde wieder als „ein“ gelesen).
pub fn save(config_dir: &Path, s: &DiagnoseSettings) -> AppResult<()> {
    std::fs::create_dir_all(config_dir)?;
    let text = serde_json::to_string_pretty(s)
        .map_err(|e| AppError(format!("Einstellungen nicht speicherbar: {e}")))?;
    let tmp = config_dir.join(format!("{FILE_NAME}.tmp"));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, config_dir.join(FILE_NAME))?;
    Ok(())
}

/// Ändert die Einstellungen unter der Sperre.
pub fn update(config_dir: &Path, change: impl FnOnce(&mut DiagnoseSettings)) -> AppResult<()> {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut s = load(config_dir);
    change(&mut s);
    save(config_dir, &s)
}

/// Übernimmt nach dem Senden nur die Zähler; den Schalter fasst das Senden nie an.
pub fn merge_counters(config_dir: &Path, sent: &DiagnoseSettings) -> AppResult<()> {
    update(config_dir, |s| {
        s.last_sent = sent.last_sent.clone();
        s.last_heartbeat = sent.last_heartbeat.clone();
        s.sent_day = sent.sent_day.clone();
        s.sent_count = sent.sent_count;
    })
}

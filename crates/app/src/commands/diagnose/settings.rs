//! Einstellungen der Analysedaten pro PC (`diagnose.json` im Konfigurationsordner).

use std::path::Path;

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

pub fn save(config_dir: &Path, s: &DiagnoseSettings) -> AppResult<()> {
    std::fs::create_dir_all(config_dir)?;
    let text = serde_json::to_string_pretty(s)
        .map_err(|e| AppError(format!("Einstellungen nicht speicherbar: {e}")))?;
    std::fs::write(config_dir.join(FILE_NAME), text)?;
    Ok(())
}

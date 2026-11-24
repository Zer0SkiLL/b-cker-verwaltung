use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

const FILE_NAME: &str = "zuletzt.json";

#[derive(Serialize, Deserialize)]
struct LastFile {
    db: PathBuf,
}

/// Zuletzt geöffnete Datenbank; `None`, wenn nichts gemerkt oder die Datei unlesbar ist.
pub fn load_last_path(config_dir: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(config_dir.join(FILE_NAME)).ok()?;
    serde_json::from_str::<LastFile>(&text).ok().map(|f| f.db)
}

/// Merkt sich den Pfad der Datenbank; legt das Konfigurationsverzeichnis bei Bedarf an.
pub fn save_last_path(config_dir: &Path, db: &Path) -> AppResult<()> {
    std::fs::create_dir_all(config_dir)?;
    let text = serde_json::to_string(&LastFile { db: db.to_path_buf() })
        .map_err(|e| crate::error::AppError(format!("Einstellungen nicht speicherbar: {e}")))?;
    std::fs::write(config_dir.join(FILE_NAME), text)?;
    Ok(())
}

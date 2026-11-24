//! Nur für automatische Tests (Debug-Builds): Datei beim Start aus einer Umgebungsvariable öffnen,
//! weil Dateidialoge in E2E-Tests nicht bedienbar sind.

use std::path::{Path, PathBuf};

use verwaltung_core::storage::lock::Identity;

use crate::dto::SessionInfo;
use crate::error::AppResult;
use crate::tauri_commands::AppState;

pub const ENV_VAR: &str = "MITGLIEDERVERWALTUNG_DB";

pub fn db_from_env(value: Option<String>) -> Option<PathBuf> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Öffnet die Datei, oder legt sie an, falls es sie noch nicht gibt.
pub fn open_startup_db(state: &AppState, path: &Path, who: &Identity) -> AppResult<SessionInfo> {
    state.with(|s| {
        if path.exists() {
            s.open(path, who)
        } else {
            s.create(path, who)
        }
    })
}

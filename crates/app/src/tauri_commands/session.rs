use std::path::PathBuf;

use chrono::{Local, Utc};
use tauri::State;

use super::AppState;
use crate::commands::session::{open_and_remember, OpenResult};
use crate::config::load_last_path;
use crate::dto::{SecondaryResultDto, SessionInfo};
use crate::error::{AppError, AppResult};
use crate::identity::identity_from_env;

#[tauri::command(async)]
pub fn last_path(state: State<AppState>) -> Option<String> {
    load_last_path(&state.config_dir).map(|p| p.display().to_string())
}

#[tauri::command(async)]
pub fn open_file(state: State<AppState>, path: String) -> AppResult<OpenResult> {
    let path = PathBuf::from(path);
    state.with(|s| open_and_remember(s, &state.config_dir, &path, &identity_from_env(), false))
}

#[tauri::command(async)]
pub fn create_file(state: State<AppState>, path: String) -> AppResult<OpenResult> {
    let path = PathBuf::from(path);
    state.with(|s| open_and_remember(s, &state.config_dir, &path, &identity_from_env(), true))
}

#[tauri::command(async)]
pub fn take_over(state: State<AppState>) -> AppResult<SessionInfo> {
    state.with(|s| s.take_over(&identity_from_env()))
}

#[tauri::command(async)]
pub fn session_info(state: State<AppState>) -> AppResult<SessionInfo> {
    state.with(|s| s.info(Utc::now()))
}

#[tauri::command(async)]
pub fn close_file(state: State<AppState>) -> AppResult<Option<SecondaryResultDto>> {
    state.with(|s| s.close(Local::now()))
}

#[tauri::command(async)]
pub fn diagnostics_log_tail(app: tauri::AppHandle) -> AppResult<Vec<String>> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|e| AppError(format!("Protokoll-Ordner unbekannt: {e}")))?;
    crate::commands::diagnostics::log_tail(&dir)
}

#[tauri::command(async)]
pub fn diagnostics_system() -> crate::commands::diagnostics::SystemInfoDto {
    crate::commands::diagnostics::system_info()
}

/// Momentaufnahme für den Problembericht. Die Sitzung ist nur für die Datei-Angaben gesperrt.
#[tauri::command(async)]
pub fn diagnostics_snapshot(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<crate::commands::diagnostics::SnapshotDto> {
    use crate::commands::diagnostics::{file_facts, snapshot_with};
    use tauri::Manager;
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|e| AppError(format!("Protokoll-Ordner unbekannt: {e}")))?;
    let file = state.with(|s| Ok(file_facts(s, Local::now().date_naive())))?;
    Ok(snapshot_with(file, &dir, &identity_from_env().host))
}

/// Protokollzeile aus der Oberfläche (`info`, `warn`, `error`).
#[tauri::command(async)]
pub fn diagnostics_log(level: String, message: String) {
    crate::commands::diagnostics::log_from_ui(&level, &message);
}

#[tauri::command(async)]
pub fn save_report(path: String, text: String) -> AppResult<()> {
    crate::commands::diagnostics::save_report(&PathBuf::from(path), &text)
}

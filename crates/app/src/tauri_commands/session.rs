use std::path::PathBuf;

use chrono::{Local, Utc};
use tauri::State;

use super::AppState;
use crate::config::{load_last_path, save_last_path};
use crate::dto::{SecondaryResultDto, SessionInfo};
use crate::error::AppResult;
use crate::identity::identity_from_env;

#[tauri::command]
pub fn last_path(state: State<AppState>) -> Option<String> {
    load_last_path(&state.config_dir).map(|p| p.display().to_string())
}

#[tauri::command]
pub fn open_file(state: State<AppState>, path: String) -> AppResult<SessionInfo> {
    let path = PathBuf::from(path);
    let info = state.with(|s| s.open(&path, &identity_from_env()))?;
    save_last_path(&state.config_dir, &path)?;
    Ok(info)
}

#[tauri::command]
pub fn create_file(state: State<AppState>, path: String) -> AppResult<SessionInfo> {
    let path = PathBuf::from(path);
    let info = state.with(|s| s.create(&path, &identity_from_env()))?;
    save_last_path(&state.config_dir, &path)?;
    Ok(info)
}

#[tauri::command]
pub fn take_over(state: State<AppState>) -> AppResult<SessionInfo> {
    state.with(|s| s.take_over(&identity_from_env()))
}

#[tauri::command]
pub fn session_info(state: State<AppState>) -> AppResult<SessionInfo> {
    state.with(|s| s.info(Utc::now()))
}

#[tauri::command]
pub fn close_file(state: State<AppState>) -> AppResult<Option<SecondaryResultDto>> {
    state.with(|s| s.close(Local::now()))
}

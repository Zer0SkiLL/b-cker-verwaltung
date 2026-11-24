use std::path::PathBuf;

use chrono::{Local, Utc};
use tauri::State;

use super::AppState;
use crate::commands::session::{open_and_remember, OpenResult};
use crate::config::load_last_path;
use crate::dto::{SecondaryResultDto, SessionInfo};
use crate::error::AppResult;
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

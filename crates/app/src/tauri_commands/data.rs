use std::path::PathBuf;

use chrono::{Local, Utc};
use tauri::State;
use verwaltung_core::query::filter::Filter;

use super::{today, AppState};
use crate::commands::addresses::{self as adr, ExportFormat};
use crate::commands::{backup as bk, import as imp};
use crate::error::AppResult;

#[tauri::command(async)]
pub fn search_addresses(state: State<AppState>, filter: Filter) -> AppResult<adr::SearchResult> {
    state.with(|s| adr::search_addresses(s, &filter, today()))
}

#[tauri::command(async)]
pub fn export_addresses(
    state: State<AppState>,
    filter: Filter,
    path: String,
    format: ExportFormat,
) -> AppResult<()> {
    state.with(|s| adr::export_addresses(s, &filter, today(), &PathBuf::from(path), format))
}

#[tauri::command(async)]
pub fn list_groups(state: State<AppState>) -> AppResult<Vec<adr::GroupDto>> {
    state.with(|s| adr::list_groups(s))
}

#[tauri::command(async)]
pub fn save_group(state: State<AppState>, name: String, filter: Filter) -> AppResult<i64> {
    state.with(|s| adr::save_group(s, &name, &filter))
}

#[tauri::command(async)]
pub fn load_group(state: State<AppState>, id: i64) -> AppResult<Filter> {
    state.with(|s| adr::load_group(s, id))
}

#[tauri::command(async)]
pub fn delete_group(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| adr::delete_group(s, id))
}

#[tauri::command(async)]
pub fn filter_options(state: State<AppState>) -> AppResult<adr::FilterOptions> {
    state.with(|s| adr::filter_options(s))
}

#[tauri::command(async)]
pub fn backup_overview(state: State<AppState>) -> AppResult<bk::BackupOverview> {
    state.with(|s| bk::backup_overview(s, Utc::now()))
}

#[tauri::command(async)]
pub fn set_secondary_dir(state: State<AppState>, dir: Option<String>) -> AppResult<()> {
    let dir = dir.map(PathBuf::from);
    state.with(|s| bk::set_secondary_dir(s, dir.as_deref()))
}

#[tauri::command(async)]
pub fn backup_now(state: State<AppState>) -> AppResult<bk::BackupNowDto> {
    state.with(|s| bk::backup_now(s, Local::now()))
}

#[tauri::command(async)]
pub fn restore_backup(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|s| bk::restore_backup(s, &PathBuf::from(path), Local::now()))
}

#[tauri::command(async)]
pub fn export_all(state: State<AppState>, path: String) -> AppResult<()> {
    state.with(|s| bk::export_all(s, &PathBuf::from(path)))
}

#[tauri::command(async)]
pub fn import_preview(
    state: State<AppState>,
    path: String,
    mapping: Option<Vec<String>>,
) -> AppResult<imp::ImportPreview> {
    state.with(|s| imp::import_preview(s, &PathBuf::from(path), mapping))
}

#[tauri::command(async)]
pub fn import_apply(
    state: State<AppState>,
    path: String,
    mapping: Vec<String>,
) -> AppResult<imp::ImportSummaryDto> {
    state.with(|s| imp::import_apply(s, &PathBuf::from(path), mapping))
}

#[tauri::command(async)]
pub fn export_report(report: Vec<imp::ReportEntryDto>, path: String) -> AppResult<()> {
    imp::export_report(&report, &PathBuf::from(path))
}

#[tauri::command(async)]
pub fn rename_group(state: State<AppState>, id: i64, name: String) -> AppResult<()> {
    state.with(|s| adr::rename_group(s, id, &name))
}

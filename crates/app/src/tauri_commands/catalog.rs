use tauri::State;
use verwaltung_core::model::{AppliesTo, Label, LabelGroup, LookupKind, LookupValue};
use verwaltung_core::repo::teams::Team;

use super::AppState;
use crate::commands::{catalog as cat, teams as tm};
use crate::error::AppResult;

#[tauri::command(async)]
pub fn list_lookups(state: State<AppState>, kind: LookupKind) -> AppResult<Vec<cat::LookupUsage>> {
    state.with(|s| cat::list_lookups(s, kind))
}

#[tauri::command(async)]
pub fn create_lookup(
    state: State<AppState>,
    kind: LookupKind,
    name: String,
) -> AppResult<LookupValue> {
    state.with(|s| cat::create_lookup(s, kind, &name))
}

#[tauri::command(async)]
pub fn rename_lookup(state: State<AppState>, id: i64, name: String) -> AppResult<LookupValue> {
    state.with(|s| cat::rename_lookup(s, id, &name))
}

#[tauri::command(async)]
pub fn delete_lookup(state: State<AppState>, id: i64, confirm: bool) -> AppResult<()> {
    state.with(|s| cat::delete_lookup(s, id, confirm))
}

#[tauri::command(async)]
pub fn list_label_tree(state: State<AppState>) -> AppResult<Vec<cat::LabelGroupTree>> {
    state.with(|s| cat::list_label_tree(s))
}

#[tauri::command(async)]
pub fn create_label_group(
    state: State<AppState>,
    name: String,
    applies_to: AppliesTo,
) -> AppResult<LabelGroup> {
    state.with(|s| cat::create_label_group(s, &name, applies_to))
}

#[tauri::command(async)]
pub fn create_label(state: State<AppState>, group_id: i64, name: String) -> AppResult<Label> {
    state.with(|s| cat::create_label(s, group_id, &name))
}

#[tauri::command(async)]
pub fn rename_label(state: State<AppState>, id: i64, name: String) -> AppResult<Label> {
    state.with(|s| cat::rename_label(s, id, &name))
}

#[tauri::command(async)]
pub fn delete_label(state: State<AppState>, id: i64, confirm: bool) -> AppResult<()> {
    state.with(|s| cat::delete_label(s, id, confirm))
}

#[tauri::command(async)]
pub fn delete_label_group(state: State<AppState>, id: i64, confirm: bool) -> AppResult<()> {
    state.with(|s| cat::delete_label_group(s, id, confirm))
}

#[tauri::command(async)]
pub fn list_teams(state: State<AppState>) -> AppResult<Vec<tm::TeamSummary>> {
    state.with(|s| tm::list_teams(s))
}

#[tauri::command(async)]
pub fn team_members(state: State<AppState>, team_id: i64) -> AppResult<Vec<tm::TeamMemberView>> {
    state.with(|s| tm::team_members(s, team_id))
}

#[tauri::command(async)]
pub fn create_team(
    state: State<AppState>,
    name: String,
    description: Option<String>,
) -> AppResult<Team> {
    state.with(|s| tm::create_team(s, &name, description.as_deref()))
}

#[tauri::command(async)]
pub fn update_team(
    state: State<AppState>,
    id: i64,
    name: String,
    description: Option<String>,
) -> AppResult<Team> {
    state.with(|s| tm::update_team(s, id, &name, description.as_deref()))
}

#[tauri::command(async)]
pub fn delete_team(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| tm::delete_team(s, id))
}

#[tauri::command(async)]
pub fn add_team_member(
    state: State<AppState>,
    team_id: i64,
    person_id: i64,
    role: Option<String>,
) -> AppResult<()> {
    state.with(|s| tm::add_team_member(s, team_id, person_id, role.as_deref()))
}

#[tauri::command(async)]
pub fn remove_team_member(state: State<AppState>, team_id: i64, person_id: i64) -> AppResult<()> {
    state.with(|s| tm::remove_team_member(s, team_id, person_id))
}

#[tauri::command(async)]
pub fn rename_label_group(state: State<AppState>, id: i64, name: String) -> AppResult<LabelGroup> {
    state.with(|s| cat::rename_label_group(s, id, &name))
}

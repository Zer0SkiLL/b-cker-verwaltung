use tauri::State;
use verwaltung_core::repo::affiliations::{Affiliation, AffiliationInput};
use verwaltung_core::repo::assignments::{Assignment, AssignmentInput};
use verwaltung_core::repo::memberships::{Membership, MembershipInput};
use verwaltung_core::repo::organisations::{Organisation, OrganisationInput};
use verwaltung_core::repo::persons::{Person, PersonInput};

use super::{today, AppState};
use crate::commands::entities as cmd;
use crate::error::AppResult;

#[tauri::command(async)]
pub fn organisation_detail(
    state: State<AppState>,
    id: i64,
    include_past: bool,
) -> AppResult<cmd::OrganisationDetail> {
    state.with(|s| cmd::organisation_detail(s, id, include_past, today()))
}

#[tauri::command(async)]
pub fn person_detail(
    state: State<AppState>,
    id: i64,
    include_past: bool,
) -> AppResult<cmd::PersonDetail> {
    state.with(|s| cmd::person_detail(s, id, include_past, today()))
}

#[tauri::command(async)]
pub fn list_people(state: State<AppState>) -> AppResult<Vec<cmd::OptionDto>> {
    state.with(|s| cmd::list_people(s))
}

#[tauri::command(async)]
pub fn list_organisations(state: State<AppState>) -> AppResult<Vec<cmd::OptionDto>> {
    state.with(|s| cmd::list_organisations(s))
}

#[tauri::command(async)]
pub fn create_organisation(
    state: State<AppState>,
    input: OrganisationInput,
) -> AppResult<Organisation> {
    state.with(|s| cmd::create_organisation(s, &input))
}

#[tauri::command(async)]
pub fn update_organisation(
    state: State<AppState>,
    id: i64,
    input: OrganisationInput,
) -> AppResult<Organisation> {
    state.with(|s| cmd::update_organisation(s, id, &input))
}

#[tauri::command(async)]
pub fn delete_organisation(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| cmd::delete_organisation(s, id))
}

#[tauri::command(async)]
pub fn create_person(state: State<AppState>, input: PersonInput) -> AppResult<Person> {
    state.with(|s| cmd::create_person(s, &input))
}

#[tauri::command(async)]
pub fn update_person(state: State<AppState>, id: i64, input: PersonInput) -> AppResult<Person> {
    state.with(|s| cmd::update_person(s, id, &input))
}

#[tauri::command(async)]
pub fn delete_person(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| cmd::delete_person(s, id))
}

#[tauri::command(async)]
pub fn create_affiliation(
    state: State<AppState>,
    input: AffiliationInput,
) -> AppResult<Affiliation> {
    state.with(|s| cmd::create_affiliation(s, &input))
}

#[tauri::command(async)]
pub fn update_affiliation(
    state: State<AppState>,
    id: i64,
    input: AffiliationInput,
) -> AppResult<Affiliation> {
    state.with(|s| cmd::update_affiliation(s, id, &input))
}

#[tauri::command(async)]
pub fn delete_affiliation(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| cmd::delete_affiliation(s, id))
}

#[tauri::command(async)]
pub fn create_membership(state: State<AppState>, input: MembershipInput) -> AppResult<Membership> {
    state.with(|s| cmd::create_membership(s, &input))
}

#[tauri::command(async)]
pub fn update_membership(
    state: State<AppState>,
    id: i64,
    input: MembershipInput,
) -> AppResult<Membership> {
    state.with(|s| cmd::update_membership(s, id, &input))
}

#[tauri::command(async)]
pub fn delete_membership(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| cmd::delete_membership(s, id))
}

#[tauri::command(async)]
pub fn assign_label(state: State<AppState>, input: AssignmentInput) -> AppResult<Assignment> {
    state.with(|s| cmd::assign_label(s, &input))
}

#[tauri::command(async)]
pub fn unassign_label(state: State<AppState>, id: i64) -> AppResult<()> {
    state.with(|s| cmd::unassign_label(s, id))
}

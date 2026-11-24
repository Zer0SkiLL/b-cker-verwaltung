//! Teams für Themen und ihre Mitglieder.

use serde::Serialize;
use verwaltung_core::repo::persons;
use verwaltung_core::repo::teams::{self, Team};

use crate::error::AppResult;
use crate::session::Session;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamSummary {
    pub team: Team,
    pub member_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMemberView {
    pub person_id: i64,
    pub name: String,
    pub role: Option<String>,
}

pub fn list_teams(s: &Session) -> AppResult<Vec<TeamSummary>> {
    s.read(|c| {
        teams::list(c)?
            .into_iter()
            .map(|team| {
                let member_count = teams::members(c, team.id)?.len();
                Ok(TeamSummary { team, member_count })
            })
            .collect()
    })
}

pub fn team_members(s: &Session, team_id: i64) -> AppResult<Vec<TeamMemberView>> {
    s.read(|c| {
        teams::members(c, team_id)?
            .into_iter()
            .map(|m| {
                Ok(TeamMemberView {
                    name: persons::get(c, m.person_id)?.display_name(),
                    person_id: m.person_id,
                    role: m.role,
                })
            })
            .collect()
    })
}

pub fn create_team(s: &mut Session, name: &str, description: Option<&str>) -> AppResult<Team> {
    s.write(|c| teams::create(c, name, description))
}

pub fn update_team(
    s: &mut Session,
    id: i64,
    name: &str,
    description: Option<&str>,
) -> AppResult<Team> {
    s.write(|c| teams::update(c, id, name, description))
}

pub fn delete_team(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| teams::delete(c, id))
}

pub fn add_team_member(
    s: &mut Session,
    team_id: i64,
    person_id: i64,
    role: Option<&str>,
) -> AppResult<()> {
    s.write(|c| teams::add_member(c, team_id, person_id, role))
}

pub fn remove_team_member(s: &mut Session, team_id: i64, person_id: i64) -> AppResult<()> {
    s.write(|c| teams::remove_member(c, team_id, person_id))
}

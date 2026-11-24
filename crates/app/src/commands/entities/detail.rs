use chrono::NaiveDate;
use rusqlite::Connection;
use serde::Serialize;
use verwaltung_core::error::Result as CoreResult;
use verwaltung_core::model::MemberRef;
use verwaltung_core::repo::affiliations::{self, Affiliation};
use verwaltung_core::repo::assignments::{self, Assignment};
use verwaltung_core::repo::memberships::{self, Membership};
use verwaltung_core::repo::organisations::{self, Organisation};
use verwaltung_core::repo::persons::{self, Person};
use verwaltung_core::repo::teams::{self, Team};
use verwaltung_core::repo::{labels, lookups};

use super::sort_key;
use crate::error::AppResult;
use crate::session::Session;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AffiliationView {
    pub affiliation: Affiliation,
    pub person_name: String,
    pub organisation_name: String,
    pub function_name: String,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignedLabel {
    pub assignment: Assignment,
    pub group_name: String,
    pub label_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TeamRole {
    pub team: Team,
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrganisationDetail {
    pub organisation: Organisation,
    pub people: Vec<AffiliationView>,
    pub memberships: Vec<Membership>,
    pub labels: Vec<AssignedLabel>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PersonDetail {
    pub person: Person,
    pub organisations: Vec<AffiliationView>,
    pub memberships: Vec<Membership>,
    pub labels: Vec<AssignedLabel>,
    pub teams: Vec<TeamRole>,
}

/// Eintrag für Auswahlfelder.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OptionDto {
    pub id: i64,
    pub name: String,
}

pub fn organisation_detail(
    s: &Session,
    id: i64,
    include_past: bool,
    today: NaiveDate,
) -> AppResult<OrganisationDetail> {
    s.read(|c| {
        let member = MemberRef::Organisation(id);
        Ok(OrganisationDetail {
            organisation: organisations::get(c, id)?,
            people: views(
                c,
                affiliations::for_organisation(c, id, include_past, today)?,
                today,
            )?,
            memberships: memberships::for_member(c, &member)?,
            labels: assigned_labels(c, &member)?,
        })
    })
}

pub fn person_detail(
    s: &Session,
    id: i64,
    include_past: bool,
    today: NaiveDate,
) -> AppResult<PersonDetail> {
    s.read(|c| {
        let member = MemberRef::Person(id);
        Ok(PersonDetail {
            person: persons::get(c, id)?,
            organisations: views(
                c,
                affiliations::for_person(c, id, include_past, today)?,
                today,
            )?,
            memberships: memberships::for_member(c, &member)?,
            labels: assigned_labels(c, &member)?,
            teams: team_roles(c, id)?,
        })
    })
}

/// Personen für Auswahlfelder, „Nachname Vorname", nach Name sortiert.
pub fn list_people(s: &Session) -> AppResult<Vec<OptionDto>> {
    let people = s.read(persons::list)?;
    let options = people
        .into_iter()
        .map(|p| OptionDto {
            id: p.id,
            name: [Some(p.last_name), p.first_name]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" "),
        })
        .collect();
    Ok(sorted(options))
}

/// Betriebe für Auswahlfelder, nach Name sortiert.
pub fn list_organisations(s: &Session) -> AppResult<Vec<OptionDto>> {
    let orgs = s.read(organisations::list)?;
    let options = orgs
        .into_iter()
        .map(|o| OptionDto {
            id: o.id,
            name: o.name,
        })
        .collect();
    Ok(sorted(options))
}

fn sorted(mut options: Vec<OptionDto>) -> Vec<OptionDto> {
    options.sort_by_key(|o| sort_key(&o.name));
    options
}

fn views(
    c: &Connection,
    list: Vec<Affiliation>,
    today: NaiveDate,
) -> CoreResult<Vec<AffiliationView>> {
    list.into_iter()
        .map(|a| {
            Ok(AffiliationView {
                person_name: persons::get(c, a.person_id)?.display_name(),
                organisation_name: organisations::get(c, a.organisation_id)?.name,
                function_name: lookups::get(c, a.function_id)?.name,
                current: a.is_current(today),
                affiliation: a,
            })
        })
        .collect()
}

fn assigned_labels(c: &Connection, member: &MemberRef) -> CoreResult<Vec<AssignedLabel>> {
    assignments::for_target(c, member)?
        .into_iter()
        .map(|a| {
            let label = labels::get_label(c, a.label_id)?;
            let group = labels::get_group(c, label.group_id)?;
            Ok(AssignedLabel {
                assignment: a,
                group_name: group.name,
                label_name: label.name,
            })
        })
        .collect()
}

fn team_roles(c: &Connection, person_id: i64) -> CoreResult<Vec<TeamRole>> {
    let mut roles = Vec::new();
    for team in teams::list(c)? {
        let member = teams::members(c, team.id)?
            .into_iter()
            .find(|m| m.person_id == person_id);
        if let Some(m) = member {
            roles.push(TeamRole { team, role: m.role });
        }
    }
    Ok(roles)
}

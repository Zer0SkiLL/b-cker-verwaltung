use verwaltung_core::repo::affiliations::{self, Affiliation, AffiliationInput};
use verwaltung_core::repo::assignments::{self, Assignment, AssignmentInput};
use verwaltung_core::repo::memberships::{self, Membership, MembershipInput};
use verwaltung_core::repo::organisations::{self, Organisation, OrganisationInput};
use verwaltung_core::repo::persons::{self, Person, PersonInput};

use crate::error::AppResult;
use crate::session::Session;

pub fn create_organisation(s: &mut Session, input: &OrganisationInput) -> AppResult<Organisation> {
    s.write(|c| organisations::create(c, input))
}

pub fn update_organisation(
    s: &mut Session,
    id: i64,
    input: &OrganisationInput,
) -> AppResult<Organisation> {
    s.write(|c| organisations::update(c, id, input))
}

pub fn delete_organisation(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| organisations::delete(c, id))
}

pub fn create_person(s: &mut Session, input: &PersonInput) -> AppResult<Person> {
    s.write(|c| persons::create(c, input))
}

pub fn update_person(s: &mut Session, id: i64, input: &PersonInput) -> AppResult<Person> {
    s.write(|c| persons::update(c, id, input))
}

pub fn delete_person(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| persons::delete(c, id))
}

pub fn create_affiliation(s: &mut Session, input: &AffiliationInput) -> AppResult<Affiliation> {
    s.write(|c| affiliations::create(c, input))
}

pub fn update_affiliation(
    s: &mut Session,
    id: i64,
    input: &AffiliationInput,
) -> AppResult<Affiliation> {
    s.write(|c| affiliations::update(c, id, input))
}

pub fn delete_affiliation(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| affiliations::delete(c, id))
}

pub fn create_membership(s: &mut Session, input: &MembershipInput) -> AppResult<Membership> {
    s.write(|c| memberships::create(c, input))
}

pub fn update_membership(
    s: &mut Session,
    id: i64,
    input: &MembershipInput,
) -> AppResult<Membership> {
    s.write(|c| memberships::update(c, id, input))
}

pub fn delete_membership(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| memberships::delete(c, id))
}

pub fn assign_label(s: &mut Session, input: &AssignmentInput) -> AppResult<Assignment> {
    s.write(|c| assignments::assign(c, input))
}

pub fn unassign_label(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| assignments::unassign(c, id))
}

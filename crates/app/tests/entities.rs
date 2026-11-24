mod common;

use chrono::NaiveDate;
use common::{db_path, fresh_session, org_input, who};
use verwaltung_app::commands::entities::{
    assign_label, create_affiliation, create_membership, create_organisation, create_person,
    delete_organisation, list_organisations, list_people, organisation_detail, person_detail,
    update_organisation,
};
use verwaltung_app::session::{Session, READ_ONLY};
use verwaltung_core::model::{AppliesTo, LookupKind, MemberRef};
use verwaltung_core::repo::affiliations::AffiliationInput;
use verwaltung_core::repo::assignments::AssignmentInput;
use verwaltung_core::repo::memberships::MembershipInput;
use verwaltung_core::repo::persons::PersonInput;
use verwaltung_core::repo::{labels, lookups, teams};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn today() -> NaiveDate {
    d(2026, 11, 24)
}

fn person(last: &str, first: &str) -> PersonInput {
    PersonInput {
        last_name: last.into(),
        first_name: Some(first.into()),
        ..Default::default()
    }
}

fn function(s: &mut Session, name: &str) -> i64 {
    s.write(|c| lookups::create(c, LookupKind::Function, name)).unwrap().id
}

#[test]
fn organisation_detail_shows_people_with_function_names() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let p = create_person(&mut s, &person("Muster", "Hans")).unwrap();
    let inhaber = function(&mut s, "Inhaber");
    let input = AffiliationInput {
        person_id: p.id,
        organisation_id: org.id,
        function_id: inhaber,
        valid_from: None,
        valid_to: None,
    };
    create_affiliation(&mut s, &input).unwrap();
    let detail = organisation_detail(&s, org.id, false, today()).unwrap();
    assert_eq!(detail.organisation.name, "Beck AG");
    assert_eq!(detail.people.len(), 1);
    assert_eq!(detail.people[0].person_name, "Hans Muster");
    assert_eq!(detail.people[0].function_name, "Inhaber");
    assert!(detail.people[0].current);
}

#[test]
fn past_affiliations_hidden_unless_requested() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let p = create_person(&mut s, &person("Alt", "Eva")).unwrap();
    let f = function(&mut s, "Mitarbeiterin");
    let input = AffiliationInput {
        person_id: p.id,
        organisation_id: org.id,
        function_id: f,
        valid_from: Some(d(2020, 1, 1)),
        valid_to: Some(d(2026, 11, 23)),
    };
    create_affiliation(&mut s, &input).unwrap();
    assert!(organisation_detail(&s, org.id, false, today()).unwrap().people.is_empty());
    let all = organisation_detail(&s, org.id, true, today()).unwrap();
    assert_eq!(all.people.len(), 1);
    assert!(!all.people[0].current);
}

#[test]
fn person_detail_lists_teams_labels_and_organisations() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let p = create_person(&mut s, &person("Muster", "Hans")).unwrap();
    let f = function(&mut s, "Inhaber");
    let aff = AffiliationInput {
        person_id: p.id,
        organisation_id: org.id,
        function_id: f,
        valid_from: None,
        valid_to: None,
    };
    create_affiliation(&mut s, &aff).unwrap();
    let label = s
        .write(|c| {
            let g = labels::create_group(c, "Rollen", AppliesTo::Person)?;
            labels::create_label(c, g.id, "Prüfungsexperte")
        })
        .unwrap();
    let assign = AssignmentInput {
        label_id: label.id,
        target: MemberRef::Person(p.id),
        assigned_on: None,
        note: None,
    };
    assign_label(&mut s, &assign).unwrap();
    s.write(|c| {
        let t = teams::create(c, "Prüfungen", None)?;
        teams::add_member(c, t.id, p.id, Some("Leitung"))
    })
    .unwrap();
    let detail = person_detail(&s, p.id, false, today()).unwrap();
    assert_eq!(detail.organisations[0].organisation_name, "Beck AG");
    assert_eq!(detail.labels[0].group_name, "Rollen");
    assert_eq!(detail.labels[0].label_name, "Prüfungsexperte");
    assert_eq!(detail.teams[0].team.name, "Prüfungen");
    assert_eq!(detail.teams[0].role.as_deref(), Some("Leitung"));
}

#[test]
fn invalid_affiliation_dates_return_core_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let p = create_person(&mut s, &person("Muster", "Hans")).unwrap();
    let f = function(&mut s, "Inhaber");
    let input = AffiliationInput {
        person_id: p.id,
        organisation_id: org.id,
        function_id: f,
        valid_from: Some(d(2024, 5, 1)),
        valid_to: Some(d(2024, 4, 1)),
    };
    let err = create_affiliation(&mut s, &input).unwrap_err();
    assert_eq!(err.0, "Bis-Datum liegt vor Von-Datum");
}

#[test]
fn label_for_wrong_target_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let label = s
        .write(|c| {
            let g = labels::create_group(c, "Rollen", AppliesTo::Person)?;
            labels::create_label(c, g.id, "Vorstand")
        })
        .unwrap();
    let input = AssignmentInput {
        label_id: label.id,
        target: MemberRef::Organisation(org.id),
        assigned_on: None,
        note: None,
    };
    let err = assign_label(&mut s, &input).unwrap_err();
    assert_eq!(err.0, "Label gilt nicht für diesen Eintragstyp");
}

#[test]
fn membership_shows_in_organisation_detail() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let input = MembershipInput {
        member: MemberRef::Organisation(org.id),
        category_id: None,
        joined_on: Some(d(2010, 1, 1)),
        left_on: None,
    };
    create_membership(&mut s, &input).unwrap();
    let detail = organisation_detail(&s, org.id, false, today()).unwrap();
    assert_eq!(detail.memberships.len(), 1);
}

#[test]
fn updates_refused_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let mut anna = fresh_session(dir.path());
    let org = create_organisation(&mut anna, &org_input("Beck AG")).unwrap();
    let mut ben = Session::default();
    ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    let err = update_organisation(&mut ben, org.id, &org_input("Neu")).unwrap_err();
    assert_eq!(err.0, READ_ONLY);
    assert_eq!(delete_organisation(&mut ben, org.id).unwrap_err().0, READ_ONLY);
}

#[test]
fn option_lists_sorted_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    create_organisation(&mut s, &org_input("Zopf AG")).unwrap();
    create_organisation(&mut s, &org_input("Ähre GmbH")).unwrap();
    create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    create_person(&mut s, &person("Zürcher", "Ida")).unwrap();
    create_person(&mut s, &person("Arnold", "Paul")).unwrap();
    let orgs: Vec<String> = list_organisations(&s).unwrap().into_iter().map(|o| o.name).collect();
    assert_eq!(orgs, vec!["Ähre GmbH", "Beck AG", "Zopf AG"]);
    let people: Vec<String> = list_people(&s).unwrap().into_iter().map(|p| p.name).collect();
    assert_eq!(people, vec!["Arnold Paul", "Zürcher Ida"]);
}

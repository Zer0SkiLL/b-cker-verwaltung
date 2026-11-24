mod common;

use chrono::NaiveDate;
use common::{fresh_session, org_input};
use verwaltung_app::commands::entities::*;
use verwaltung_core::model::{AppliesTo, LookupKind, MemberRef};
use verwaltung_core::repo::affiliations::AffiliationInput;
use verwaltung_core::repo::assignments::AssignmentInput;
use verwaltung_core::repo::memberships::MembershipInput;
use verwaltung_core::repo::persons::PersonInput;
use verwaltung_core::repo::{labels, lookups};

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()
}

#[test]
fn person_update_and_delete() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let input = PersonInput {
        last_name: "Keller".into(),
        ..Default::default()
    };
    let p = create_person(&mut s, &input).unwrap();
    let changed = PersonInput {
        compensation: Some("CHF 300 / Prüfungstag".into()),
        ..input
    };
    let updated = update_person(&mut s, p.id, &changed).unwrap();
    assert_eq!(
        updated.compensation.as_deref(),
        Some("CHF 300 / Prüfungstag")
    );
    delete_person(&mut s, p.id).unwrap();
    assert!(person_detail(&s, p.id, false, today()).is_err());
}

#[test]
fn affiliation_membership_and_label_update_and_delete() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = create_organisation(&mut s, &org_input("Beck AG")).unwrap();
    let p = create_person(
        &mut s,
        &PersonInput {
            last_name: "Muster".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let f = s
        .write(|c| lookups::create(c, LookupKind::Function, "Inhaber"))
        .unwrap();
    let aff_input = AffiliationInput {
        person_id: p.id,
        organisation_id: org.id,
        function_id: f.id,
        valid_from: None,
        valid_to: None,
    };
    let aff = create_affiliation(&mut s, &aff_input).unwrap();
    let ended = AffiliationInput {
        valid_to: Some(today()),
        ..aff_input
    };
    assert_eq!(
        update_affiliation(&mut s, aff.id, &ended).unwrap().valid_to,
        Some(today())
    );
    delete_affiliation(&mut s, aff.id).unwrap();

    let member = MemberRef::Organisation(org.id);
    let mem_input = MembershipInput {
        member,
        category_id: None,
        joined_on: None,
        left_on: None,
    };
    let mem = create_membership(&mut s, &mem_input).unwrap();
    let left = MembershipInput {
        left_on: Some(today()),
        ..mem_input
    };
    assert_eq!(
        update_membership(&mut s, mem.id, &left).unwrap().left_on,
        Some(today())
    );
    delete_membership(&mut s, mem.id).unwrap();

    let label = s
        .write(|c| {
            let g = labels::create_group(c, "Ausbildung", AppliesTo::Organisation)?;
            labels::create_label(c, g.id, "Lehrbetrieb")
        })
        .unwrap();
    let assignment = assign_label(
        &mut s,
        &AssignmentInput {
            label_id: label.id,
            target: member,
            assigned_on: None,
            note: None,
        },
    )
    .unwrap();
    unassign_label(&mut s, assignment.id).unwrap();

    let detail = organisation_detail(&s, org.id, true, today()).unwrap();
    assert!(detail.people.is_empty());
    assert!(detail.memberships.is_empty());
    assert!(detail.labels.is_empty());
}

mod common;
use common::{d, raw_org, raw_person, test_conn};
use verwaltung_core::model::MemberRef;
use verwaltung_core::repo::memberships::{self, MembershipInput};

#[test]
fn membership_for_person_and_for_organisation() {
    let conn = test_conn();
    let org = raw_org(&conn, "A");
    let p = raw_person(&conn, "Ehrenmitglied");
    let m1 = memberships::create(
        &conn,
        &MembershipInput {
            member: MemberRef::Organisation(org),
            category_id: None,
            joined_on: Some(d(1999, 1, 1)),
            left_on: None,
        },
    )
    .unwrap();
    memberships::create(
        &conn,
        &MembershipInput {
            member: MemberRef::Person(p),
            category_id: None,
            joined_on: None,
            left_on: None,
        },
    )
    .unwrap();
    assert_eq!(
        memberships::for_member(&conn, &MemberRef::Organisation(org)).unwrap(),
        vec![m1.clone()]
    );
    assert_eq!(
        memberships::for_member(&conn, &MemberRef::Person(p))
            .unwrap()
            .len(),
        1
    );
    let updated = memberships::update(
        &conn,
        m1.id,
        &MembershipInput {
            left_on: Some(d(2025, 12, 31)),
            ..input_of(&m1)
        },
    )
    .unwrap();
    assert_eq!(updated.left_on, Some(d(2025, 12, 31)));
    memberships::delete(&conn, m1.id).unwrap();
    assert!(
        memberships::for_member(&conn, &MemberRef::Organisation(org))
            .unwrap()
            .is_empty()
    );
}

fn input_of(m: &memberships::Membership) -> MembershipInput {
    MembershipInput {
        member: m.member,
        category_id: m.category_id,
        joined_on: m.joined_on,
        left_on: m.left_on,
    }
}

#[test]
fn membership_active_rules() {
    let conn = test_conn();
    let org = raw_org(&conn, "A");
    let today = d(2026, 6, 1);
    let make = |left: Option<chrono::NaiveDate>| {
        memberships::create(
            &conn,
            &MembershipInput {
                member: MemberRef::Organisation(org),
                category_id: None,
                joined_on: None,
                left_on: left,
            },
        )
        .unwrap()
    };
    assert!(make(None).is_active(today));
    assert!(make(Some(d(2026, 6, 2))).is_active(today));
    assert!(!make(Some(today)).is_active(today));
}

#[test]
fn left_before_joined_rejected() {
    let conn = test_conn();
    let org = raw_org(&conn, "A");
    let res = memberships::create(
        &conn,
        &MembershipInput {
            member: MemberRef::Organisation(org),
            category_id: None,
            joined_on: Some(d(2020, 1, 1)),
            left_on: Some(d(2019, 1, 1)),
        },
    );
    assert!(res.is_err());
}

fn member(conn: &rusqlite::Connection, who: MemberRef, left_on: Option<chrono::NaiveDate>) {
    memberships::create(
        conn,
        &MembershipInput {
            member: who,
            category_id: None,
            joined_on: Some(d(2000, 1, 1)),
            left_on,
        },
    )
    .unwrap();
}

#[test]
fn count_active_members_counts_members_not_their_contacts() {
    let conn = test_conn();
    let today = d(2026, 11, 26);
    let beck = raw_org(&conn, "Beck AG");
    let brot = raw_org(&conn, "Brot GmbH");
    let alt = raw_org(&conn, "Alt AG");
    let einzel = raw_person(&conn, "Einzelmitglied");
    let kontakt = raw_person(&conn, "Kontakt");
    member(&conn, MemberRef::Organisation(beck), None);
    // Austritt erst in Zukunft: noch aktiv.
    member(&conn, MemberRef::Organisation(brot), Some(d(2027, 1, 1)));
    member(&conn, MemberRef::Organisation(alt), Some(d(2020, 1, 1)));
    member(&conn, MemberRef::Person(einzel), None);
    // Wieder eingetreten: alte und neue Mitgliedschaft, zählt einmal.
    member(&conn, MemberRef::Organisation(alt), None);
    let function = common::function(&conn, "Inhaber");
    verwaltung_core::repo::affiliations::create(
        &conn,
        &verwaltung_core::repo::affiliations::AffiliationInput {
            person_id: kontakt,
            organisation_id: beck,
            function_id: function,
            valid_from: None,
            valid_to: None,
        },
    )
    .unwrap();
    assert_eq!(memberships::count_active_members(&conn, today).unwrap(), 4);
}

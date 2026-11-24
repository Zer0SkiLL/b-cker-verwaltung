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
        &MembershipInput { member: MemberRef::Organisation(org), category_id: None, joined_on: Some(d(1999, 1, 1)), left_on: None },
    )
    .unwrap();
    memberships::create(
        &conn,
        &MembershipInput { member: MemberRef::Person(p), category_id: None, joined_on: None, left_on: None },
    )
    .unwrap();
    assert_eq!(memberships::for_member(&conn, &MemberRef::Organisation(org)).unwrap(), vec![m1.clone()]);
    assert_eq!(memberships::for_member(&conn, &MemberRef::Person(p)).unwrap().len(), 1);
    let updated = memberships::update(&conn, m1.id, &MembershipInput { left_on: Some(d(2025, 12, 31)), ..input_of(&m1) }).unwrap();
    assert_eq!(updated.left_on, Some(d(2025, 12, 31)));
    memberships::delete(&conn, m1.id).unwrap();
    assert!(memberships::for_member(&conn, &MemberRef::Organisation(org)).unwrap().is_empty());
}

fn input_of(m: &memberships::Membership) -> MembershipInput {
    MembershipInput { member: m.member, category_id: m.category_id, joined_on: m.joined_on, left_on: m.left_on }
}

#[test]
fn membership_active_rules() {
    let conn = test_conn();
    let org = raw_org(&conn, "A");
    let today = d(2026, 6, 1);
    let make = |left: Option<chrono::NaiveDate>| {
        memberships::create(
            &conn,
            &MembershipInput { member: MemberRef::Organisation(org), category_id: None, joined_on: None, left_on: left },
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
        &MembershipInput { member: MemberRef::Organisation(org), category_id: None, joined_on: Some(d(2020, 1, 1)), left_on: Some(d(2019, 1, 1)) },
    );
    assert!(res.is_err());
}

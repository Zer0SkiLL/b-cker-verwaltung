mod common;
use common::{count, d, function, raw_org, raw_person, test_conn};
use verwaltung_core::error::CoreError;
use verwaltung_core::repo::affiliations::{self, AffiliationInput};
use verwaltung_core::repo::persons;

fn link(person_id: i64, organisation_id: i64, function_id: i64) -> AffiliationInput {
    AffiliationInput {
        person_id,
        organisation_id,
        function_id,
        valid_from: None,
        valid_to: None,
    }
}

#[test]
fn person_can_have_two_affiliations() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let (a, b) = (raw_org(&conn, "A"), raw_org(&conn, "B"));
    affiliations::create(&conn, &link(p, a, function(&conn, "Inhaber"))).unwrap();
    affiliations::create(&conn, &link(p, b, function(&conn, "Mitarbeiter"))).unwrap();
    assert_eq!(
        affiliations::for_person(&conn, p, false, d(2026, 1, 1))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        affiliations::for_organisation(&conn, a, false, d(2026, 1, 1))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn past_affiliation_hidden_by_default_and_shown_with_include_past() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let a = raw_org(&conn, "A");
    let f = function(&conn, "Lernende");
    let past = affiliations::create(
        &conn,
        &AffiliationInput {
            valid_from: Some(d(2020, 8, 1)),
            valid_to: Some(d(2023, 7, 31)),
            ..link(p, a, f)
        },
    )
    .unwrap();
    let today = d(2026, 1, 1);
    assert!(!past.is_current(today));
    assert!(past.is_current(d(2023, 7, 31)));
    assert!(affiliations::for_person(&conn, p, false, today)
        .unwrap()
        .is_empty());
    assert_eq!(
        affiliations::for_person(&conn, p, true, today).unwrap(),
        vec![past]
    );
}

#[test]
fn valid_to_before_from_rejected() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let a = raw_org(&conn, "A");
    let input = AffiliationInput {
        valid_from: Some(d(2024, 1, 1)),
        valid_to: Some(d(2023, 1, 1)),
        ..link(p, a, function(&conn, "Inhaber"))
    };
    let err = affiliations::create(&conn, &input).unwrap_err();
    assert!(matches!(err, CoreError::Validation(ref m) if m == "Bis-Datum liegt vor Von-Datum"));
}

#[test]
fn update_and_delete_affiliation() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let a = raw_org(&conn, "A");
    let created = affiliations::create(&conn, &link(p, a, function(&conn, "Inhaber"))).unwrap();
    let updated = affiliations::update(
        &conn,
        created.id,
        &AffiliationInput {
            valid_to: Some(d(2025, 12, 31)),
            ..link(p, a, created.function_id)
        },
    )
    .unwrap();
    assert_eq!(updated.valid_to, Some(d(2025, 12, 31)));
    affiliations::delete(&conn, created.id).unwrap();
    assert!(matches!(
        affiliations::delete(&conn, created.id),
        Err(CoreError::NotFound { .. })
    ));
}

#[test]
fn deleting_person_removes_affiliations() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let a = raw_org(&conn, "A");
    affiliations::create(&conn, &link(p, a, function(&conn, "Inhaber"))).unwrap();
    persons::delete(&conn, p).unwrap();
    assert_eq!(count(&conn, "affiliations"), 0);
    assert_eq!(count(&conn, "organisations"), 1);
}

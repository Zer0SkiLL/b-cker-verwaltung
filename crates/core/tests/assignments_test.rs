mod common;
use common::{d, raw_org, raw_person, test_conn};
use verwaltung_core::error::CoreError;
use verwaltung_core::model::{AppliesTo, MemberRef};
use verwaltung_core::repo::assignments::{self, AssignmentInput};
use verwaltung_core::repo::labels;

#[test]
fn assign_with_date_and_note() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let l = labels::find_or_create_label(&conn, "Lehrgang", AppliesTo::Person, "Berufsprüfung").unwrap();
    let a = assignments::assign(
        &conn,
        &AssignmentInput { label_id: l.id, target: MemberRef::Person(p), assigned_on: Some(d(2019, 6, 30)), note: Some(" mit Auszeichnung ".into()) },
    )
    .unwrap();
    assert_eq!(a.note.as_deref(), Some("mit Auszeichnung"));
    assert_eq!(assignments::for_target(&conn, &MemberRef::Person(p)).unwrap(), vec![a.clone()]);
    assignments::unassign(&conn, a.id).unwrap();
    assert!(assignments::for_target(&conn, &MemberRef::Person(p)).unwrap().is_empty());
}

#[test]
fn person_only_label_rejected_for_organisation() {
    let conn = test_conn();
    let org = raw_org(&conn, "A");
    let l = labels::find_or_create_label(&conn, "Verbandsfunktion", AppliesTo::Person, "Vorstand").unwrap();
    let err = assignments::assign(
        &conn,
        &AssignmentInput { label_id: l.id, target: MemberRef::Organisation(org), assigned_on: None, note: None },
    )
    .unwrap_err();
    assert!(matches!(err, CoreError::Validation(ref m) if m == "Label gilt nicht für diesen Eintragstyp"));
}

#[test]
fn same_label_twice_allowed() {
    let conn = test_conn();
    let p = raw_person(&conn, "Muster");
    let l = labels::find_or_create_label(&conn, "Lehrgang", AppliesTo::Both, "Kurs").unwrap();
    for year in [2020, 2024] {
        assignments::assign(
            &conn,
            &AssignmentInput { label_id: l.id, target: MemberRef::Person(p), assigned_on: Some(d(year, 1, 1)), note: None },
        )
        .unwrap();
    }
    assert_eq!(assignments::for_target(&conn, &MemberRef::Person(p)).unwrap().len(), 2);
}

mod common;
use common::{count, raw_person, test_conn};
use verwaltung_core::error::CoreError;
use verwaltung_core::model::AppliesTo;
use verwaltung_core::repo::labels;

#[test]
fn create_group_and_labels_listed_by_name() {
    let conn = test_conn();
    let g = labels::create_group(&conn, "Verbandsfunktion", AppliesTo::Person).unwrap();
    labels::create_label(&conn, g.id, "Vorstand").unwrap();
    labels::create_label(&conn, g.id, "Prüfungsexperte").unwrap();
    let names: Vec<String> = labels::list_labels(&conn, g.id).unwrap().into_iter().map(|l| l.name).collect();
    assert_eq!(names, vec!["Prüfungsexperte", "Vorstand"]);
    assert_eq!(labels::list_groups(&conn).unwrap()[0].applies_to, AppliesTo::Person);
}

#[test]
fn rename_label_trims() {
    let conn = test_conn();
    let g = labels::create_group(&conn, "Lehrgang", AppliesTo::Person).unwrap();
    let l = labels::create_label(&conn, g.id, "BB-Kurs").unwrap();
    assert_eq!(labels::rename_label(&conn, l.id, " Berufsbildnerkurs ").unwrap().name, "Berufsbildnerkurs");
}

#[test]
fn find_or_create_label_creates_group_once() {
    let conn = test_conn();
    let a = labels::find_or_create_label(&conn, "Ausbildung Fachrichtung", AppliesTo::Organisation, "Fr Conf").unwrap();
    let b = labels::find_or_create_label(&conn, "Ausbildung Fachrichtung", AppliesTo::Organisation, "Fr Conf").unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(labels::list_groups(&conn).unwrap().len(), 1);
}

#[test]
fn empty_label_name_rejected() {
    let conn = test_conn();
    let g = labels::create_group(&conn, "X", AppliesTo::Both).unwrap();
    assert!(matches!(labels::create_label(&conn, g.id, ""), Err(CoreError::Validation(_))));
}

#[test]
fn label_delete_confirmed_removes_assignments() {
    let conn = test_conn();
    let g = labels::create_group(&conn, "Verbandsfunktion", AppliesTo::Person).unwrap();
    let l = labels::create_label(&conn, g.id, "Vorstand").unwrap();
    let p = raw_person(&conn, "Muster");
    conn.execute("INSERT INTO label_assignments(label_id, person_id) VALUES (?1, ?2)", [l.id, p]).unwrap();
    assert!(matches!(labels::delete_label(&conn, l.id, false), Err(CoreError::InUse { count: 1 })));
    labels::delete_label(&conn, l.id, true).unwrap();
    assert_eq!(count(&conn, "label_assignments"), 0);
    assert_eq!(count(&conn, "persons"), 1);
}

#[test]
fn group_delete_cascades_labels() {
    let conn = test_conn();
    let g = labels::create_group(&conn, "Lehrgang", AppliesTo::Person).unwrap();
    let l = labels::create_label(&conn, g.id, "BP").unwrap();
    let p = raw_person(&conn, "Muster");
    conn.execute("INSERT INTO label_assignments(label_id, person_id) VALUES (?1, ?2)", [l.id, p]).unwrap();
    assert!(matches!(labels::delete_group(&conn, g.id, false), Err(CoreError::InUse { count: 1 })));
    labels::delete_group(&conn, g.id, true).unwrap();
    assert_eq!(count(&conn, "labels"), 0);
    assert_eq!(count(&conn, "label_assignments"), 0);
}

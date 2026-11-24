mod common;
use common::test_conn;
use verwaltung_core::error::CoreError;
use verwaltung_core::repo::persons::{self, PersonInput};

fn input(first: Option<&str>, last: &str) -> PersonInput {
    PersonInput { first_name: first.map(String::from), last_name: last.into(), ..Default::default() }
}

#[test]
fn person_display_name() {
    let conn = test_conn();
    let p = persons::create(&conn, &input(Some(" Anna "), "Beispiel")).unwrap();
    assert_eq!(p.display_name(), "Anna Beispiel");
    let q = persons::create(&conn, &input(None, "Muster")).unwrap();
    assert_eq!(q.display_name(), "Muster");
}

#[test]
fn person_create_update_get_delete() {
    let conn = test_conn();
    let p = persons::create(
        &conn,
        &PersonInput { compensation: Some("CHF 300 / Prüfungstag".into()), ..input(Some("Hans"), "Muster") },
    )
    .unwrap();
    assert_eq!(p.compensation.as_deref(), Some("CHF 300 / Prüfungstag"));
    let u = persons::update(&conn, p.id, &PersonInput { city: Some("Thun".into()), ..input(Some("Hans"), "Müller") }).unwrap();
    assert_eq!(u.last_name, "Müller");
    assert_eq!(u.compensation, None);
    let st: String = conn.query_row("SELECT search_text FROM persons", [], |r| r.get(0)).unwrap();
    assert_eq!(st, "hans müller thun");
    assert_eq!(persons::list(&conn).unwrap().len(), 1);
    persons::delete(&conn, p.id).unwrap();
    assert!(matches!(persons::get(&conn, p.id), Err(CoreError::NotFound { .. })));
}

#[test]
fn empty_last_name_rejected() {
    let conn = test_conn();
    assert!(matches!(persons::create(&conn, &input(Some("A"), " ")), Err(CoreError::Validation(_))));
}

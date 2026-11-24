mod common;
use common::{count, raw_person, test_conn};
use verwaltung_core::error::CoreError;
use verwaltung_core::repo::organisations::{self, OrganisationInput};

fn input(name: &str) -> OrganisationInput {
    OrganisationInput {
        name: name.into(),
        ..Default::default()
    }
}

#[test]
fn create_trims_and_blanks_become_none() {
    let conn = test_conn();
    let org = organisations::create(
        &conn,
        &OrganisationInput {
            name: "  Bäckerei Muster ".into(),
            city: Some(" Bern ".into()),
            email: Some("   ".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(org.name, "Bäckerei Muster");
    assert_eq!(org.city.as_deref(), Some("Bern"));
    assert_eq!(org.email, None);
    assert_eq!(organisations::get(&conn, org.id).unwrap(), org);
}

#[test]
fn update_recomputes_search_text() {
    let conn = test_conn();
    let org = organisations::create(&conn, &input("Alt")).unwrap();
    organisations::update(
        &conn,
        org.id,
        &OrganisationInput {
            city: Some("Zürich".into()),
            ..input("Neu")
        },
    )
    .unwrap();
    let st: String = conn
        .query_row("SELECT search_text FROM organisations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(st, "neu zürich");
}

#[test]
fn get_missing_returns_not_found() {
    let conn = test_conn();
    assert!(matches!(
        organisations::get(&conn, 42),
        Err(CoreError::NotFound { id: 42, .. })
    ));
    assert!(matches!(
        organisations::update(&conn, 42, &input("X")),
        Err(CoreError::NotFound { .. })
    ));
}

#[test]
fn empty_name_rejected() {
    let conn = test_conn();
    assert!(matches!(
        organisations::create(&conn, &input("  ")),
        Err(CoreError::Validation(_))
    ));
}

#[test]
fn list_sorted_by_name() {
    let conn = test_conn();
    organisations::create(&conn, &input("zeta")).unwrap();
    organisations::create(&conn, &input("Alpha")).unwrap();
    let names: Vec<String> = organisations::list(&conn)
        .unwrap()
        .into_iter()
        .map(|o| o.name)
        .collect();
    assert_eq!(names, vec!["Alpha", "zeta"]);
}

#[test]
fn delete_organisation_keeps_persons() {
    let conn = test_conn();
    let org = organisations::create(&conn, &input("A")).unwrap();
    let p = raw_person(&conn, "Muster");
    conn.execute(
        "INSERT INTO lookup_values(kind, name) VALUES ('function', 'Inhaber')",
        [],
    )
    .unwrap();
    let f = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO affiliations(person_id, organisation_id, function_id) VALUES (?1, ?2, ?3)",
        [p, org.id, f],
    )
    .unwrap();
    organisations::delete(&conn, org.id).unwrap();
    assert_eq!(count(&conn, "affiliations"), 0);
    assert_eq!(count(&conn, "persons"), 1);
    assert!(matches!(
        organisations::delete(&conn, org.id),
        Err(CoreError::NotFound { .. })
    ));
}

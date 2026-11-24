mod common;
use common::{raw_org, raw_person, test_conn};
use verwaltung_core::error::CoreError;
use verwaltung_core::model::LookupKind;
use verwaltung_core::repo::lookups;

#[test]
fn create_and_list_sorted_by_sort_then_name() {
    let conn = test_conn();
    lookups::create(&conn, LookupKind::OrgType, "ÜK-Zentrum").unwrap();
    lookups::create(&conn, LookupKind::OrgType, "Andere").unwrap();
    lookups::create(&conn, LookupKind::Function, "Inhaber").unwrap();
    conn.execute(
        "UPDATE lookup_values SET sort = -1 WHERE name = 'ÜK-Zentrum'",
        [],
    )
    .unwrap();
    let names: Vec<String> = lookups::list(&conn, LookupKind::OrgType)
        .unwrap()
        .into_iter()
        .map(|v| v.name)
        .collect();
    assert_eq!(names, vec!["ÜK-Zentrum", "Andere"]);
}

#[test]
fn rename_updates_value() {
    let conn = test_conn();
    let v = lookups::create(&conn, LookupKind::Function, "Inhabr").unwrap();
    let renamed = lookups::rename(&conn, v.id, "  Inhaber ").unwrap();
    assert_eq!(renamed.name, "Inhaber");
    assert_eq!(renamed.kind, LookupKind::Function);
}

#[test]
fn find_or_create_is_idempotent() {
    let conn = test_conn();
    let a = lookups::find_or_create(&conn, LookupKind::CantonalAssociation, "Bern").unwrap();
    let b = lookups::find_or_create(&conn, LookupKind::CantonalAssociation, " Bern ").unwrap();
    assert_eq!(a.id, b.id);
    assert_eq!(
        lookups::list(&conn, LookupKind::CantonalAssociation)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn empty_name_is_rejected() {
    let conn = test_conn();
    let err = lookups::create(&conn, LookupKind::OrgType, "   ").unwrap_err();
    assert!(matches!(err, CoreError::Validation(ref m) if m == "Name darf nicht leer sein"));
}

#[test]
fn delete_in_use_without_confirm_returns_in_use_count() {
    let conn = test_conn();
    let typ = lookups::create(&conn, LookupKind::OrgType, "Mitgliedsbetrieb").unwrap();
    for name in ["A", "B"] {
        let id = raw_org(&conn, name);
        conn.execute(
            "UPDATE organisations SET org_type_id = ?1 WHERE id = ?2",
            [typ.id, id],
        )
        .unwrap();
    }
    assert_eq!(lookups::usage_count(&conn, typ.id).unwrap(), 2);
    let err = lookups::delete(&conn, typ.id, false).unwrap_err();
    assert!(matches!(err, CoreError::InUse { count: 2 }));
}

#[test]
fn delete_confirmed_nulls_references() {
    let conn = test_conn();
    let kat = lookups::create(&conn, LookupKind::MembershipCategory, "Aktiv").unwrap();
    let org = raw_org(&conn, "A");
    conn.execute(
        "INSERT INTO memberships(organisation_id, category_id) VALUES (?1, ?2)",
        [org, kat.id],
    )
    .unwrap();
    lookups::delete(&conn, kat.id, true).unwrap();
    let cat: Option<i64> = conn
        .query_row("SELECT category_id FROM memberships", [], |r| r.get(0))
        .unwrap();
    assert_eq!(cat, None);
    assert!(lookups::list(&conn, LookupKind::MembershipCategory)
        .unwrap()
        .is_empty());
}

#[test]
fn function_in_use_cannot_be_deleted() {
    let conn = test_conn();
    let f = lookups::create(&conn, LookupKind::Function, "Inhaber").unwrap();
    let org = raw_org(&conn, "A");
    let p = raw_person(&conn, "Muster");
    conn.execute(
        "INSERT INTO affiliations(person_id, organisation_id, function_id) VALUES (?1, ?2, ?3)",
        [p, org, f.id],
    )
    .unwrap();
    let err = lookups::delete(&conn, f.id, true).unwrap_err();
    assert!(matches!(err, CoreError::Validation(ref m) if m == "Funktion wird noch verwendet"));
}

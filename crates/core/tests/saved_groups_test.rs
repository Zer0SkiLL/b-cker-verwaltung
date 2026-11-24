mod common;
use common::test_conn;
use verwaltung_core::query::filter::{Filter, MembershipStatus};
use verwaltung_core::query::saved_groups;

#[test]
fn saved_group_roundtrip_and_overwrite() {
    let conn = test_conn();
    let f1 = Filter {
        label_ids: vec![1, 2],
        ..Default::default()
    };
    let id = saved_groups::save(&conn, "Alle Prüfungsexperten", &f1).unwrap();
    assert_eq!(saved_groups::load(&conn, id).unwrap(), f1);
    let f2 = Filter {
        membership: MembershipStatus::Active,
        ..Default::default()
    };
    let id2 = saved_groups::save(&conn, "Alle Prüfungsexperten", &f2).unwrap();
    assert_eq!(id, id2);
    assert_eq!(saved_groups::load(&conn, id).unwrap(), f2);
    assert_eq!(
        saved_groups::list(&conn).unwrap(),
        vec![(id, "Alle Prüfungsexperten".to_string())]
    );
    saved_groups::delete(&conn, id).unwrap();
    assert!(saved_groups::list(&conn).unwrap().is_empty());
}

#[test]
fn rename_group_changes_name() {
    let conn = test_conn();
    let id = saved_groups::save(&conn, "Vorstand", &Filter::default()).unwrap();
    saved_groups::rename(&conn, id, " Vorstand 2027 ").unwrap();
    assert_eq!(
        saved_groups::list(&conn).unwrap(),
        vec![(id, "Vorstand 2027".to_string())]
    );
}

#[test]
fn rename_group_to_existing_name_is_refused() {
    let conn = test_conn();
    saved_groups::save(&conn, "A", &Filter::default()).unwrap();
    let b = saved_groups::save(&conn, "B", &Filter::default()).unwrap();
    let err = saved_groups::rename(&conn, b, "A").unwrap_err();
    assert_eq!(err.to_string(), "Name ist bereits vergeben");
}

#[test]
fn rename_unknown_group_not_found() {
    let conn = test_conn();
    assert!(matches!(
        saved_groups::rename(&conn, 99, "X"),
        Err(verwaltung_core::error::CoreError::NotFound { .. })
    ));
}

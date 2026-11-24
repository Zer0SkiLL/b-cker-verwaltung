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

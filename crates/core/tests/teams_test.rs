mod common;
use common::{count, raw_person, test_conn};
use verwaltung_core::repo::teams::{self, TeamMember};

#[test]
fn team_crud_and_list() {
    let conn = test_conn();
    let t = teams::create(&conn, " Prüfungen ", Some("QV")).unwrap();
    assert_eq!(t.name, "Prüfungen");
    let u = teams::update(&conn, t.id, "Prüfungswesen", None).unwrap();
    assert_eq!(u.description, None);
    assert_eq!(teams::list(&conn).unwrap(), vec![u]);
}

#[test]
fn team_add_member_twice_updates_role() {
    let conn = test_conn();
    let t = teams::create(&conn, "Prüfungen", None).unwrap();
    let p = raw_person(&conn, "Muster");
    teams::add_member(&conn, t.id, p, None).unwrap();
    teams::add_member(&conn, t.id, p, Some("Leitung")).unwrap();
    assert_eq!(teams::members(&conn, t.id).unwrap(), vec![TeamMember { person_id: p, role: Some("Leitung".into()) }]);
    teams::remove_member(&conn, t.id, p).unwrap();
    assert!(teams::members(&conn, t.id).unwrap().is_empty());
}

#[test]
fn deleting_team_removes_members_not_persons() {
    let conn = test_conn();
    let t = teams::create(&conn, "Prüfungen", None).unwrap();
    let p = raw_person(&conn, "Muster");
    teams::add_member(&conn, t.id, p, None).unwrap();
    teams::delete(&conn, t.id).unwrap();
    assert_eq!(count(&conn, "team_members"), 0);
    assert_eq!(count(&conn, "persons"), 1);
}

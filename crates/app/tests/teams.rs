mod common;

use common::fresh_session;
use verwaltung_app::commands::teams::{
    add_team_member, create_team, delete_team, list_teams, remove_team_member, team_members,
    update_team,
};
use verwaltung_core::repo::persons::{self, PersonInput};

fn person(s: &mut verwaltung_app::session::Session, last: &str, first: &str) -> i64 {
    let input = PersonInput {
        last_name: last.into(),
        first_name: Some(first.into()),
        ..Default::default()
    };
    s.write(|c| persons::create(c, &input)).unwrap().id
}

#[test]
fn members_show_names_and_roles() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let team = create_team(&mut s, "Prüfungen", Some("Lehrabschluss")).unwrap();
    let hans = person(&mut s, "Muster", "Hans");
    add_team_member(&mut s, team.id, hans, Some("Leitung")).unwrap();
    let members = team_members(&s, team.id).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].name, "Hans Muster");
    assert_eq!(members[0].role.as_deref(), Some("Leitung"));
}

#[test]
fn add_member_twice_updates_role() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let team = create_team(&mut s, "GV", None).unwrap();
    let eva = person(&mut s, "Keller", "Eva");
    add_team_member(&mut s, team.id, eva, Some("Mitglied")).unwrap();
    add_team_member(&mut s, team.id, eva, Some("Leitung")).unwrap();
    let members = team_members(&s, team.id).unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].role.as_deref(), Some("Leitung"));
    remove_team_member(&mut s, team.id, eva).unwrap();
    assert!(team_members(&s, team.id).unwrap().is_empty());
}

#[test]
fn summary_counts_members() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let team = create_team(&mut s, "Vorstand", None).unwrap();
    let a = person(&mut s, "A", "Anna");
    let b = person(&mut s, "B", "Ben");
    add_team_member(&mut s, team.id, a, None).unwrap();
    add_team_member(&mut s, team.id, b, None).unwrap();
    let renamed = update_team(&mut s, team.id, "Vorstand 2027", None).unwrap();
    assert_eq!(renamed.name, "Vorstand 2027");
    let list = list_teams(&s).unwrap();
    assert_eq!(list[0].member_count, 2);
    delete_team(&mut s, team.id).unwrap();
    assert!(list_teams(&s).unwrap().is_empty());
}

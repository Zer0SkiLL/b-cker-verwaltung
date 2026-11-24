mod common;
use common::{d, function, test_conn};
use rusqlite::Connection;
use verwaltung_core::model::{AppliesTo, LookupKind, MemberRef};
use verwaltung_core::query::filter::{search, EntryKind, Filter, MembershipStatus};
use verwaltung_core::repo::affiliations::{self, AffiliationInput};
use verwaltung_core::repo::assignments::{self, AssignmentInput};
use verwaltung_core::repo::memberships::{self, MembershipInput};
use verwaltung_core::repo::organisations::{self, OrganisationInput};
use verwaltung_core::repo::persons::{self, PersonInput};
use verwaltung_core::repo::{labels, lookups, teams};

struct Fx {
    conn: Connection,
    vorstand: i64,
    experte: i64,
    bp: i64,
    team: i64,
    inhaber: i64,
}

fn today() -> chrono::NaiveDate {
    d(2026, 6, 1)
}

fn org(conn: &Connection, name: &str, city: &str, email: &str) -> i64 {
    let input = OrganisationInput {
        name: name.into(),
        city: Some(city.into()),
        street: Some("Hauptgasse 1".into()),
        email: Some(email.into()),
        ..Default::default()
    };
    organisations::create(conn, &input).unwrap().id
}

fn person(conn: &Connection, first: &str, last: &str) -> i64 {
    let input = PersonInput { first_name: Some(first.into()), last_name: last.into(), ..Default::default() };
    persons::create(conn, &input).unwrap().id
}

fn label(conn: &Connection, label_id: i64, target: MemberRef) {
    assignments::assign(conn, &AssignmentInput { label_id, target, assigned_on: None, note: None }).unwrap();
}

fn member(conn: &Connection, m: MemberRef, left: Option<chrono::NaiveDate>) {
    memberships::create(conn, &MembershipInput { member: m, category_id: None, joined_on: None, left_on: left }).unwrap();
}

fn link(conn: &Connection, p: i64, o: i64, f: i64, to: Option<chrono::NaiveDate>) {
    let input = AffiliationInput { person_id: p, organisation_id: o, function_id: f, valid_from: None, valid_to: to };
    affiliations::create(conn, &input).unwrap();
}

fn fixture() -> Fx {
    let conn = test_conn();
    let a = org(&conn, "Bäckerei Müller", "Zürich", "info@mueller.ch");
    let b = org(&conn, "Confiserie Beispiel", "Bern", "info@beispiel.ch");
    let (hans, anna, peter, eva) = (
        person(&conn, "Hans", "Müller"),
        person(&conn, "Anna", "Beispiel"),
        person(&conn, "Peter", "Alt"),
        person(&conn, "Eva", "Privat"),
    );
    let inhaber = function(&conn, "Inhaber");
    let mitarbeiter = function(&conn, "Mitarbeiter");
    link(&conn, hans, a, inhaber, None);
    link(&conn, anna, b, mitarbeiter, None);
    link(&conn, peter, a, inhaber, Some(d(2020, 12, 31)));
    let vorstand = labels::find_or_create_label(&conn, "Verbandsfunktion", AppliesTo::Person, "Vorstand").unwrap().id;
    let experte = labels::find_or_create_label(&conn, "Verbandsfunktion", AppliesTo::Person, "Prüfungsexperte").unwrap().id;
    let bp = labels::find_or_create_label(&conn, "Lehrgang", AppliesTo::Person, "Berufsprüfung").unwrap().id;
    label(&conn, vorstand, MemberRef::Person(hans));
    label(&conn, experte, MemberRef::Person(anna));
    label(&conn, experte, MemberRef::Person(peter));
    label(&conn, bp, MemberRef::Person(peter));
    let team = teams::create(&conn, "Prüfungen", None).unwrap().id;
    teams::add_member(&conn, team, hans, None).unwrap();
    member(&conn, MemberRef::Organisation(a), None);
    member(&conn, MemberRef::Organisation(b), Some(d(2020, 12, 31)));
    member(&conn, MemberRef::Person(eva), None);
    Fx { conn, vorstand, experte, bp, team, inhaber }
}

fn names(fx: &Fx, f: &Filter) -> Vec<String> {
    search(&fx.conn, f, today()).unwrap().into_iter().map(|r| r.display_name).collect()
}

#[test]
fn text_search_finds_umlaut_case_insensitive() {
    let fx = fixture();
    let found = names(&fx, &Filter { text: Some("müller".into()), ..Default::default() });
    assert_eq!(found, vec!["Bäckerei Müller", "Hans Müller"]);
}

#[test]
fn multiple_words_are_anded() {
    let fx = fixture();
    assert_eq!(names(&fx, &Filter { text: Some("MÜLLER zürich".into()), ..Default::default() }), vec!["Bäckerei Müller"]);
}

#[test]
fn labels_same_group_or() {
    let fx = fixture();
    let f = Filter { label_ids: vec![fx.vorstand, fx.experte], ..Default::default() };
    assert_eq!(names(&fx, &f), vec!["Anna Beispiel", "Hans Müller", "Peter Alt"]);
}

#[test]
fn labels_different_groups_and() {
    let fx = fixture();
    let f = Filter { label_ids: vec![fx.experte, fx.bp], ..Default::default() };
    assert_eq!(names(&fx, &f), vec!["Peter Alt"]);
}

#[test]
fn team_filter_excludes_organisations() {
    let fx = fixture();
    assert_eq!(names(&fx, &Filter { team_ids: vec![fx.team], ..Default::default() }), vec!["Hans Müller"]);
}

#[test]
fn function_filter_ignores_past_affiliations() {
    let fx = fixture();
    assert_eq!(names(&fx, &Filter { function_ids: vec![fx.inhaber], ..Default::default() }), vec!["Hans Müller"]);
}

#[test]
fn active_membership_via_organisation_counts_for_person() {
    let fx = fixture();
    let f = Filter { kinds: vec![EntryKind::Person], membership: MembershipStatus::Active, ..Default::default() };
    assert_eq!(names(&fx, &f), vec!["Eva Privat", "Hans Müller"]);
}

#[test]
fn exited_members_only() {
    let fx = fixture();
    let f = Filter { membership: MembershipStatus::Exited, ..Default::default() };
    assert_eq!(names(&fx, &f), vec!["Anna Beispiel", "Confiserie Beispiel"]);
}

#[test]
fn org_type_and_cantonal_filters_match_org_and_current_persons() {
    let fx = fixture();
    let typ = lookups::create(&fx.conn, LookupKind::OrgType, "Mitgliedsbetrieb").unwrap().id;
    let be = lookups::create(&fx.conn, LookupKind::CantonalAssociation, "Bern").unwrap().id;
    fx.conn.execute("UPDATE organisations SET org_type_id = ?1, cantonal_association_id = ?2 WHERE name = 'Bäckerei Müller'", [typ, be]).unwrap();
    assert_eq!(names(&fx, &Filter { org_type_ids: vec![typ], ..Default::default() }), vec!["Bäckerei Müller", "Hans Müller"]);
    assert_eq!(names(&fx, &Filter { cantonal_ids: vec![be], kinds: vec![EntryKind::Organisation], ..Default::default() }), vec!["Bäckerei Müller"]);
}

#[test]
fn category_filter_matches_membership_category() {
    let fx = fixture();
    let kat = lookups::create(&fx.conn, LookupKind::MembershipCategory, "Ehrenmitglied").unwrap().id;
    fx.conn.execute("UPDATE memberships SET category_id = ?1 WHERE person_id IS NOT NULL", [kat]).unwrap();
    assert_eq!(names(&fx, &Filter { category_ids: vec![kat], ..Default::default() }), vec!["Eva Privat"]);
}

#[test]
fn person_row_shows_current_organisation_and_falls_back_to_its_contact() {
    let fx = fixture();
    let rows = search(&fx.conn, &Filter { text: Some("hans".into()), ..Default::default() }, today()).unwrap();
    let hans = &rows[0];
    assert_eq!(hans.kind, EntryKind::Person);
    assert_eq!(hans.organisation.as_deref(), Some("Bäckerei Müller"));
    assert_eq!(hans.email.as_deref(), Some("info@mueller.ch"));
    assert_eq!(hans.city.as_deref(), Some("Zürich"));
}

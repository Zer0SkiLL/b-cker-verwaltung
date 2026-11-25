mod common;
use calamine::{open_workbook, Data, Reader, Xlsx};
use common::{d, function, test_conn};
use rusqlite::Connection;
use verwaltung_core::export_full::{full_export, SHEETS};
use verwaltung_core::model::{AppliesTo, MemberRef};
use verwaltung_core::repo::affiliations::{self, AffiliationInput};
use verwaltung_core::repo::assignments::{self, AssignmentInput};
use verwaltung_core::repo::memberships::{self, MembershipInput};
use verwaltung_core::repo::organisations::{self, OrganisationInput};
use verwaltung_core::repo::persons::{self, PersonInput};
use verwaltung_core::repo::{labels, teams};

fn filled() -> Connection {
    let conn = test_conn();
    let org = organisations::create(
        &conn,
        &OrganisationInput {
            name: "Bäckerei Müller".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let p = persons::create(
        &conn,
        &PersonInput {
            first_name: Some("Hans".into()),
            last_name: "Müller".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let f = function(&conn, "Inhaber");
    affiliations::create(
        &conn,
        &AffiliationInput {
            person_id: p.id,
            organisation_id: org.id,
            function_id: f,
            valid_from: Some(d(2001, 4, 1)),
            valid_to: None,
        },
    )
    .unwrap();
    memberships::create(
        &conn,
        &MembershipInput {
            member: MemberRef::Organisation(org.id),
            category_id: None,
            joined_on: Some(d(1999, 1, 1)),
            left_on: None,
        },
    )
    .unwrap();
    let l = labels::find_or_create_label(&conn, "Verbandsfunktion", AppliesTo::Person, "Vorstand")
        .unwrap();
    assignments::assign(
        &conn,
        &AssignmentInput {
            label_id: l.id,
            target: MemberRef::Person(p.id),
            assigned_on: None,
            note: None,
        },
    )
    .unwrap();
    let t = teams::create(&conn, "Prüfungen", None).unwrap();
    teams::add_member(&conn, t.id, p.id, Some("Leitung")).unwrap();
    conn
}

fn export(conn: &Connection) -> (tempfile::TempDir, Xlsx<std::io::BufReader<std::fs::File>>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gesamt.xlsx");
    full_export(conn, &path).unwrap();
    let wb = open_workbook(&path).unwrap();
    (dir, wb)
}

fn sheet(wb: &mut Xlsx<std::io::BufReader<std::fs::File>>, name: &str) -> Vec<Vec<String>> {
    let range = wb.worksheet_range(name).unwrap();
    range
        .rows()
        .map(|r| {
            r.iter()
                .map(|c| match c {
                    Data::Empty => String::new(),
                    other => other.to_string(),
                })
                .collect()
        })
        .collect()
}

#[test]
fn full_export_has_all_sheets_in_order() {
    let (_d, wb) = export(&filled());
    assert_eq!(
        wb.sheet_names(),
        vec![
            "Betriebe",
            "Personen",
            "Zugehörigkeiten",
            "Mitgliedschaften",
            "Label-Gruppen",
            "Labels",
            "Label-Zuweisungen",
            "Teams",
            "Teammitglieder",
            "Auswahllisten",
            "Gespeicherte Gruppen"
        ]
    );
    assert_eq!(SHEETS.len(), 11);
}

#[test]
fn row_counts_match_db() {
    let conn = filled();
    let (_d, mut wb) = export(&conn);
    for (name, table) in [
        ("Betriebe", "organisations"),
        ("Personen", "persons"),
        ("Zugehörigkeiten", "affiliations"),
        ("Teammitglieder", "team_members"),
        ("Auswahllisten", "lookup_values"),
    ] {
        assert_eq!(
            sheet(&mut wb, name).len() as i64 - 1,
            common::count(&conn, table),
            "{name}"
        );
        assert_eq!(sheet(&mut wb, name)[0][0], "ID");
    }
}

#[test]
fn references_resolved_to_names() {
    let (_d, mut wb) = export(&filled());
    let z = sheet(&mut wb, "Zugehörigkeiten");
    assert_eq!(
        z[0],
        vec![
            "ID",
            "Person-ID",
            "Person",
            "Betrieb-ID",
            "Betrieb",
            "Funktion",
            "Von",
            "Bis"
        ]
    );
    assert_eq!(
        &z[1][2..6],
        &["Hans Müller", "1", "Bäckerei Müller", "Inhaber"]
    );
    let m = sheet(&mut wb, "Mitgliedschaften");
    assert_eq!(
        m[0],
        vec![
            "ID",
            "Typ",
            "Mitglied-ID",
            "Mitglied",
            "Kategorie",
            "Eintritt",
            "Austritt"
        ]
    );
    assert_eq!(m[1][1], "Betrieb");
    assert_eq!(m[1][3], "Bäckerei Müller");
    let tm = sheet(&mut wb, "Teammitglieder");
    assert!(tm[1].contains(&"Prüfungen".to_string()) && tm[1].contains(&"Leitung".to_string()));
}

#[test]
fn empty_db_exports_headers_only() {
    let (_d, mut wb) = export(&test_conn());
    for name in SHEETS {
        assert_eq!(sheet(&mut wb, name).len(), 1, "{name}");
    }
}

#[test]
fn dates_swiss_format() {
    let (_d, mut wb) = export(&filled());
    assert_eq!(sheet(&mut wb, "Zugehörigkeiten")[1][6], "01.04.2001");
    assert_eq!(sheet(&mut wb, "Mitgliedschaften")[1][5], "01.01.1999");
}

#[test]
fn organisations_sheet_has_number_columns() {
    let (_d, mut wb) = export(&filled());
    let rows = sheet(&mut wb, "Betriebe");
    assert!(
        rows[0].contains(&"Handelsregister-Nr".to_string()),
        "{:?}",
        rows[0]
    );
    assert!(rows[0].contains(&"Kostenstellen-Nr".to_string()));
}

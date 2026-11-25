//! Import nach dem Aufbau der echten Mitgliederliste (Kopfzeilen wie in der Datei des Vereins).
mod common;
use chrono::NaiveDate;
use common::{count, raw_org, test_conn};
use rust_xlsxwriter::Workbook;
use verwaltung_core::error::CoreError;
use verwaltung_core::import::apply::apply;
use verwaltung_core::import::mapping::{Field, Mapping};
use verwaltung_core::import::plan::{plan, reconcile, ImportPlan};
use verwaltung_core::import::sheet::{read_first_sheet, Cell, Sheet, SheetRow};
use verwaltung_core::model::LookupKind;
use verwaltung_core::query::filter::{search, Filter};
use verwaltung_core::repo::{defaults, labels, lookups};

/// Kopfzeilen wie in der Mitgliederliste des Vereins, inkl. drei leerer Spalten am Ende.
const HEADERS: [&str; 29] = [
    "OBC-ID",
    "Kt Verband",
    "Firma",
    "Name",
    "Vorname",
    "Zusatz/Inhaber (Handelsregister)",
    "Adresszusatz",
    "Adresse",
    "PLZ",
    "Ort",
    "Kategorie",
    "Mail",
    "Home",
    "Lehrbetrieb",
    "Fr Bäck",
    "Fr Conf",
    "DH",
    "Besonderes",
    "Eintritt",
    "Austritt",
    "Filialen",
    "Telefon",
    "Mobiltelefon",
    "Kostenstellen-Nr",
    "Mandat",
    "Handelsregister-Nr",
    "",
    "",
    "",
];

fn headers() -> Vec<String> {
    HEADERS.iter().map(|h| h.to_string()).collect()
}

fn row(line: usize, values: &[(&str, Cell)]) -> SheetRow {
    let cells = HEADERS
        .iter()
        .map(|h| {
            values
                .iter()
                .find(|(k, _)| k == h)
                .map(|(_, c)| c.clone())
                .unwrap_or(Cell::Empty)
        })
        .collect();
    SheetRow { line, cells }
}

fn t(s: &str) -> Cell {
    Cell::Text(s.into())
}

fn planned(rows: Vec<SheetRow>) -> ImportPlan {
    let sheet = Sheet {
        headers: headers(),
        rows,
    };
    plan(&sheet, &Mapping::default_for(&headers()))
}

fn messages(p: &ImportPlan) -> Vec<String> {
    p.report.iter().map(|r| r.message.clone()).collect()
}

fn firm(line: usize, name: &str, extra: &[(&str, Cell)]) -> SheetRow {
    // Zusätzliche Werte zuerst: `row` nimmt den ersten Wert einer Spalte.
    let mut values: Vec<(&str, Cell)> = extra.to_vec();
    values.extend([
        ("Firma", t(name)),
        ("Adresse", t("Hauptstrasse 1")),
        ("PLZ", Cell::Number(9000.0)),
        ("Ort", t("St. Gallen")),
    ]);
    row(line, &values)
}

#[test]
fn default_mapping_matches_real_headers() {
    let m = Mapping::default_for(&headers());
    let field = |h: &str| {
        m.columns
            .iter()
            .find(|(k, _)| k == h)
            .map(|(_, f)| *f)
            .unwrap()
    };
    assert_eq!(field("OBC-ID"), Field::LegacyId);
    assert_eq!(field("Firma"), Field::Company);
    assert_eq!(field("Name"), Field::LastName);
    assert_eq!(field("Vorname"), Field::FirstName);
    assert_eq!(field("Zusatz/Inhaber (Handelsregister)"), Field::Addition);
    assert_eq!(field("Adresszusatz"), Field::Ignore);
    assert_eq!(field("Kostenstellen-Nr"), Field::CostCenterNo);
    assert_eq!(field("Mandat"), Field::Mandate);
    assert_eq!(field("Handelsregister-Nr"), Field::CommercialRegisterNo);
    assert_eq!(m.columns[26].1, Field::Ignore);
    assert_eq!(m.columns.len(), 29);
}

#[test]
fn same_company_same_address_is_merged() {
    let p = planned(vec![
        firm(
            2,
            "Bäckerei Muster",
            &[("Name", t("Muster")), ("Vorname", t("Anna"))],
        ),
        firm(
            3,
            "Bäckerei Muster",
            &[("Name", t("Muster")), ("Vorname", t("Beat"))],
        ),
    ]);
    assert_eq!(p.organisations.len(), 1);
    let org = &p.organisations[0];
    assert_eq!(org.lines, vec![2, 3]);
    assert_eq!(org.persons.len(), 2);
    assert!(messages(&p).contains(&"2 Zeilen zu einem Betrieb zusammengefasst".to_string()));
}

#[test]
fn merge_ignores_case_and_spaces() {
    let p = planned(vec![
        firm(2, "Bäckerei  Muster", &[]),
        firm(3, "bäckerei muster ", &[]),
    ]);
    assert_eq!(p.organisations.len(), 1);
}

#[test]
fn same_company_other_address_stays_separate() {
    let p = planned(vec![
        firm(2, "Bäckerei Muster", &[]),
        firm(3, "Bäckerei Muster", &[("Adresse", t("Seeweg 2"))]),
    ]);
    assert_eq!(p.organisations.len(), 2);
}

#[test]
fn deviating_contact_goes_to_person() {
    let p = planned(vec![
        firm(
            2,
            "Beck AG",
            &[
                ("OBC-ID", Cell::Number(12.0)),
                ("Mail", t("info@beck.ch")),
                ("Telefon", t("071 111 11 11")),
            ],
        ),
        firm(
            3,
            "Beck AG",
            &[
                ("OBC-ID", Cell::Number(77.0)),
                ("Name", t("Keller")),
                ("Mail", t("keller@beck.ch")),
                ("Telefon", t("071 222 22 22")),
                ("Besonderes", t("Prüfungsexperte")),
            ],
        ),
    ]);
    let org = &p.organisations[0];
    assert_eq!(org.input.email.as_deref(), Some("info@beck.ch"));
    assert_eq!(org.input.legacy_id.as_deref(), Some("12"));
    let person = &org.persons[0].input;
    assert_eq!(person.email.as_deref(), Some("keller@beck.ch"));
    assert_eq!(person.phone.as_deref(), Some("071 222 22 22"));
    let notes = person.notes.as_deref().unwrap();
    assert!(notes.contains("OBC-ID: 77"), "{notes}");
    assert!(notes.contains("Prüfungsexperte"), "{notes}");
}

#[test]
fn deviating_category_is_reported() {
    let p = planned(vec![
        firm(2, "Beck AG", &[("Kategorie", t("Mitglied mit Geschäft"))]),
        firm(3, "Beck AG", &[("Kategorie", t("Kein Mitglied"))]),
    ]);
    assert_eq!(
        p.organisations[0].category.as_deref(),
        Some("Mitglied mit Geschäft")
    );
    assert!(
        messages(&p).contains(&"Abweichende Angabe für Kategorie (erste Zeile 2 gilt)".to_string())
    );
}

#[test]
fn deviation_without_person_goes_to_org_notes() {
    let p = planned(vec![
        firm(2, "Beck AG", &[("Mail", t("info@beck.ch"))]),
        firm(3, "Beck AG", &[("Mail", t("büro@beck.ch"))]),
    ]);
    let notes = p.organisations[0].input.notes.as_deref().unwrap();
    assert!(notes.contains("Zeile 3"), "{notes}");
    assert!(notes.contains("büro@beck.ch"), "{notes}");
}

#[test]
fn empty_values_filled_from_later_rows() {
    let p = planned(vec![
        firm(2, "Beck AG", &[]),
        firm(
            3,
            "Beck AG",
            &[
                ("Kategorie", t("Mitglied mit Geschäft")),
                ("Home", t("beck.ch")),
            ],
        ),
    ]);
    let org = &p.organisations[0];
    assert_eq!(org.category.as_deref(), Some("Mitglied mit Geschäft"));
    assert_eq!(org.input.website.as_deref(), Some("beck.ch"));
}

#[test]
fn row_without_company_is_individual() {
    let p = planned(vec![row(
        2,
        &[
            ("Name", t("Keller")),
            ("Vorname", t("Hans")),
            ("Adresse", t("Dorf 3")),
            ("PLZ", Cell::Number(7000.0)),
            ("Ort", t("Chur")),
            ("Mail", t("hans@keller.ch")),
            ("Kategorie", t("Mitglied ohne Geschäft")),
            (
                "Eintritt",
                Cell::Date(NaiveDate::from_ymd_opt(2001, 5, 1).unwrap()),
            ),
        ],
    )]);
    assert!(p.organisations.is_empty());
    let ind = &p.individuals[0];
    assert_eq!(ind.person.input.last_name, "Keller");
    assert_eq!(ind.person.input.city.as_deref(), Some("Chur"));
    assert_eq!(ind.person.input.email.as_deref(), Some("hans@keller.ch"));
    assert_eq!(ind.category.as_deref(), Some("Mitglied ohne Geschäft"));
    assert_eq!(ind.joined, NaiveDate::from_ymd_opt(2001, 5, 1));
}

#[test]
fn individual_flags_are_ignored_with_note() {
    let p = planned(vec![row(
        2,
        &[("Name", t("Keller")), ("Fr Bäck", Cell::Bool(true))],
    )]);
    assert_eq!(p.individuals.len(), 1);
    assert!(messages(&p).contains(&"Fachrichtung bei Einzelmitglied ignoriert".to_string()));
}

#[test]
fn first_name_only_becomes_last_name_with_note() {
    let p = planned(vec![firm(2, "Beck AG", &[("Vorname", t("Anna"))])]);
    let person = &p.organisations[0].persons[0].input;
    assert_eq!(person.last_name, "Anna");
    assert_eq!(person.first_name, None);
    assert!(messages(&p).contains(&"Nur Vorname angegeben, als Nachname übernommen".to_string()));
}

#[test]
fn row_without_company_and_name_is_skipped() {
    let p = planned(vec![row(2, &[("Ort", t("Bern"))])]);
    assert_eq!(p.skipped.len(), 1);
    assert_eq!(
        p.skipped[0].reason,
        "Weder Firma noch Name, Zeile übersprungen"
    );
}

#[test]
fn mandate_split_to_person() {
    let p = planned(vec![firm(
        2,
        "Beck AG",
        &[("Name", t("Keller")), ("Mandat", t("Präsident; Vorstand,"))],
    )]);
    assert_eq!(
        p.organisations[0].persons[0].mandates,
        vec!["Präsident", "Vorstand"]
    );
    assert!(p.organisations[0].mandates.is_empty());
}

#[test]
fn mandate_without_person_to_org() {
    let p = planned(vec![firm(2, "Beck AG", &[("Mandat", t("Revisor"))])]);
    assert_eq!(p.organisations[0].mandates, vec!["Revisor"]);
}

#[test]
fn numbers_and_addition_land_in_org() {
    let p = planned(vec![firm(
        2,
        "Beck AG",
        &[
            ("Zusatz/Inhaber (Handelsregister)", t("Inh. Hans Beck")),
            ("Kostenstellen-Nr", Cell::Number(4711.0)),
            ("Handelsregister-Nr", t("CHE-123.456.789")),
            ("Adresszusatz", t("c/o Irgendwer")),
        ],
    )]);
    let o = &p.organisations[0].input;
    assert_eq!(o.addition.as_deref(), Some("Inh. Hans Beck"));
    assert_eq!(o.cost_center_no.as_deref(), Some("4711"));
    assert_eq!(o.commercial_register_no.as_deref(), Some("CHE-123.456.789"));
    assert_eq!(o.po_box, None);
}

#[test]
fn reconcile_ok_for_mixed_plan() {
    let p = planned(vec![
        firm(2, "Beck AG", &[("Name", t("A"))]),
        firm(3, "Beck AG", &[("Name", t("B"))]),
        firm(4, "Brot GmbH", &[]),
        row(5, &[("Name", t("Einzeln"))]),
        row(6, &[("Ort", t("leer"))]),
    ]);
    assert_eq!(p.data_rows, 5);
    assert_eq!(reconcile(&p), Ok(()));
}

#[test]
fn reconcile_detects_mismatch() {
    let mut p = planned(vec![firm(2, "Beck AG", &[]), row(3, &[("Name", t("X"))])]);
    p.individuals.clear();
    assert!(reconcile(&p)
        .unwrap_err()
        .starts_with("Der Abgleich der Zeilen geht nicht auf"));
}

// ---------- Schreiben ----------

fn seeded() -> rusqlite::Connection {
    let conn = test_conn();
    defaults::seed_new_file(&conn).unwrap();
    conn
}

fn names(conn: &rusqlite::Connection, kind: LookupKind) -> Vec<String> {
    lookups::list(conn, kind)
        .unwrap()
        .into_iter()
        .map(|v| v.name)
        .collect()
}

#[test]
fn kein_mitglied_matches_default_category() {
    let mut conn = seeded();
    let p = planned(vec![
        firm(2, "Beck AG", &[("Kategorie", t("kein Mitglied"))]),
        firm(3, "Brot AG", &[("Kategorie", t("MITGLIED MIT GESCHÄFT"))]),
    ]);
    apply(&mut conn, &p).unwrap();
    assert_eq!(names(&conn, LookupKind::MembershipCategory).len(), 4);
}

#[test]
fn persons_get_kontaktperson_and_individuals_membership() {
    let mut conn = seeded();
    let p = planned(vec![
        firm(2, "Beck AG", &[("Name", t("Keller"))]),
        row(
            3,
            &[
                ("Name", t("Einzeln")),
                ("Kategorie", t("Mitglied ohne Geschäft")),
            ],
        ),
    ]);
    let summary = apply(&mut conn, &p).unwrap();
    assert_eq!(
        (summary.organisations, summary.persons, summary.individuals),
        (1, 1, 1)
    );
    assert_eq!(names(&conn, LookupKind::Function), vec!["Kontaktperson"]);
    let person_memberships: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memberships WHERE person_id IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(person_memberships, 1);
    assert_eq!(count(&conn, "memberships"), 2);
}

#[test]
fn mandate_labels_on_person() {
    let mut conn = seeded();
    let p = planned(vec![firm(
        2,
        "Beck AG",
        &[("Name", t("Keller")), ("Mandat", t("Präsident, Vorstand"))],
    )]);
    apply(&mut conn, &p).unwrap();
    let group = labels::list_groups(&conn)
        .unwrap()
        .into_iter()
        .find(|g| g.name == "Mandat")
        .unwrap();
    assert_eq!(labels::list_labels(&conn, group.id).unwrap().len(), 2);
    let on_person: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM label_assignments WHERE person_id IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(on_person, 2);
}

#[test]
fn apply_into_non_empty_db_is_rejected() {
    let mut conn = test_conn();
    raw_org(&conn, "Schon da");
    let p = planned(vec![firm(2, "Beck", &[])]);
    assert!(matches!(
        apply(&mut conn, &p),
        Err(CoreError::DatabaseNotEmpty)
    ));
}

#[test]
fn apply_is_atomic() {
    let mut conn = test_conn();
    let mut p = planned(vec![firm(2, "Gut", &[]), firm(3, "Schlecht", &[])]);
    p.organisations[1].cantonal = Some("   ".into());
    assert!(apply(&mut conn, &p).is_err());
    assert_eq!(count(&conn, "organisations"), 0);
    assert_eq!(count(&conn, "lookup_values"), 0);
}

#[test]
fn end_to_end_import_then_filter() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mitglieder.xlsx");
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    for (c, h) in HEADERS.iter().enumerate() {
        ws.write_string(0, c as u16, *h).unwrap();
    }
    let col = |name: &str| HEADERS.iter().position(|h| *h == name).unwrap() as u16;
    ws.write_string(1, col("Firma"), "Confiserie Süss").unwrap();
    ws.write_boolean(1, col("Lehrbetrieb"), true).unwrap();
    ws.write_boolean(1, col("Fr Conf"), true).unwrap();
    ws.write_string(2, col("Firma"), "Bäckerei Brot").unwrap();
    ws.write_boolean(2, col("Fr Bäck"), true).unwrap();
    wb.save(&path).unwrap();

    let sheet = read_first_sheet(&path).unwrap();
    let p = plan(&sheet, &Mapping::default_for(&sheet.headers));
    let mut conn = test_conn();
    apply(&mut conn, &p).unwrap();
    let conf = labels::find_or_create_label(
        &conn,
        "Ausbildung Fachrichtung",
        verwaltung_core::model::AppliesTo::Organisation,
        "Fr Conf",
    )
    .unwrap();
    let today = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
    let rows = search(
        &conn,
        &Filter {
            label_ids: vec![conf.id],
            ..Default::default()
        },
        today,
    )
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].display_name, "Confiserie Süss");
}

#[test]
fn individual_keeps_business_fields_in_notes() {
    let p = planned(vec![row(
        2,
        &[
            ("Name", t("Keller")),
            ("Kt Verband", t("ZH")),
            ("Zusatz/Inhaber (Handelsregister)", t("pensioniert")),
            ("Home", t("keller.ch")),
            ("Handelsregister-Nr", t("CHE-1")),
        ],
    )]);
    let notes = p.individuals[0].person.input.notes.clone().unwrap();
    for part in [
        "Kantonalverband: ZH",
        "Zusatz: pensioniert",
        "Website: keller.ch",
        "Handelsregister-Nr: CHE-1",
    ] {
        assert!(notes.contains(part), "{part} fehlt in {notes}");
    }
}

#[test]
fn same_company_other_address_is_noted() {
    let p = planned(vec![
        firm(2, "Bäckerei Muster", &[]),
        firm(3, "Bäckerei Muster", &[("Adresse", t("Seeweg 2"))]),
    ]);
    assert!(messages(&p)
        .contains(&"Gleicher Firmenname wie Zeile 2, andere Adresse: eigener Betrieb".to_string()));
}

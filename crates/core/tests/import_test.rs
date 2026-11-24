mod common;
use common::{count, raw_org, test_conn};
use rust_xlsxwriter::Workbook;
use verwaltung_core::error::CoreError;
use verwaltung_core::import::apply::apply;
use verwaltung_core::import::mapping::{plan, Field, ImportPlan, Mapping, PlannedRow};
use verwaltung_core::import::sheet::{read_first_sheet, Cell, Sheet, SheetRow};
use verwaltung_core::query::filter::{search, Filter};
use verwaltung_core::repo::labels;
use verwaltung_core::repo::organisations::OrganisationInput;

const HEADERS: [&str; 23] = [
    "ID1", "Anzahl", "Kt Verband", "Name/Firma", "Vorname", "Zusatz", "Adresse", "PLZ", "Ort", "Kategorie",
    "Mail", "Home", "Lehrbetrieb", "Fr Bäck", "Fr Conf", "DH", "Besonderes", "Eintritt", "Austritt",
    "Filialen", "Telefon", "Mobiltelefon", "Postfach",
];

fn headers() -> Vec<String> {
    HEADERS.iter().map(|h| h.to_string()).collect()
}

/// Baut eine Zeile aus (Kopfzeile, Zelle)-Paaren; alle anderen Spalten bleiben leer.
fn row(line: usize, values: &[(&str, Cell)]) -> SheetRow {
    let cells = HEADERS
        .iter()
        .map(|h| values.iter().find(|(k, _)| k == h).map(|(_, c)| c.clone()).unwrap_or(Cell::Empty))
        .collect();
    SheetRow { line, cells }
}

fn t(s: &str) -> Cell {
    Cell::Text(s.into())
}

fn planned(rows: Vec<SheetRow>) -> ImportPlan {
    let sheet = Sheet { headers: headers(), rows };
    plan(&sheet, &Mapping::default_for(&headers()))
}

fn messages(p: &ImportPlan) -> Vec<String> {
    p.report.iter().map(|r| r.message.clone()).collect()
}

#[test]
fn default_mapping_matches_spec_headers() {
    let m = Mapping::default_for(&headers());
    assert_eq!(m.columns.len(), 23);
    assert!(m.columns.iter().all(|(_, f)| *f != Field::Ignore));
    let lower: Vec<String> = HEADERS.iter().map(|h| format!(" {} ", h.to_lowercase())).collect();
    assert!(Mapping::default_for(&lower).columns.iter().all(|(_, f)| *f != Field::Ignore));
    assert_eq!(Mapping::default_for(&["Unbekannt".to_string()]).columns[0].1, Field::Ignore);
}

#[test]
fn row_without_first_name_is_organisation_only() {
    let p = planned(vec![row(2, &[("Name/Firma", t("Bäckerei Beck AG")), ("Zusatz", t("Filiale Bahnhof")), ("PLZ", Cell::Number(3000.0))])]);
    let r = &p.rows[0];
    assert!(r.person.is_none());
    assert_eq!(r.organisation.name, "Bäckerei Beck AG");
    assert_eq!(r.organisation.addition.as_deref(), Some("Filiale Bahnhof"));
    assert_eq!(r.organisation.zip.as_deref(), Some("3000"));
}

#[test]
fn row_with_first_name_creates_person_org_and_inhaber() {
    let p = planned(vec![
        row(2, &[("Name/Firma", t("Muster")), ("Vorname", t("Hans")), ("Zusatz", t("Bäckerei Muster")), ("Mobiltelefon", t("079 123 45 67"))]),
        row(3, &[("Name/Firma", t("Muster")), ("Vorname", t("Hans"))]),
    ]);
    let first = &p.rows[0];
    let person = first.person.as_ref().unwrap();
    assert_eq!(person.last_name, "Muster");
    assert_eq!(person.first_name.as_deref(), Some("Hans"));
    assert_eq!(person.mobile.as_deref(), Some("079 123 45 67"));
    assert_eq!(first.organisation.name, "Bäckerei Muster");
    assert_eq!(first.organisation.addition, None);
    assert_eq!(p.rows[1].organisation.name, "Hans Muster");
}

#[test]
fn fachrichtung_flags_become_labels() {
    let p = planned(vec![
        row(2, &[("Name/Firma", t("A")), ("Lehrbetrieb", t("x")), ("Fr Bäck", t("X")), ("DH", Cell::Number(1.0))]),
        row(3, &[("Name/Firma", t("B")), ("Lehrbetrieb", t("ja"))]),
    ]);
    assert_eq!(p.rows[0].flags, vec!["Fr Bäck", "DH"]);
    assert!(p.rows[1].flags.is_empty());
    assert!(p.report.iter().any(|r| r.line == 3 && r.message == "Lehrbetrieb ohne Fachrichtung"));
}

#[test]
fn messy_row_produces_report_not_failure() {
    let p = planned(vec![row(
        5,
        &[
            ("Name/Firma", t("  Beck  ")),
            ("Fr Conf", t("vielleicht")),
            ("Eintritt", t("31.02.2019")),
            ("Austritt", t("2021")),
            ("Anzahl", t("viele")),
            ("PLZ", Cell::Number(8000.0)),
        ],
    )]);
    let r = &p.rows[0];
    assert_eq!(r.organisation.name, "Beck");
    assert_eq!(r.organisation.zip.as_deref(), Some("8000"));
    assert_eq!(r.organisation.apprentice_count, None);
    assert_eq!(r.joined, None);
    assert_eq!(r.left, Some(chrono::NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));
    assert!(r.flags.is_empty());
    assert_eq!(p.report.len(), 4, "{:?}", messages(&p));
    assert!(p.report.iter().all(|e| e.line == 5));
    assert!(messages(&p).contains(&"Nur Jahr angegeben, 01.01. angenommen".to_string()));
}

#[test]
fn rows_without_name_are_skipped_and_reported() {
    let p = planned(vec![row(2, &[("Ort", t("Bern"))])]);
    assert!(p.rows.is_empty());
    assert_eq!(p.report[0].line, 2);
}

#[test]
fn duplicate_firm_names_reported() {
    let p = planned(vec![row(2, &[("Name/Firma", t("Beck"))]), row(3, &[("Name/Firma", t("BECK"))])]);
    assert_eq!(p.rows.len(), 2);
    let dup: Vec<usize> = p.report.iter().filter(|r| r.message.starts_with("Doppelter Firmenname")).map(|r| r.line).collect();
    assert_eq!(dup, vec![2, 3]);
}

#[test]
fn apply_into_non_empty_db_is_rejected() {
    let mut conn = test_conn();
    raw_org(&conn, "Schon da");
    let p = planned(vec![row(2, &[("Name/Firma", t("Beck"))])]);
    assert!(matches!(apply(&mut conn, &p), Err(CoreError::DatabaseNotEmpty)));
}

#[test]
fn apply_is_atomic() {
    let mut conn = test_conn();
    let good = PlannedRow {
        line: 2,
        organisation: OrganisationInput { name: "Gut".into(), ..Default::default() },
        person: None,
        category: None,
        cantonal: None,
        flags: vec![],
        joined: None,
        left: None,
    };
    let bad = PlannedRow { line: 3, cantonal: Some("   ".into()), ..good.clone() };
    let p = ImportPlan { rows: vec![good, bad], report: vec![] };
    assert!(apply(&mut conn, &p).is_err());
    assert_eq!(count(&conn, "organisations"), 0);
    assert_eq!(count(&conn, "lookup_values"), 0);
}

#[test]
fn apply_creates_memberships_lookups_and_affiliation() {
    let mut conn = test_conn();
    let p = planned(vec![row(
        2,
        &[
            ("Name/Firma", t("Muster")),
            ("Vorname", t("Hans")),
            ("Kategorie", t("Aktiv")),
            ("Kt Verband", t("Bern")),
            ("Eintritt", t("01.01.1999")),
        ],
    )]);
    let summary = apply(&mut conn, &p).unwrap();
    assert_eq!((summary.organisations, summary.persons), (1, 1));
    assert_eq!(count(&conn, "memberships"), 1);
    assert_eq!(count(&conn, "affiliations"), 1);
    let f: String = conn
        .query_row("SELECT l.name FROM affiliations a JOIN lookup_values l ON l.id = a.function_id", [], |r| r.get(0))
        .unwrap();
    assert_eq!(f, "Inhaber");
    let joined: String = conn.query_row("SELECT joined_on FROM memberships", [], |r| r.get(0)).unwrap();
    assert_eq!(joined, "1999-01-01");
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
    ws.write_string(1, col("Name/Firma"), "Confiserie Süss").unwrap();
    ws.write_string(1, col("Lehrbetrieb"), "x").unwrap();
    ws.write_string(1, col("Fr Conf"), "x").unwrap();
    ws.write_string(2, col("Name/Firma"), "Bäckerei Brot").unwrap();
    ws.write_string(2, col("Fr Bäck"), "x").unwrap();
    wb.save(&path).unwrap();

    let sheet = read_first_sheet(&path).unwrap();
    let p = plan(&sheet, &Mapping::default_for(&sheet.headers));
    let mut conn = test_conn();
    apply(&mut conn, &p).unwrap();
    let conf = labels::find_or_create_label(&conn, "Ausbildung Fachrichtung", verwaltung_core::model::AppliesTo::Organisation, "Fr Conf").unwrap();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let found = search(&conn, &Filter { label_ids: vec![conf.id], ..Default::default() }, today).unwrap();
    let names: Vec<String> = found.into_iter().map(|r| r.display_name).collect();
    assert_eq!(names, vec!["Confiserie Süss"]);
}

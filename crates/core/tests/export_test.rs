use calamine::{open_workbook, Data, Reader, Xlsx};
use verwaltung_core::export::{contact_table, write_csv, write_xlsx, Table};
use verwaltung_core::query::filter::{ContactRow, EntryKind};

fn rows() -> Vec<ContactRow> {
    vec![ContactRow {
        kind: EntryKind::Organisation,
        id: 1,
        display_name: "Bäckerei Müller".into(),
        organisation: None,
        street: Some("Hauptgasse 1".into()),
        zip: Some("0800".into()),
        city: Some("Zürich".into()),
        po_box: None,
        email: Some("info@mueller.ch".into()),
        phone: None,
        mobile: None,
    }]
}

#[test]
fn contact_table_headers_exact() {
    let t = contact_table(&rows());
    assert_eq!(
        t.headers,
        vec![
            "Typ",
            "Name",
            "Betrieb",
            "Adresse",
            "Postfach",
            "PLZ",
            "Ort",
            "Mail",
            "Telefon",
            "Mobiltelefon"
        ]
    );
    assert_eq!(t.rows[0][0], "Betrieb");
    assert_eq!(t.rows[0][1], "Bäckerei Müller");
}

#[test]
fn csv_has_bom_and_semicolons_and_umlauts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("liste.csv");
    write_csv(&contact_table(&rows()), &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
    let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
    assert!(text.starts_with("Typ;Name;Betrieb;"));
    assert!(text.contains("Betrieb;Bäckerei Müller;"));
}

#[test]
fn csv_quotes_field_with_semicolon() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.csv");
    let t = Table {
        headers: vec!["A".into()],
        rows: vec![vec!["x;y".into()]],
    };
    write_csv(&t, &path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"x;y\""));
}

#[test]
fn xlsx_roundtrip_with_calamine() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("liste.xlsx");
    write_xlsx(&contact_table(&rows()), &path, "Kontakte").unwrap();
    let mut wb: Xlsx<_> = open_workbook(&path).unwrap();
    let range = wb.worksheet_range("Kontakte").unwrap();
    let cells: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
    assert_eq!(cells[0][0], Data::String("Typ".into()));
    assert_eq!(cells[1][1], Data::String("Bäckerei Müller".into()));
    assert_eq!(cells[1][5], Data::String("0800".into()));
}

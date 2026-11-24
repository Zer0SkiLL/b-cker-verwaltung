use chrono::NaiveDate;
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook};
use verwaltung_core::import::sheet::{read_first_sheet, Cell};

#[test]
fn reads_headers_and_skips_empty_rows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("liste.xlsx");
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.write_string(0, 0, " Name/Firma ").unwrap();
    ws.write_string(0, 1, "PLZ").unwrap();
    ws.write_string(0, 2, "Eintritt").unwrap();
    ws.write_string(1, 0, "Bäckerei Muster").unwrap();
    ws.write_number(1, 1, 8000.0).unwrap();
    let fmt = Format::new().set_num_format("dd.mm.yyyy");
    ws.write_datetime_with_format(1, 2, ExcelDateTime::from_ymd(2019, 3, 5).unwrap(), &fmt).unwrap();
    // Zeile 3 (Index 2) bleibt leer
    ws.write_string(3, 0, "Confiserie").unwrap();
    wb.save(&path).unwrap();

    let sheet = read_first_sheet(&path).unwrap();
    assert_eq!(sheet.headers, vec!["Name/Firma", "PLZ", "Eintritt"]);
    assert_eq!(sheet.rows.len(), 2);
    assert_eq!(sheet.rows[0].line, 2);
    assert_eq!(sheet.rows[0].cells[0], Cell::Text("Bäckerei Muster".into()));
    assert_eq!(sheet.rows[0].cells[1], Cell::Number(8000.0));
    assert_eq!(sheet.rows[0].cells[2], Cell::Date(NaiveDate::from_ymd_opt(2019, 3, 5).unwrap()));
    assert_eq!(sheet.rows[1].line, 4);
    assert_eq!(sheet.rows[1].cells[0], Cell::Text("Confiserie".into()));
}

#[test]
fn missing_file_is_an_error() {
    assert!(read_first_sheet(std::path::Path::new("/nicht/vorhanden.xlsx")).is_err());
}

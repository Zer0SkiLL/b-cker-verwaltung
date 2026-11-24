//! Export von Listen als CSV (Excel-kompatibel) und XLSX.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use rust_xlsxwriter::{Format, Workbook, Worksheet};

use crate::error::{CoreError, Result};
use crate::query::filter::{ContactRow, EntryKind};

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
const CONTACT_HEADERS: [&str; 10] =
    ["Typ", "Name", "Betrieb", "Adresse", "Postfach", "PLZ", "Ort", "Mail", "Telefon", "Mobiltelefon"];

/// Tabelle aus Texten; Grundlage aller Exporte.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Kontaktliste; dient auch als Datenquelle für Word-Serienbriefe.
pub fn contact_table(rows: &[ContactRow]) -> Table {
    let text = |v: &Option<String>| v.clone().unwrap_or_default();
    Table {
        headers: CONTACT_HEADERS.iter().map(|h| h.to_string()).collect(),
        rows: rows
            .iter()
            .map(|r| {
                vec![
                    match r.kind {
                        EntryKind::Person => "Person".into(),
                        EntryKind::Organisation => "Betrieb".into(),
                    },
                    r.display_name.clone(),
                    text(&r.organisation),
                    text(&r.street),
                    text(&r.po_box),
                    text(&r.zip),
                    text(&r.city),
                    text(&r.email),
                    text(&r.phone),
                    text(&r.mobile),
                ]
            })
            .collect(),
    }
}

/// CSV mit UTF-8-BOM und Semikolon, damit Excel (CH) Umlaute und Spalten korrekt erkennt.
pub fn write_csv(table: &Table, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(UTF8_BOM)?;
    let mut writer = csv::WriterBuilder::new().delimiter(b';').from_writer(file);
    writer.write_record(&table.headers).map_err(csv_error)?;
    for row in &table.rows {
        writer.write_record(row).map_err(csv_error)?;
    }
    writer.flush()?;
    Ok(())
}

/// XLSX mit einem Tabellenblatt: fette Kopfzeile, Autofilter, alle Zellen als Text.
pub fn write_xlsx(table: &Table, path: &Path, sheet_name: &str) -> Result<()> {
    let mut workbook = Workbook::new();
    fill_sheet(workbook.add_worksheet(), sheet_name, table)?;
    workbook.save(path).map_err(xlsx_error)
}

pub(crate) fn fill_sheet(sheet: &mut Worksheet, name: &str, table: &Table) -> Result<()> {
    let bold = Format::new().set_bold();
    sheet.set_name(name).map_err(xlsx_error)?;
    for (col, header) in table.headers.iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, header, &bold).map_err(xlsx_error)?;
    }
    for (r, row) in table.rows.iter().enumerate() {
        for (col, value) in row.iter().enumerate() {
            sheet.write_string(r as u32 + 1, col as u16, value).map_err(xlsx_error)?;
        }
    }
    if !table.headers.is_empty() {
        let last_col = table.headers.len() as u16 - 1;
        sheet.autofilter(0, 0, table.rows.len() as u32, last_col).map_err(xlsx_error)?;
    }
    Ok(())
}

pub(crate) fn xlsx_error(e: rust_xlsxwriter::XlsxError) -> CoreError {
    CoreError::Io(std::io::Error::other(format!("Excel-Export fehlgeschlagen: {e}")))
}

fn csv_error(e: csv::Error) -> CoreError {
    CoreError::Io(std::io::Error::other(format!("CSV-Export fehlgeschlagen: {e}")))
}

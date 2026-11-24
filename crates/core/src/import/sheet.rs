use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveDate;

use crate::error::{CoreError, Result};

/// Inhalt des ersten Tabellenblatts: Kopfzeile und nicht-leere Datenzeilen.
#[derive(Debug, Clone, PartialEq)]
pub struct Sheet {
    pub headers: Vec<String>,
    pub rows: Vec<SheetRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetRow {
    /// Zeilennummer wie in Excel (1-basiert), für den Import-Bericht.
    pub line: usize,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Number(f64),
    Bool(bool),
    Date(NaiveDate),
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        match self {
            Cell::Empty => true,
            Cell::Text(t) => t.trim().is_empty(),
            _ => false,
        }
    }
}

/// Liest das erste Tabellenblatt einer .xlsx/.xls-Datei.
pub fn read_first_sheet(path: &Path) -> Result<Sheet> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| CoreError::Import(format!("Datei kann nicht geöffnet werden: {e}")))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| CoreError::Import("Die Datei enthält kein Tabellenblatt.".into()))?
        .map_err(|e| CoreError::Import(format!("Tabellenblatt kann nicht gelesen werden: {e}")))?;
    let first_line = range.start().map(|(row, _)| row as usize + 1).unwrap_or(1);
    let mut rows = range.rows();
    let headers: Vec<String> = match rows.next() {
        Some(header) => header.iter().map(|c| c.to_string().trim().to_string()).collect(),
        None => return Err(CoreError::Import("Das Tabellenblatt ist leer.".into())),
    };
    let data = rows
        .enumerate()
        .map(|(i, r)| SheetRow { line: first_line + i + 1, cells: padded(r, headers.len()) })
        .filter(|r| !r.cells.iter().all(Cell::is_empty))
        .collect();
    Ok(Sheet { headers, rows: data })
}

fn padded(row: &[Data], len: usize) -> Vec<Cell> {
    let mut cells: Vec<Cell> = row.iter().map(convert).collect();
    cells.resize(len.max(cells.len()), Cell::Empty);
    cells
}

fn convert(data: &Data) -> Cell {
    match data {
        Data::Empty => Cell::Empty,
        Data::String(s) => Cell::Text(s.clone()),
        Data::Int(i) => Cell::Number(*i as f64),
        Data::Float(f) => Cell::Number(*f),
        Data::Bool(b) => Cell::Bool(*b),
        Data::DateTime(dt) => dt.as_datetime().map(|d| Cell::Date(d.date())).unwrap_or(Cell::Empty),
        Data::DateTimeIso(s) => NaiveDate::parse_from_str(&s[..s.len().min(10)], "%Y-%m-%d")
            .map(Cell::Date)
            .unwrap_or_else(|_| Cell::Text(s.clone())),
        other => Cell::Text(other.to_string()),
    }
}

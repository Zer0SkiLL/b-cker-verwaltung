//! Spaltenzuordnung und Lesen einzelner Zellen (ohne Datenbank).

use std::collections::HashMap;

use chrono::NaiveDate;

use super::sheet::{Cell, Sheet, SheetRow};
use super::values::{as_date, as_flag, as_int, as_text, DateResult, FlagResult};

/// Zielfeld einer Excel-Spalte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    LegacyId,
    CantonalAssociation,
    Company,
    LastName,
    FirstName,
    Addition,
    Street,
    Zip,
    City,
    Category,
    Email,
    Website,
    Apprenticeship,
    FrBaeck,
    FrConf,
    Dh,
    Notes,
    Joined,
    Left,
    Branches,
    Phone,
    Mobile,
    CostCenterNo,
    Mandate,
    CommercialRegisterNo,
    ApprenticeCount,
    Ignore,
}

/// Standard-Zuordnung nach den Kopfzeilen der Mitgliederliste (ohne Gross-/Kleinschreibung).
/// Nicht aufgeführte Spalten (z.B. „Adresszusatz“, leere Kopfzeilen) werden ignoriert.
const DEFAULTS: &[(&str, Field)] = &[
    ("obc-id", Field::LegacyId),
    ("kt verband", Field::CantonalAssociation),
    ("firma", Field::Company),
    ("name", Field::LastName),
    ("vorname", Field::FirstName),
    ("zusatz/inhaber (handelsregister)", Field::Addition),
    ("adresse", Field::Street),
    ("plz", Field::Zip),
    ("ort", Field::City),
    ("kategorie", Field::Category),
    ("mail", Field::Email),
    ("home", Field::Website),
    ("lehrbetrieb", Field::Apprenticeship),
    ("fr bäck", Field::FrBaeck),
    ("fr conf", Field::FrConf),
    ("dh", Field::Dh),
    ("besonderes", Field::Notes),
    ("eintritt", Field::Joined),
    ("austritt", Field::Left),
    ("filialen", Field::Branches),
    ("telefon", Field::Phone),
    ("mobiltelefon", Field::Mobile),
    ("kostenstellen-nr", Field::CostCenterNo),
    ("mandat", Field::Mandate),
    ("handelsregister-nr", Field::CommercialRegisterNo),
];

/// Fachrichtungs-Spalten und die daraus entstehenden Labels.
pub const FLAG_GROUP: &str = "Ausbildung Fachrichtung";
pub const FLAG_FIELDS: &[(Field, &str)] = &[
    (Field::FrBaeck, "Fr Bäck"),
    (Field::FrConf, "Fr Conf"),
    (Field::Dh, "DH"),
];

/// Zuordnung pro Spalte, in Spaltenreihenfolge des Blatts (Kopfzeile nur zur Anzeige).
#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub columns: Vec<(String, Field)>,
}

impl Mapping {
    pub fn default_for(headers: &[String]) -> Mapping {
        let columns = headers
            .iter()
            .map(|h| {
                let key = h.trim().to_lowercase();
                let field = DEFAULTS
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map_or(Field::Ignore, |(_, f)| *f);
                (h.clone(), field)
            })
            .collect();
        Mapping { columns }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportEntry {
    pub line: usize,
    pub column: Option<String>,
    pub message: String,
}

/// Feld → (Spaltenindex, Kopfzeile). `mapping.columns[i]` gehört zur Spalte `i` des Blatts,
/// damit auch doppelte oder leere Kopfzeilen eindeutig zugeordnet werden können.
/// Wird ein Feld mehrfach zugeordnet, gilt die erste Spalte.
pub(crate) type Index = HashMap<Field, (usize, String)>;

pub(crate) fn column_index(sheet: &Sheet, mapping: &Mapping) -> Index {
    let mut index = Index::new();
    let width = sheet.headers.len();
    for (pos, (header, field)) in mapping.columns.iter().enumerate().take(width) {
        if *field != Field::Ignore {
            index
                .entry(*field)
                .or_insert((pos, header.trim().to_string()));
        }
    }
    index
}

/// Liest Zellen einer Zeile und sammelt Hinweise für den Bericht.
pub(crate) struct RowCtx<'a> {
    pub row: &'a SheetRow,
    pub index: &'a Index,
    pub report: &'a mut Vec<ReportEntry>,
}

impl RowCtx<'_> {
    fn cell(&self, field: Field) -> &Cell {
        self.index
            .get(&field)
            .and_then(|(i, _)| self.row.cells.get(*i))
            .unwrap_or(&Cell::Empty)
    }

    pub fn text(&self, field: Field) -> Option<String> {
        as_text(self.cell(field))
    }

    pub fn note(&mut self, field: Option<Field>, message: String) {
        let column = field
            .and_then(|f| self.index.get(&f))
            .map(|(_, h)| h.clone());
        self.report.push(ReportEntry {
            line: self.row.line,
            column,
            message,
        });
    }

    pub fn int(&mut self, field: Field) -> Option<i64> {
        as_int(self.cell(field)).unwrap_or_else(|bad| {
            self.note(Some(field), format!("Keine ganze Zahl: {bad}"));
            None
        })
    }

    pub fn date(&mut self, field: Field) -> Option<NaiveDate> {
        match as_date(self.cell(field)) {
            DateResult::None => None,
            DateResult::Date(d) => Some(d),
            DateResult::YearOnly(d) => {
                self.note(Some(field), "Nur Jahr angegeben, 01.01. angenommen".into());
                Some(d)
            }
            DateResult::Invalid(bad) => {
                self.note(Some(field), format!("Ungültiges Datum: {bad}"));
                None
            }
        }
    }

    pub fn flag(&mut self, field: Field) -> bool {
        match as_flag(self.cell(field)) {
            FlagResult::Yes => true,
            FlagResult::No => false,
            FlagResult::Unknown(bad) => {
                self.note(Some(field), format!("Unbekannter Ja/Nein-Wert: {bad}"));
                false
            }
        }
    }
}

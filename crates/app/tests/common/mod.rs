#![allow(dead_code)]
use std::path::{Path, PathBuf};

use verwaltung_app::session::Session;
use verwaltung_core::storage::lock::Identity;

pub fn who(user: &str) -> Identity {
    Identity {
        user: user.into(),
        host: format!("pc-{user}"),
    }
}

pub fn db_path(dir: &Path) -> PathBuf {
    dir.join("verband.sqlite")
}

/// Neue Datei, schreibend geöffnet von „anna".
pub fn fresh_session(dir: &Path) -> Session {
    let mut session = Session::default();
    session.create(&db_path(dir), &who("anna")).unwrap();
    session
}

pub fn org_input(name: &str) -> verwaltung_core::repo::organisations::OrganisationInput {
    verwaltung_core::repo::organisations::OrganisationInput {
        name: name.into(),
        ..Default::default()
    }
}

/// Schreibt eine Excel-Datei mit Kopfzeile und Textzeilen (erstes Blatt).
pub fn write_xlsx(path: &Path, headers: &[&str], rows: &[Vec<&str>]) {
    let mut wb = rust_xlsxwriter::Workbook::new();
    let ws = wb.add_worksheet();
    for (col, h) in headers.iter().enumerate() {
        ws.write_string(0, col as u16, *h).unwrap();
    }
    for (r, row) in rows.iter().enumerate() {
        for (col, value) in row.iter().enumerate() {
            if !value.is_empty() {
                ws.write_string(r as u32 + 1, col as u16, *value).unwrap();
            }
        }
    }
    wb.save(path).unwrap();
}

/// Kopfzeile der heutigen Mitgliederliste (Spec §6.4).
pub const LIST_HEADERS: [&str; 23] = [
    "ID1",
    "Anzahl",
    "Kt Verband",
    "Name/Firma",
    "Vorname",
    "Zusatz",
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
    "Postfach",
];

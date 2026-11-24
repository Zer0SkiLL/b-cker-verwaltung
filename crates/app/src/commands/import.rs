//! Excel-Import der bisherigen Mitgliederliste: Vorschau (Probelauf) und Übernehmen.

use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;
use verwaltung_core::export::{write_csv, Table};
use verwaltung_core::import::apply::apply;
use verwaltung_core::import::mapping::{plan, Field, ImportPlan, Mapping, PlannedRow, ReportEntry};
use verwaltung_core::import::sheet::{read_first_sheet, Sheet};

use crate::error::{AppError, AppResult};
use crate::session::Session;

pub const MAPPING_MISMATCH: &str = "Die Zuordnung passt nicht zur Datei.";
const SAMPLE_ROWS: usize = 20;

pub const ALL_FIELDS: [Field; 24] = [
    Field::LegacyId,
    Field::ApprenticeCount,
    Field::CantonalAssociation,
    Field::Name,
    Field::FirstName,
    Field::Addition,
    Field::Street,
    Field::Zip,
    Field::City,
    Field::Category,
    Field::Email,
    Field::Website,
    Field::Apprenticeship,
    Field::FrBaeck,
    Field::FrConf,
    Field::Dh,
    Field::Notes,
    Field::Joined,
    Field::Left,
    Field::Branches,
    Field::Phone,
    Field::Mobile,
    Field::PoBox,
    Field::Ignore,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReportEntryDto {
    pub line: usize,
    pub column: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreviewRow {
    pub line: usize,
    pub organisation: String,
    pub person: Option<String>,
    pub city: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub headers: Vec<String>,
    pub mapping: Vec<String>,
    pub fields: Vec<String>,
    pub rows: usize,
    pub organisations: usize,
    pub persons: usize,
    pub sample: Vec<PreviewRow>,
    pub report: Vec<ReportEntryDto>,
    pub database_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImportSummaryDto {
    pub organisations: usize,
    pub persons: usize,
    pub report: Vec<ReportEntryDto>,
}

pub fn field_key(field: Field) -> &'static str {
    match field {
        Field::LegacyId => "legacy_id",
        Field::ApprenticeCount => "apprentice_count",
        Field::CantonalAssociation => "cantonal_association",
        Field::Name => "name",
        Field::FirstName => "first_name",
        Field::Addition => "addition",
        Field::Street => "street",
        Field::Zip => "zip",
        Field::City => "city",
        Field::Category => "category",
        Field::Email => "email",
        Field::Website => "website",
        Field::Apprenticeship => "apprenticeship",
        Field::FrBaeck => "fr_baeck",
        Field::FrConf => "fr_conf",
        Field::Dh => "dh",
        Field::Notes => "notes",
        Field::Joined => "joined",
        Field::Left => "left",
        Field::Branches => "branches",
        Field::Phone => "phone",
        Field::Mobile => "mobile",
        Field::PoBox => "po_box",
        Field::Ignore => "ignore",
    }
}

pub fn parse_field(key: &str) -> Option<Field> {
    ALL_FIELDS.into_iter().find(|f| field_key(*f) == key)
}

/// Probelauf: liest die Datei, plant mit Standard- oder eigener Zuordnung, ändert nichts.
pub fn import_preview(
    s: &mut Session,
    path: &Path,
    mapping: Option<Vec<String>>,
) -> AppResult<ImportPreview> {
    let sheet = read_first_sheet(path)?;
    let mapping = match mapping {
        Some(keys) => build_mapping(&sheet, &keys)?,
        None => Mapping::default_for(&sheet.headers),
    };
    let planned = plan(&sheet, &mapping);
    let database_empty = s.read(is_empty)?;
    Ok(ImportPreview {
        headers: sheet.headers.clone(),
        mapping: mapping
            .columns
            .iter()
            .map(|(_, f)| field_key(*f).to_string())
            .collect(),
        fields: ALL_FIELDS
            .iter()
            .map(|f| field_key(*f).to_string())
            .collect(),
        rows: planned.rows.len(),
        organisations: planned.rows.len(),
        persons: planned.rows.iter().filter(|r| r.person.is_some()).count(),
        sample: planned
            .rows
            .iter()
            .take(SAMPLE_ROWS)
            .map(preview_row)
            .collect(),
        report: report_dtos(&planned.report),
        database_empty,
    })
}

/// Plant erneut und übernimmt alles oder nichts (nur in eine leere Datei).
pub fn import_apply(
    s: &mut Session,
    path: &Path,
    mapping: Vec<String>,
) -> AppResult<ImportSummaryDto> {
    let sheet = read_first_sheet(path)?;
    let planned: ImportPlan = plan(&sheet, &build_mapping(&sheet, &mapping)?);
    let summary = s.write_mut(|c| apply(c, &planned))?;
    Ok(ImportSummaryDto {
        organisations: summary.organisations,
        persons: summary.persons,
        report: report_dtos(&summary.report),
    })
}

/// Speichert den Import-Bericht als CSV (für die Nachbearbeitung in Excel).
pub fn export_report(report: &[ReportEntryDto], path: &Path) -> AppResult<()> {
    let table = Table {
        headers: vec!["Zeile".into(), "Spalte".into(), "Meldung".into()],
        rows: report
            .iter()
            .map(|r| {
                vec![
                    r.line.to_string(),
                    r.column.clone().unwrap_or_default(),
                    r.message.clone(),
                ]
            })
            .collect(),
    };
    Ok(write_csv(&table, path)?)
}

fn build_mapping(sheet: &Sheet, keys: &[String]) -> AppResult<Mapping> {
    if keys.len() != sheet.headers.len() {
        return Err(AppError::new(MAPPING_MISMATCH));
    }
    let columns = sheet
        .headers
        .iter()
        .zip(keys)
        .map(|(header, key)| {
            parse_field(key)
                .map(|field| (header.clone(), field))
                .ok_or_else(|| AppError(format!("Unbekanntes Feld: {key}")))
        })
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Mapping { columns })
}

fn is_empty(c: &Connection) -> verwaltung_core::Result<bool> {
    let count: i64 = c.query_row(
        "SELECT (SELECT COUNT(*) FROM organisations) + (SELECT COUNT(*) FROM persons)",
        [],
        |r| r.get(0),
    )?;
    Ok(count == 0)
}

fn preview_row(row: &PlannedRow) -> PreviewRow {
    PreviewRow {
        line: row.line,
        organisation: row.organisation.name.clone(),
        person: row.person.as_ref().map(|p| {
            [p.first_name.clone(), Some(p.last_name.clone())]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ")
        }),
        city: row.organisation.city.clone(),
        category: row.category.clone(),
    }
}

fn report_dtos(report: &[ReportEntry]) -> Vec<ReportEntryDto> {
    report
        .iter()
        .map(|r| ReportEntryDto {
            line: r.line,
            column: r.column.clone(),
            message: r.message.clone(),
        })
        .collect()
}

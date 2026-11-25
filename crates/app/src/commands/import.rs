//! Excel-Import der Mitgliederliste: Vorschau (Probelauf), Übernehmen, Rückgängig.

use std::path::Path;

use chrono::{DateTime, Local};
use rusqlite::Connection;
use serde::Serialize;
use verwaltung_core::export::{write_csv, write_xlsx, Table};
use verwaltung_core::import::apply::apply;
use verwaltung_core::import::mapping::{Field, Mapping};
use verwaltung_core::import::plan::{plan, reconcile, ImportPlan};
use verwaltung_core::import::sheet::{read_first_sheet, Sheet};
use verwaltung_core::storage::backup::{backup_to_dir, db_stem, local_backup_dir};

use super::import_preview::{export_table, individuals, new_entries, organisations, report_dtos};
pub use super::import_preview::{
    NewEntries, PreviewIndividual, PreviewOrganisation, PreviewPerson, ReportEntryDto, SkippedDto,
};
use crate::error::{AppError, AppResult};
use crate::session::Session;

pub const MAPPING_MISMATCH: &str = "Die Zuordnung passt nicht zur Datei.";
pub const UNDO_INVALID: &str = "Ungültige Sicherung für das Rückgängigmachen.";

pub const ALL_FIELDS: [Field; 27] = [
    Field::Company,
    Field::LastName,
    Field::FirstName,
    Field::Addition,
    Field::LegacyId,
    Field::CantonalAssociation,
    Field::Street,
    Field::Zip,
    Field::City,
    Field::Category,
    Field::Email,
    Field::Website,
    Field::Phone,
    Field::Mobile,
    Field::Apprenticeship,
    Field::FrBaeck,
    Field::FrConf,
    Field::Dh,
    Field::Notes,
    Field::Joined,
    Field::Left,
    Field::Branches,
    Field::ApprenticeCount,
    Field::CostCenterNo,
    Field::CommercialRegisterNo,
    Field::Mandate,
    Field::Ignore,
];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub headers: Vec<String>,
    pub mapping: Vec<String>,
    pub fields: Vec<String>,
    /// Nicht leere Datenzeilen der Datei.
    pub rows: usize,
    pub organisations: Vec<PreviewOrganisation>,
    /// Zeilen, die in einen schon vorhandenen Betrieb zusammengefasst wurden.
    pub merged: usize,
    pub persons: usize,
    pub individuals: Vec<PreviewIndividual>,
    pub skipped: Vec<SkippedDto>,
    pub new_entries: NewEntries,
    /// Gesetzt, wenn der Zeilenabgleich nicht aufgeht (Übernehmen ist dann gesperrt).
    pub reconcile_error: Option<String>,
    pub report: Vec<ReportEntryDto>,
    pub database_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummaryDto {
    pub organisations: usize,
    pub persons: usize,
    pub individuals: usize,
    pub report: Vec<ReportEntryDto>,
    /// Sicherung von unmittelbar vor dem Import (für „Import rückgängig machen“).
    pub undo_backup: String,
}

pub fn field_key(field: Field) -> &'static str {
    match field {
        Field::LegacyId => "legacy_id",
        Field::CantonalAssociation => "cantonal_association",
        Field::Company => "company",
        Field::LastName => "last_name",
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
        Field::CostCenterNo => "cost_center_no",
        Field::Mandate => "mandate",
        Field::CommercialRegisterNo => "commercial_register_no",
        Field::ApprenticeCount => "apprentice_count",
        Field::Ignore => "ignore",
    }
}

pub fn parse_field(key: &str) -> Option<Field> {
    ALL_FIELDS.into_iter().find(|f| field_key(*f) == key)
}

fn planned(path: &Path, mapping: Option<Vec<String>>) -> AppResult<(Sheet, Mapping, ImportPlan)> {
    let sheet = read_first_sheet(path)?;
    let mapping = match mapping {
        Some(keys) => build_mapping(&sheet, &keys)?,
        None => Mapping::default_for(&sheet.headers),
    };
    let planned = plan(&sheet, &mapping);
    Ok((sheet, mapping, planned))
}

/// Probelauf: liest die Datei, plant mit Standard- oder eigener Zuordnung, ändert nichts.
pub fn import_preview(
    s: &mut Session,
    path: &Path,
    mapping: Option<Vec<String>>,
) -> AppResult<ImportPreview> {
    let (sheet, mapping, planned) = planned(path, mapping)?;
    let database_empty = s.read(is_empty)?;
    let new_entries = s.read(|c| new_entries(c, &planned))?;
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
        rows: planned.data_rows,
        merged: planned
            .organisations
            .iter()
            .map(|o| o.lines.len() - 1)
            .sum(),
        persons: planned.organisations.iter().map(|o| o.persons.len()).sum(),
        organisations: organisations(&planned),
        individuals: individuals(&planned),
        skipped: planned
            .skipped
            .iter()
            .map(|s| SkippedDto {
                line: s.line,
                reason: s.reason.clone(),
            })
            .collect(),
        new_entries,
        reconcile_error: reconcile(&planned).err(),
        report: report_dtos(&planned.report),
        database_empty,
    })
}

/// Vorschau als Excel-Datei: eine Zeile pro Quellzeile.
pub fn export_preview(
    _s: &mut Session,
    path: &Path,
    mapping: Option<Vec<String>>,
    out: &Path,
) -> AppResult<()> {
    let (_, _, planned) = planned(path, mapping)?;
    Ok(write_xlsx(&export_table(&planned), out, "Vorschau")?)
}

/// Plant erneut, sichert die Datei und übernimmt alles oder nichts (nur in eine leere Datei).
pub fn import_apply(
    s: &mut Session,
    path: &Path,
    mapping: Vec<String>,
    now: DateTime<Local>,
) -> AppResult<ImportSummaryDto> {
    let (_, _, planned) = planned(path, Some(mapping))?;
    reconcile(&planned).map_err(AppError)?;
    if !s.read(is_empty)? {
        return Err(verwaltung_core::CoreError::DatabaseNotEmpty.into());
    }
    let ws = s.writable()?;
    let backup = backup_to_dir(
        &ws.conn,
        &local_backup_dir(ws.path()),
        &db_stem(ws.path()),
        now,
    )?;
    let summary = s.write_mut(|c| apply(c, &planned))?;
    s.remember_import_backup(backup.clone());
    Ok(ImportSummaryDto {
        organisations: summary.organisations,
        persons: summary.persons,
        individuals: summary.individuals,
        report: report_dtos(&summary.report),
        undo_backup: backup.display().to_string(),
    })
}

/// Spielt die Sicherung von unmittelbar vor dem letzten Import zurück (nur diese, nur einmal).
pub fn import_undo(s: &mut Session, backup: &Path, now: DateTime<Local>) -> AppResult<()> {
    s.writable()?;
    let Some(backup) = s.take_import_backup(backup) else {
        return Err(AppError::new(UNDO_INVALID));
    };
    if let Err(err) = s.writable()?.restore(&backup, now) {
        // Fehlgeschlagen: erneut versuchen erlauben.
        s.remember_import_backup(backup);
        return Err(err.into());
    }
    Ok(())
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

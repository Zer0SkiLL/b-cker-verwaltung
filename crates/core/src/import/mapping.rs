//! Spaltenzuordnung und Probelauf (ohne Datenbank).

use std::collections::HashMap;

use chrono::NaiveDate;

use super::sheet::{Cell, Sheet, SheetRow};
use super::values::{as_date, as_flag, as_int, as_text, DateResult, FlagResult};
use crate::repo::organisations::OrganisationInput;
use crate::repo::persons::PersonInput;

/// Zielfeld einer Excel-Spalte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    LegacyId,
    ApprenticeCount,
    CantonalAssociation,
    Name,
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
    PoBox,
    Ignore,
}

/// Standard-Zuordnung gemäss Spezifikation (Kopfzeile, ohne Gross-/Kleinschreibung).
const DEFAULTS: &[(&str, Field)] = &[
    ("id1", Field::LegacyId),
    ("anzahl", Field::ApprenticeCount),
    ("kt verband", Field::CantonalAssociation),
    ("name/firma", Field::Name),
    ("vorname", Field::FirstName),
    ("zusatz", Field::Addition),
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
    ("postfach", Field::PoBox),
];

/// Fachrichtungs-Spalten und die daraus entstehenden Labels.
pub const FLAG_GROUP: &str = "Ausbildung Fachrichtung";
const FLAG_FIELDS: &[(Field, &str)] = &[
    (Field::FrBaeck, "Fr Bäck"),
    (Field::FrConf, "Fr Conf"),
    (Field::Dh, "DH"),
];

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

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedRow {
    pub line: usize,
    pub organisation: OrganisationInput,
    pub person: Option<PersonInput>,
    pub category: Option<String>,
    pub cantonal: Option<String>,
    pub flags: Vec<&'static str>,
    pub joined: Option<NaiveDate>,
    pub left: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportPlan {
    pub rows: Vec<PlannedRow>,
    pub report: Vec<ReportEntry>,
}

/// Probelauf: deutet alle Zeilen, ohne etwas zu speichern.
pub fn plan(sheet: &Sheet, mapping: &Mapping) -> ImportPlan {
    let index = column_index(sheet, mapping);
    let mut result = ImportPlan::default();
    for row in &sheet.rows {
        let mut ctx = RowCtx {
            row,
            index: &index,
            report: &mut result.report,
        };
        if let Some(planned) = plan_row(&mut ctx) {
            result.rows.push(planned);
        }
    }
    report_duplicates(&mut result);
    result
}

/// Feld → (Spaltenindex, Kopfzeile); bei doppelter Zuordnung gilt die erste Spalte.
type Index = HashMap<Field, (usize, String)>;

fn column_index(sheet: &Sheet, mapping: &Mapping) -> Index {
    let mut index = Index::new();
    for (header, field) in &mapping.columns {
        let pos = sheet
            .headers
            .iter()
            .position(|h| h.trim().eq_ignore_ascii_case(header.trim()));
        if let (Some(pos), false) = (pos, *field == Field::Ignore) {
            index
                .entry(*field)
                .or_insert((pos, header.trim().to_string()));
        }
    }
    index
}

struct RowCtx<'a> {
    row: &'a SheetRow,
    index: &'a Index,
    report: &'a mut Vec<ReportEntry>,
}

impl RowCtx<'_> {
    fn cell(&self, field: Field) -> &Cell {
        self.index
            .get(&field)
            .and_then(|(i, _)| self.row.cells.get(*i))
            .unwrap_or(&Cell::Empty)
    }

    fn text(&self, field: Field) -> Option<String> {
        as_text(self.cell(field))
    }

    fn note(&mut self, field: Option<Field>, message: String) {
        let column = field
            .and_then(|f| self.index.get(&f))
            .map(|(_, h)| h.clone());
        self.report.push(ReportEntry {
            line: self.row.line,
            column,
            message,
        });
    }

    fn int(&mut self, field: Field) -> Option<i64> {
        as_int(self.cell(field)).unwrap_or_else(|bad| {
            self.note(Some(field), format!("Keine ganze Zahl: {bad}"));
            None
        })
    }

    fn date(&mut self, field: Field) -> Option<NaiveDate> {
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

    fn flag(&mut self, field: Field) -> bool {
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

fn plan_row(ctx: &mut RowCtx) -> Option<PlannedRow> {
    let Some(name) = ctx.text(Field::Name) else {
        ctx.note(
            Some(Field::Name),
            "Name/Firma fehlt, Zeile übersprungen".into(),
        );
        return None;
    };
    let first_name = ctx.text(Field::FirstName);
    let mobile = ctx.text(Field::Mobile);
    let (org_name, addition, person) = match &first_name {
        Some(first) => (
            ctx.text(Field::Addition)
                .unwrap_or_else(|| format!("{first} {name}")),
            None,
            Some(PersonInput {
                first_name: Some(first.clone()),
                last_name: name,
                mobile: mobile.clone(),
                ..Default::default()
            }),
        ),
        None => (name, ctx.text(Field::Addition), None),
    };
    let organisation = organisation(
        ctx,
        org_name,
        addition,
        person.is_none().then_some(mobile).flatten(),
    );
    let flags = flags(ctx);
    let (joined, left) = membership_dates(ctx);
    Some(PlannedRow {
        line: ctx.row.line,
        organisation,
        person,
        category: ctx.text(Field::Category),
        cantonal: ctx.text(Field::CantonalAssociation),
        flags,
        joined,
        left,
    })
}

/// Betrieb ohne Person: Mobiltelefon wird zur Telefonnummer bzw. in die Bemerkungen übernommen.
fn organisation(
    ctx: &mut RowCtx,
    name: String,
    addition: Option<String>,
    mobile: Option<String>,
) -> OrganisationInput {
    let mut phone = ctx.text(Field::Phone);
    let mut notes = ctx.text(Field::Notes);
    if let Some(m) = mobile {
        if phone.is_none() {
            phone = Some(m);
        } else {
            let extra = format!("Mobiltelefon: {m}");
            notes = Some(notes.map_or(extra.clone(), |n| format!("{n}\n{extra}")));
        }
    }
    OrganisationInput {
        name,
        addition,
        street: ctx.text(Field::Street),
        po_box: ctx.text(Field::PoBox),
        zip: ctx.text(Field::Zip),
        city: ctx.text(Field::City),
        phone,
        email: ctx.text(Field::Email),
        website: ctx.text(Field::Website),
        branches: ctx.int(Field::Branches),
        apprentice_count: ctx.int(Field::ApprenticeCount),
        notes,
        legacy_id: ctx.text(Field::LegacyId),
        org_type_id: None,
        cantonal_association_id: None,
    }
}

fn flags(ctx: &mut RowCtx) -> Vec<&'static str> {
    let flags: Vec<&'static str> = FLAG_FIELDS
        .iter()
        .filter_map(|(field, label)| ctx.flag(*field).then_some(*label))
        .collect();
    if ctx.flag(Field::Apprenticeship) && flags.is_empty() {
        ctx.note(
            Some(Field::Apprenticeship),
            "Lehrbetrieb ohne Fachrichtung".into(),
        );
    }
    flags
}

fn membership_dates(ctx: &mut RowCtx) -> (Option<NaiveDate>, Option<NaiveDate>) {
    let joined = ctx.date(Field::Joined);
    let left = ctx.date(Field::Left);
    match (joined, left) {
        (Some(j), Some(l)) if l < j => {
            ctx.note(
                Some(Field::Left),
                "Austritt liegt vor Eintritt, Austritt ignoriert".into(),
            );
            (joined, None)
        }
        _ => (joined, left),
    }
}

fn report_duplicates(plan: &mut ImportPlan) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for row in &plan.rows {
        *counts
            .entry(row.organisation.name.to_lowercase())
            .or_default() += 1;
    }
    for row in &plan.rows {
        if counts[&row.organisation.name.to_lowercase()] > 1 {
            plan.report.push(ReportEntry {
                line: row.line,
                column: None,
                message: format!("Doppelter Firmenname: {}", row.organisation.name),
            });
        }
    }
    plan.report.sort_by_key(|r| r.line);
}

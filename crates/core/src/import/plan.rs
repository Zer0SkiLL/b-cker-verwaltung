//! Probelauf: deutet die Zeilen der Mitgliederliste, fasst Betriebe zusammen und prüft,
//! dass jede Zeile genau einmal verarbeitet ist. Speichert nichts.

use std::collections::HashMap;

use chrono::NaiveDate;

use super::mapping::{column_index, Field, Mapping, ReportEntry, RowCtx, FLAG_FIELDS};
use super::sheet::Sheet;
use crate::repo::organisations::OrganisationInput;
use crate::repo::persons::PersonInput;

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedPerson {
    pub line: usize,
    pub input: PersonInput,
    pub mandates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedOrganisation {
    /// Alle Zeilen der Datei, die zu diesem Betrieb gehören (erste zuerst).
    pub lines: Vec<usize>,
    pub input: OrganisationInput,
    pub category: Option<String>,
    pub cantonal: Option<String>,
    pub flags: Vec<&'static str>,
    pub mandates: Vec<String>,
    pub joined: Option<NaiveDate>,
    pub left: Option<NaiveDate>,
    pub persons: Vec<PlannedPerson>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannedIndividual {
    pub person: PlannedPerson,
    pub category: Option<String>,
    pub joined: Option<NaiveDate>,
    pub left: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkippedRow {
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImportPlan {
    /// Anzahl nicht leerer Datenzeilen der Datei.
    pub data_rows: usize,
    pub organisations: Vec<PlannedOrganisation>,
    pub individuals: Vec<PlannedIndividual>,
    pub skipped: Vec<SkippedRow>,
    pub report: Vec<ReportEntry>,
}

pub const SKIP_NO_NAME: &str = "Weder Firma noch Name, Zeile übersprungen";

/// Gelesene Werte einer Zeile.
struct RowData {
    line: usize,
    company: Option<String>,
    last: Option<String>,
    first: Option<String>,
    legacy_id: Option<String>,
    cantonal: Option<String>,
    addition: Option<String>,
    street: Option<String>,
    zip: Option<String>,
    city: Option<String>,
    category: Option<String>,
    email: Option<String>,
    website: Option<String>,
    flags: Vec<&'static str>,
    notes: Option<String>,
    joined: Option<NaiveDate>,
    left: Option<NaiveDate>,
    branches: Option<i64>,
    apprentice_count: Option<i64>,
    phone: Option<String>,
    mobile: Option<String>,
    cost_center_no: Option<String>,
    commercial_register_no: Option<String>,
    mandates: Vec<String>,
}

pub fn plan(sheet: &Sheet, mapping: &Mapping) -> ImportPlan {
    let index = column_index(sheet, mapping);
    let mut result = ImportPlan {
        data_rows: sheet.rows.len(),
        ..Default::default()
    };
    let mut by_key: HashMap<(String, String, String), usize> = HashMap::new();
    for row in &sheet.rows {
        let mut ctx = RowCtx {
            row,
            index: &index,
            report: &mut result.report,
        };
        let data = read_row(&mut ctx);
        let person = person_from(&mut ctx, &data);
        match &data.company {
            Some(company) => {
                let key = (norm(company), norm_opt(&data.zip), norm_opt(&data.street));
                match by_key.get(&key) {
                    Some(&i) => merge_into(&mut ctx, &mut result.organisations[i], data, person),
                    None => {
                        by_key.insert(key, result.organisations.len());
                        result.organisations.push(new_organisation(data, person));
                    }
                }
            }
            None => match person {
                Some(person) => {
                    if !data.flags.is_empty() {
                        ctx.note(None, "Fachrichtung bei Einzelmitglied ignoriert".into());
                    }
                    result.individuals.push(individual(data, person));
                }
                None => {
                    ctx.note(Some(Field::Company), SKIP_NO_NAME.into());
                    result.skipped.push(SkippedRow {
                        line: data.line,
                        reason: SKIP_NO_NAME.into(),
                    });
                }
            },
        }
    }
    report_merged(&mut result);
    result.report.sort_by_key(|r| r.line);
    result
}

/// Jede Datenzeile muss genau einem Ergebnis zugeordnet sein.
pub fn reconcile(plan: &ImportPlan) -> Result<(), String> {
    let mut lines: Vec<usize> = plan
        .organisations
        .iter()
        .flat_map(|o| o.lines.iter().copied())
        .chain(plan.individuals.iter().map(|i| i.person.line))
        .chain(plan.skipped.iter().map(|s| s.line))
        .collect();
    let total = lines.len();
    lines.sort_unstable();
    lines.dedup();
    if total == plan.data_rows && lines.len() == total {
        Ok(())
    } else {
        Err(format!(
            "Der Abgleich der Zeilen geht nicht auf ({} Zeilen in der Datei, {} verarbeitet) – bitte Problem melden",
            plan.data_rows, total
        ))
    }
}

fn read_row(ctx: &mut RowCtx) -> RowData {
    let joined = ctx.date(Field::Joined);
    let left = ctx.date(Field::Left);
    let joined = match (joined, left) {
        (Some(j), Some(l)) if l < j => {
            ctx.note(
                Some(Field::Joined),
                "Eintritt liegt nach Austritt, Eintritt ignoriert".into(),
            );
            None
        }
        _ => joined,
    };
    RowData {
        line: ctx.row.line,
        company: ctx.text(Field::Company),
        last: ctx.text(Field::LastName),
        first: ctx.text(Field::FirstName),
        legacy_id: ctx.text(Field::LegacyId),
        cantonal: ctx.text(Field::CantonalAssociation),
        addition: ctx.text(Field::Addition),
        street: ctx.text(Field::Street),
        zip: ctx.text(Field::Zip),
        city: ctx.text(Field::City),
        category: ctx.text(Field::Category),
        email: ctx.text(Field::Email),
        website: ctx.text(Field::Website),
        flags: flags(ctx),
        notes: ctx.text(Field::Notes),
        joined,
        left,
        branches: ctx.int(Field::Branches),
        apprentice_count: ctx.int(Field::ApprenticeCount),
        phone: ctx.text(Field::Phone),
        mobile: ctx.text(Field::Mobile),
        cost_center_no: ctx.text(Field::CostCenterNo),
        commercial_register_no: ctx.text(Field::CommercialRegisterNo),
        mandates: ctx.text(Field::Mandate).map(split_list).unwrap_or_default(),
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

fn split_list(text: String) -> Vec<String> {
    text.split([',', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Person der Zeile (nur Name und Mobiltelefon; übrige Kontaktdaten gehören zunächst dem Betrieb).
fn person_from(ctx: &mut RowCtx, d: &RowData) -> Option<PlannedPerson> {
    let (first, last) = match (&d.first, &d.last) {
        (_, Some(last)) => (d.first.clone(), last.clone()),
        (Some(first), None) => {
            ctx.note(
                Some(Field::FirstName),
                "Nur Vorname angegeben, als Nachname übernommen".into(),
            );
            (None, first.clone())
        }
        (None, None) => return None,
    };
    Some(PlannedPerson {
        line: d.line,
        input: PersonInput {
            first_name: first,
            last_name: last,
            mobile: d.mobile.clone(),
            ..Default::default()
        },
        mandates: Vec::new(),
    })
}

fn new_organisation(d: RowData, person: Option<PlannedPerson>) -> PlannedOrganisation {
    let mut phone = d.phone.clone();
    let mut notes = d.notes.clone();
    if person.is_none() {
        if let Some(m) = &d.mobile {
            if phone.is_none() {
                phone = Some(m.clone());
            } else {
                append(&mut notes, format!("Mobiltelefon: {m}"));
            }
        }
    }
    let (org_mandates, persons) = attach_mandates(d.mandates.clone(), person);
    PlannedOrganisation {
        lines: vec![d.line],
        input: OrganisationInput {
            name: d.company.clone().unwrap_or_default(),
            addition: d.addition,
            street: d.street,
            zip: d.zip,
            city: d.city,
            phone,
            email: d.email,
            website: d.website,
            branches: d.branches,
            apprentice_count: d.apprentice_count,
            notes,
            legacy_id: d.legacy_id,
            commercial_register_no: d.commercial_register_no,
            cost_center_no: d.cost_center_no,
            ..Default::default()
        },
        category: d.category,
        cantonal: d.cantonal,
        flags: d.flags,
        mandates: org_mandates,
        joined: d.joined,
        left: d.left,
        persons,
    }
}

/// Mandate gehen an die Person der Zeile, sonst an den Betrieb.
fn attach_mandates(
    mandates: Vec<String>,
    person: Option<PlannedPerson>,
) -> (Vec<String>, Vec<PlannedPerson>) {
    match person {
        Some(p) => (Vec::new(), vec![PlannedPerson { mandates, ..p }]),
        None => (mandates, Vec::new()),
    }
}

/// Weitere Zeile eines schon bekannten Betriebs.
fn merge_into(
    ctx: &mut RowCtx,
    org: &mut PlannedOrganisation,
    d: RowData,
    person: Option<PlannedPerson>,
) {
    let first = org.lines[0];
    let mut deviations: Vec<&str> = Vec::new();
    let mut check = |deviates: bool, label: &'static str| {
        if deviates {
            deviations.push(label);
        }
    };
    check(fill(&mut org.input.addition, d.addition), "Zusatz");
    check(fill(&mut org.input.city, d.city), "Ort");
    check(fill(&mut org.cantonal, d.cantonal), "Kantonalverband");
    check(fill(&mut org.category, d.category), "Kategorie");
    check(fill_eq(&mut org.joined, d.joined), "Eintritt");
    check(fill_eq(&mut org.left, d.left), "Austritt");
    check(fill(&mut org.input.website, d.website), "Website");
    check(fill_eq(&mut org.input.branches, d.branches), "Filialen");
    check(
        fill_eq(&mut org.input.apprentice_count, d.apprentice_count),
        "Anzahl Lernende",
    );
    check(
        fill(&mut org.input.cost_center_no, d.cost_center_no),
        "Kostenstellen-Nr",
    );
    check(
        fill(
            &mut org.input.commercial_register_no,
            d.commercial_register_no,
        ),
        "Handelsregister-Nr",
    );
    for label in deviations {
        ctx.note(
            None,
            format!("Abweichende Angabe für {label} (erste Zeile {first} gilt)"),
        );
    }
    for flag in d.flags {
        if !org.flags.contains(&flag) {
            org.flags.push(flag);
        }
    }
    let line = d.line;
    match person {
        // Kontaktangaben einer weiteren Zeile mit Person gehören der Person, sofern sie nicht
        // genau denen des Betriebs entsprechen.
        Some(mut p) => {
            if let Some(v) = own(&org.input.email, d.email) {
                p.input.email = Some(v);
            }
            if let Some(v) = own(&org.input.phone, d.phone) {
                p.input.phone = Some(v);
            }
            if let Some(v) = own(&org.input.notes, d.notes) {
                append(&mut p.input.notes, v);
            }
            if let Some(v) = own(&org.input.legacy_id, d.legacy_id) {
                append(&mut p.input.notes, format!("OBC-ID: {v}"));
            }
            p.mandates = d.mandates;
            org.persons.push(p);
        }
        // Ohne Person: Lücken füllen, Abweichungen in die Bemerkungen des Betriebs.
        None => {
            let extras = [
                ("Mail", contact(&mut org.input.email, d.email)),
                ("Telefon", contact(&mut org.input.phone, d.phone)),
                ("Besonderes", contact(&mut org.input.notes, d.notes)),
                ("OBC-ID", contact(&mut org.input.legacy_id, d.legacy_id)),
                ("Mobiltelefon", contact(&mut org.input.phone, d.mobile)),
            ];
            for (label, value) in extras {
                if let Some(value) = value {
                    append(
                        &mut org.input.notes,
                        format!("Zeile {line}: {label} {value}"),
                    );
                }
            }
            for m in d.mandates {
                if !org.mandates.contains(&m) {
                    org.mandates.push(m);
                }
            }
        }
    }
    org.lines.push(line);
}

fn individual(d: RowData, person: PlannedPerson) -> PlannedIndividual {
    let mut notes = d.notes;
    if let Some(id) = d.legacy_id {
        append(&mut notes, format!("OBC-ID: {id}"));
    }
    PlannedIndividual {
        person: PlannedPerson {
            input: PersonInput {
                street: d.street,
                zip: d.zip,
                city: d.city,
                phone: d.phone,
                email: d.email,
                notes,
                ..person.input
            },
            mandates: d.mandates,
            ..person
        },
        category: d.category,
        joined: d.joined,
        left: d.left,
    }
}

fn report_merged(plan: &mut ImportPlan) {
    for org in plan.organisations.iter().filter(|o| o.lines.len() > 1) {
        plan.report.push(ReportEntry {
            line: org.lines[0],
            column: None,
            message: format!(
                "{} Zeilen zu einem Betrieb zusammengefasst",
                org.lines.len()
            ),
        });
    }
}

/// Füllt einen leeren Text; liefert `true`, wenn ein vorhandener Wert abweicht.
fn fill(slot: &mut Option<String>, new: Option<String>) -> bool {
    match (slot.as_ref(), new) {
        (None, Some(v)) => {
            *slot = Some(v);
            false
        }
        (Some(old), Some(v)) => norm(old) != norm(&v),
        _ => false,
    }
}

fn fill_eq<T: PartialEq>(slot: &mut Option<T>, new: Option<T>) -> bool {
    match (slot.as_ref(), new) {
        (None, Some(v)) => {
            *slot = Some(v);
            false
        }
        (Some(old), Some(v)) => *old != v,
        _ => false,
    }
}

/// Wert einer weiteren Zeile, der nicht genau dem des Betriebs entspricht.
fn own(org_value: &Option<String>, new: Option<String>) -> Option<String> {
    new.filter(|v| org_value.as_deref().map(norm) != Some(norm(v)))
}

/// Füllt eine leere Kontaktangabe; liefert den abweichenden Wert, der anderswo hin muss.
fn contact(slot: &mut Option<String>, new: Option<String>) -> Option<String> {
    match (slot.as_ref(), new) {
        (None, Some(v)) => {
            *slot = Some(v);
            None
        }
        (Some(old), Some(v)) if norm(old) != norm(&v) => Some(v),
        _ => None,
    }
}

fn append(notes: &mut Option<String>, extra: String) {
    *notes = Some(match notes.take() {
        Some(n) => format!("{n}\n{extra}"),
        None => extra,
    });
}

fn norm(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn norm_opt(text: &Option<String>) -> String {
    text.as_deref().map(norm).unwrap_or_default()
}

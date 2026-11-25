//! Vorschau des Imports: gegliedert wie nachher im Programm, mit „neu angelegt“ und Excel-Export.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use verwaltung_core::export::Table;
use verwaltung_core::import::apply::{IMPORT_FUNCTION, MANDATE_GROUP};
use verwaltung_core::import::lookup::{contains, existing_labels, existing_names};
use verwaltung_core::import::mapping::{ReportEntry, FLAG_GROUP};
use verwaltung_core::import::plan::{ImportPlan, PlannedOrganisation, PlannedPerson};
use verwaltung_core::model::LookupKind;
use verwaltung_core::repo::persons::PersonInput;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportEntryDto {
    pub line: usize,
    pub column: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreviewPerson {
    pub line: usize,
    pub name: String,
    pub function: String,
    pub labels: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreviewOrganisation {
    pub lines: Vec<usize>,
    pub name: String,
    pub city: Option<String>,
    pub category: Option<String>,
    pub labels: Vec<String>,
    pub persons: Vec<PreviewPerson>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PreviewIndividual {
    pub person: PreviewPerson,
    pub city: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkippedDto {
    pub line: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct NewEntries {
    pub categories: Vec<String>,
    pub cantonal: Vec<String>,
    pub functions: Vec<String>,
    /// „Gruppe: Label“
    pub labels: Vec<String>,
}

pub fn report_dtos(report: &[ReportEntry]) -> Vec<ReportEntryDto> {
    report
        .iter()
        .map(|r| ReportEntryDto {
            line: r.line,
            column: r.column.clone(),
            message: r.message.clone(),
        })
        .collect()
}

fn notes_for(report: &[ReportEntry], lines: &[usize]) -> Vec<String> {
    report
        .iter()
        .filter(|r| lines.contains(&r.line))
        .map(|r| match &r.column {
            Some(c) => format!("{c}: {}", r.message),
            None => r.message.clone(),
        })
        .collect()
}

fn person_name(p: &PersonInput) -> String {
    [p.first_name.clone(), Some(p.last_name.clone())]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
}

fn mandate_labels(mandates: &[String]) -> Vec<String> {
    mandates
        .iter()
        .map(|m| format!("{MANDATE_GROUP}: {m}"))
        .collect()
}

fn preview_person(p: &PlannedPerson, function: &str, report: &[ReportEntry]) -> PreviewPerson {
    PreviewPerson {
        line: p.line,
        name: person_name(&p.input),
        function: function.into(),
        labels: mandate_labels(&p.mandates),
        notes: notes_for(report, &[p.line]),
    }
}

fn org_labels(o: &PlannedOrganisation) -> Vec<String> {
    o.flags
        .iter()
        .map(|f| format!("{FLAG_GROUP}: {f}"))
        .chain(mandate_labels(&o.mandates))
        .collect()
}

fn org_lines_without_person(o: &PlannedOrganisation) -> Vec<usize> {
    o.lines
        .iter()
        .enumerate()
        .filter(|(i, line)| *i == 0 || !o.persons.iter().any(|p| p.line == **line))
        .map(|(_, line)| *line)
        .collect()
}

pub fn organisations(plan: &ImportPlan) -> Vec<PreviewOrganisation> {
    plan.organisations
        .iter()
        .map(|o| {
            // Hinweise aller Zeilen ohne Person (die erste immer); Zeilen mit Person zeigen ihre
            // eigenen, ohne die schon beim Betrieb stehenden zu wiederholen.
            let notes = notes_for(&plan.report, &org_lines_without_person(o));
            let persons = o
                .persons
                .iter()
                .map(|p| {
                    let person = preview_person(p, IMPORT_FUNCTION, &plan.report);
                    PreviewPerson {
                        notes: person
                            .notes
                            .into_iter()
                            .filter(|n| !notes.contains(n))
                            .collect(),
                        ..person
                    }
                })
                .collect();
            PreviewOrganisation {
                lines: o.lines.clone(),
                name: o.input.name.clone(),
                city: o.input.city.clone(),
                category: o.category.clone(),
                labels: org_labels(o),
                persons,
                notes,
            }
        })
        .collect()
}

pub fn individuals(plan: &ImportPlan) -> Vec<PreviewIndividual> {
    plan.individuals
        .iter()
        .map(|i| PreviewIndividual {
            person: preview_person(&i.person, "Einzelmitglied", &plan.report),
            city: i.person.input.city.clone(),
            category: i.category.clone(),
        })
        .collect()
}

fn push_new(target: &mut Vec<String>, existing: &[String], name: &str) {
    if !contains(existing, name) && !contains(target, name) {
        target.push(name.to_string());
    }
}

/// Einträge, die der Import neu anlegen würde (Abgleich ohne Gross-/Kleinschreibung).
pub fn new_entries(conn: &Connection, plan: &ImportPlan) -> verwaltung_core::Result<NewEntries> {
    let cats = existing_names(conn, LookupKind::MembershipCategory)?;
    let cantonal = existing_names(conn, LookupKind::CantonalAssociation)?;
    let functions = existing_names(conn, LookupKind::Function)?;
    let labels = existing_labels(conn)?;
    let mut out = NewEntries::default();
    let categories = plan
        .organisations
        .iter()
        .map(|o| &o.category)
        .chain(plan.individuals.iter().map(|i| &i.category))
        .flatten();
    for c in categories {
        push_new(&mut out.categories, &cats, c);
    }
    for c in plan
        .organisations
        .iter()
        .filter_map(|o| o.cantonal.as_ref())
    {
        push_new(&mut out.cantonal, &cantonal, c);
    }
    if plan.organisations.iter().any(|o| !o.persons.is_empty()) {
        push_new(&mut out.functions, &functions, IMPORT_FUNCTION);
    }
    let all_labels = plan.organisations.iter().flat_map(|o| {
        org_labels(o)
            .into_iter()
            .chain(o.persons.iter().flat_map(|p| mandate_labels(&p.mandates)))
    });
    let individual_labels = plan
        .individuals
        .iter()
        .flat_map(|i| mandate_labels(&i.person.mandates));
    for l in all_labels.chain(individual_labels) {
        push_new(&mut out.labels, &labels, &l);
    }
    Ok(out)
}

/// Eine Zeile pro Quellzeile, zum Vergleich mit der Originalliste.
pub fn export_table(plan: &ImportPlan) -> Table {
    let mut rows: Vec<(usize, Vec<String>)> = Vec::new();
    let notes = |lines: &[usize]| notes_for(&plan.report, lines).join("; ");
    for o in &plan.organisations {
        let labels = org_labels(o).join(", ");
        for (i, line) in o.lines.iter().enumerate() {
            let person = o.persons.iter().find(|p| p.line == *line);
            let result = if i == 0 { "Betrieb" } else { "Zusammengefasst" };
            let mut row_labels = labels.clone();
            if let Some(p) = person.filter(|p| !p.mandates.is_empty()) {
                row_labels = [row_labels, mandate_labels(&p.mandates).join(", ")]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            rows.push((
                *line,
                vec![
                    line.to_string(),
                    result.into(),
                    o.input.name.clone(),
                    person.map(|p| person_name(&p.input)).unwrap_or_default(),
                    o.category.clone().unwrap_or_default(),
                    row_labels,
                    notes(&[*line]),
                ],
            ));
        }
    }
    for i in &plan.individuals {
        let line = i.person.line;
        rows.push((
            line,
            vec![
                line.to_string(),
                "Einzelmitglied".into(),
                String::new(),
                person_name(&i.person.input),
                i.category.clone().unwrap_or_default(),
                mandate_labels(&i.person.mandates).join(", "),
                notes(&[line]),
            ],
        ));
    }
    for s in &plan.skipped {
        rows.push((
            s.line,
            vec![
                s.line.to_string(),
                "Übersprungen".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                s.reason.clone(),
            ],
        ));
    }
    rows.sort_by_key(|(line, _)| *line);
    Table {
        headers: [
            "Zeile",
            "Ergebnis",
            "Betrieb",
            "Person",
            "Kategorie",
            "Labels",
            "Hinweise",
        ]
        .iter()
        .map(|h| h.to_string())
        .collect(),
        rows: rows.into_iter().map(|(_, r)| r).collect(),
    }
}

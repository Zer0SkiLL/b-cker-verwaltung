//! Gesamtexport aller Daten als XLSX: lesbar ohne dieses Programm (Exit-Strategie, Archiv).

use std::path::Path;

use chrono::NaiveDate;
use rusqlite::types::ValueRef;
use rusqlite::Connection;

use crate::error::Result;
use crate::export::{write_xlsx_sheets, Table};
use crate::text::format_date;

/// Tabellenblätter in Ausgabereihenfolge.
pub const SHEETS: [&str; 11] = [
    "Betriebe",
    "Personen",
    "Zugehörigkeiten",
    "Mitgliedschaften",
    "Label-Gruppen",
    "Labels",
    "Label-Zuweisungen",
    "Teams",
    "Teammitglieder",
    "Auswahllisten",
    "Gespeicherte Gruppen",
];

const PERSON_NAME: &str = "trim(coalesce(p.first_name || ' ', '') || p.last_name)";

/// (Kopfzeile, SQL, Spalten mit Datum) pro Blatt, in der Reihenfolge von `SHEETS`.
fn definitions() -> [(Vec<&'static str>, String, Vec<usize>); 11] {
    [
        (
            vec!["ID", "Name/Firma", "Zusatz", "Adresse", "Postfach", "PLZ", "Ort", "Telefon", "Mail", "Website",
                 "Filialen", "Anzahl Lernende", "Bemerkungen", "Alt-ID", "Typ", "Kantonalverband",
                 "Handelsregister-Nr", "Kostenstellen-Nr"],
            "SELECT o.id, o.name, o.addition, o.street, o.po_box, o.zip, o.city, o.phone, o.email, o.website, \
             o.branches, o.apprentice_count, o.notes, o.legacy_id, t.name, k.name, o.commercial_register_no, \
             o.cost_center_no FROM organisations o \
             LEFT JOIN lookup_values t ON t.id = o.org_type_id \
             LEFT JOIN lookup_values k ON k.id = o.cantonal_association_id ORDER BY o.id".into(),
            vec![],
        ),
        (
            vec!["ID", "Anrede", "Vorname", "Nachname", "Adresse", "PLZ", "Ort", "Telefon", "Mobiltelefon", "Mail",
                 "Honorar/Lohn", "Bemerkungen"],
            "SELECT id, salutation, first_name, last_name, street, zip, city, phone, mobile, email, compensation, \
             notes FROM persons ORDER BY id".into(),
            vec![],
        ),
        (
            vec!["ID", "Person-ID", "Person", "Betrieb-ID", "Betrieb", "Funktion", "Von", "Bis"],
            format!(
                "SELECT a.id, p.id, {PERSON_NAME}, o.id, o.name, f.name, a.valid_from, a.valid_to FROM affiliations a \
                 JOIN persons p ON p.id = a.person_id JOIN organisations o ON o.id = a.organisation_id \
                 JOIN lookup_values f ON f.id = a.function_id ORDER BY a.id"
            ),
            vec![6, 7],
        ),
        (
            vec!["ID", "Typ", "Mitglied-ID", "Mitglied", "Kategorie", "Eintritt", "Austritt"],
            format!(
                "SELECT m.id, CASE WHEN m.organisation_id IS NOT NULL THEN 'Betrieb' ELSE 'Person' END, \
                 coalesce(m.organisation_id, m.person_id), coalesce(o.name, {PERSON_NAME}), c.name, m.joined_on, \
                 m.left_on FROM memberships m LEFT JOIN organisations o ON o.id = m.organisation_id \
                 LEFT JOIN persons p ON p.id = m.person_id LEFT JOIN lookup_values c ON c.id = m.category_id \
                 ORDER BY m.id"
            ),
            vec![5, 6],
        ),
        (
            vec!["ID", "Name", "Gilt für"],
            "SELECT id, name, CASE applies_to WHEN 'person' THEN 'Personen' WHEN 'organisation' THEN 'Betriebe' \
             ELSE 'Beide' END FROM label_groups ORDER BY id".into(),
            vec![],
        ),
        (
            vec!["ID", "Gruppe-ID", "Gruppe", "Name"],
            "SELECT l.id, g.id, g.name, l.name FROM labels l JOIN label_groups g ON g.id = l.group_id ORDER BY l.id".into(),
            vec![],
        ),
        (
            vec!["ID", "Label-ID", "Gruppe", "Label", "Typ", "Ziel-ID", "Ziel", "Datum", "Notiz"],
            format!(
                "SELECT a.id, l.id, g.name, l.name, CASE WHEN a.organisation_id IS NOT NULL THEN 'Betrieb' \
                 ELSE 'Person' END, coalesce(a.organisation_id, a.person_id), coalesce(o.name, {PERSON_NAME}), \
                 a.assigned_on, a.note FROM label_assignments a JOIN labels l ON l.id = a.label_id \
                 JOIN label_groups g ON g.id = l.group_id LEFT JOIN organisations o ON o.id = a.organisation_id \
                 LEFT JOIN persons p ON p.id = a.person_id ORDER BY a.id"
            ),
            vec![7],
        ),
        (vec!["ID", "Name", "Beschreibung"], "SELECT id, name, description FROM teams ORDER BY id".into(), vec![]),
        (
            vec!["ID", "Team-ID", "Team", "Person-ID", "Person", "Rolle"],
            format!(
                "SELECT tm.id, t.id, t.name, p.id, {PERSON_NAME}, tm.role FROM team_members tm \
                 JOIN teams t ON t.id = tm.team_id JOIN persons p ON p.id = tm.person_id ORDER BY tm.id"
            ),
            vec![],
        ),
        (
            vec!["ID", "Art", "Name", "Sortierung"],
            "SELECT id, CASE kind WHEN 'org_type' THEN 'Organisationstyp' WHEN 'function' THEN 'Funktion' \
             WHEN 'membership_category' THEN 'Mitgliedschaftskategorie' ELSE 'Kantonalverband' END, name, sort \
             FROM lookup_values ORDER BY kind, sort, name".into(),
            vec![],
        ),
        (vec!["ID", "Name", "Filter"], "SELECT id, name, filter_json FROM saved_groups ORDER BY id".into(), vec![]),
    ]
}

pub fn full_export(conn: &Connection, path: &Path) -> Result<()> {
    let sheets = SHEETS
        .iter()
        .zip(definitions())
        .map(|(name, (headers, sql, dates))| {
            Ok((name.to_string(), query_table(conn, &headers, &sql, &dates)?))
        })
        .collect::<Result<Vec<_>>>()?;
    write_xlsx_sheets(&sheets, path)
}

fn query_table(
    conn: &Connection,
    headers: &[&str],
    sql: &str,
    date_cols: &[usize],
) -> Result<Table> {
    let mut stmt = conn.prepare(sql)?;
    let width = headers.len();
    let rows = stmt.query_map([], |row| {
        (0..width)
            .map(|i| row.get_ref(i).map(|v| cell_text(v, date_cols.contains(&i))))
            .collect()
    })?;
    Ok(Table {
        headers: headers.iter().map(|h| h.to_string()).collect(),
        rows: rows.collect::<rusqlite::Result<Vec<Vec<String>>>>()?,
    })
}

fn cell_text(value: ValueRef, is_date: bool) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(i) => i.to_string(),
        ValueRef::Real(f) => f.to_string(),
        ValueRef::Text(t) | ValueRef::Blob(t) => {
            let text = String::from_utf8_lossy(t).into_owned();
            match NaiveDate::parse_from_str(&text, "%Y-%m-%d") {
                Ok(d) if is_date => format_date(d),
                _ => text,
            }
        }
    }
}

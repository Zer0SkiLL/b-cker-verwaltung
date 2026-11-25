//! Abgleich mit bestehenden Listeneinträgen und Labels ohne Rücksicht auf Gross-/Kleinschreibung
//! (nur für den Import; die Eingabe im Programm bleibt unverändert).

use rusqlite::Connection;

use crate::error::Result;
use crate::model::{AppliesTo, Label, LookupKind, LookupValue};
use crate::repo::{labels, lookups};

fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Bestehender Eintrag mit gleichem Namen (ohne Gross-/Kleinschreibung) oder ein neuer.
pub fn lookup_or_create(conn: &Connection, kind: LookupKind, name: &str) -> Result<LookupValue> {
    match lookups::list(conn, kind)?
        .into_iter()
        .find(|v| same(&v.name, name))
    {
        Some(existing) => Ok(existing),
        None => lookups::create(conn, kind, name),
    }
}

/// Label in der Gruppe (beide ohne Gross-/Kleinschreibung gesucht), bei Bedarf neu angelegt.
pub fn label_or_create(
    conn: &Connection,
    group: &str,
    applies_to: AppliesTo,
    name: &str,
) -> Result<Label> {
    let group_id = match labels::list_groups(conn)?
        .into_iter()
        .find(|g| same(&g.name, group))
    {
        Some(g) => g.id,
        None => labels::create_group(conn, group, applies_to)?.id,
    };
    match labels::list_labels(conn, group_id)?
        .into_iter()
        .find(|l| same(&l.name, name))
    {
        Some(existing) => Ok(existing),
        None => labels::create_label(conn, group_id, name),
    }
}

/// Namen der vorhandenen Einträge einer Liste (für „neu angelegt werden“ in der Vorschau).
pub fn existing_names(conn: &Connection, kind: LookupKind) -> Result<Vec<String>> {
    Ok(lookups::list(conn, kind)?
        .into_iter()
        .map(|v| v.name)
        .collect())
}

/// „Gruppe: Label“ aller vorhandenen Labels.
pub fn existing_labels(conn: &Connection) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for g in labels::list_groups(conn)? {
        for l in labels::list_labels(conn, g.id)? {
            out.push(format!("{}: {}", g.name, l.name));
        }
    }
    Ok(out)
}

/// Ob `name` (ohne Gross-/Kleinschreibung) in `existing` vorkommt.
pub fn contains(existing: &[String], name: &str) -> bool {
    existing.iter().any(|e| same(e, name))
}

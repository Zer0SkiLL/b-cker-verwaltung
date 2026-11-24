//! Datenzugriff pro Fachbereich. Alle Funktionen nehmen `&Connection` und liefern neue Werte.

pub mod affiliations;
pub mod assignments;
pub mod labels;
pub mod lookups;
pub mod memberships;
pub mod organisations;
pub mod persons;
pub mod settings;
pub mod teams;

use crate::error::{CoreError, Result};

/// Getrimmter Pflichtname; leer ergibt einen Validierungsfehler.
pub(crate) fn require_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Validation("Name darf nicht leer sein".into()));
    }
    Ok(trimmed.to_string())
}

/// Lehnt das Löschen eines verwendeten Eintrags ohne Bestätigung ab.
pub(crate) fn guard_in_use(count: i64, confirm: bool) -> Result<()> {
    if count > 0 && !confirm {
        return Err(CoreError::InUse { count });
    }
    Ok(())
}

/// Prüft, dass ein Bis-Datum nicht vor dem Von-Datum liegt.
pub(crate) fn check_date_order(
    from: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
    message: &str,
) -> Result<()> {
    match (from, to) {
        (Some(f), Some(t)) if t < f => Err(CoreError::Validation(message.into())),
        _ => Ok(()),
    }
}

/// Prüft vor dem Umbenennen, dass kein anderer Eintrag der Tabelle den Namen schon trägt
/// (sonst käme die englische UNIQUE-Meldung von SQLite bis zur Oberfläche).
pub(crate) fn ensure_unique_name(
    conn: &rusqlite::Connection,
    table: &str,
    name: &str,
    id: i64,
) -> Result<()> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE name = ?1 AND id <> ?2");
    let taken: i64 = conn.query_row(&sql, rusqlite::params![name, id], |r| r.get(0))?;
    if taken > 0 {
        return Err(CoreError::Validation("Name ist bereits vergeben".into()));
    }
    Ok(())
}

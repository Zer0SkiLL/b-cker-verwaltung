//! Datenzugriff pro Fachbereich. Alle Funktionen nehmen `&Connection` und liefern neue Werte.

pub mod affiliations;
pub mod labels;
pub mod lookups;
pub mod memberships;
pub mod organisations;
pub mod persons;

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

//! Datenzugriff pro Fachbereich. Alle Funktionen nehmen `&Connection` und liefern neue Werte.

pub mod labels;
pub mod lookups;

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

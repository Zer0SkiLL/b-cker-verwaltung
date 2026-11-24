//! Text-Hilfen: Suchnormalisierung, Bereinigung und Schweizer Datumsformat.

use chrono::NaiveDate;

/// Schweizer Datumsformat TT.MM.JJJJ.
pub const DATE_FORMAT: &str = "%d.%m.%Y";

/// Verbindet nicht-leere Teile mit Leerzeichen und wandelt Unicode-korrekt in Kleinbuchstaben.
pub fn normalize_search(parts: &[Option<&str>]) -> String {
    parts
        .iter()
        .flatten()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn format_date(d: NaiveDate) -> String {
    d.format(DATE_FORMAT).to_string()
}

/// Trimmt einen optionalen Text; leere Texte werden zu `None`.
pub fn clean(value: &Option<String>) -> Option<String> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(String::from)
}

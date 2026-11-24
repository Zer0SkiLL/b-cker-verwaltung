//! Betriebe und Personen: Detailansichten und Bearbeiten.

mod detail;
mod write;

pub use detail::*;
pub use write::*;

/// Sortierschlüssel nach Schweizer Gewohnheit: Umlaute wie Grundbuchstaben, ohne Gross/klein.
pub(crate) fn sort_key(name: &str) -> String {
    name.to_lowercase()
        .replace('ä', "a")
        .replace('ö', "o")
        .replace('ü', "u")
        .replace('é', "e")
        .replace('è', "e")
}

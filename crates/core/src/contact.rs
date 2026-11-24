//! Hilfen zum Kontaktieren: E-Mail-Liste für Outlook-BCC und mailto-Link.

use std::collections::HashSet;

use crate::query::filter::ContactRow;

/// Maximale Länge eines mailto-Links, die Mailprogramme zuverlässig verarbeiten.
pub const MAILTO_MAX_LEN: usize = 2000;

#[derive(Debug, Clone, PartialEq)]
pub struct EmailList {
    pub addresses: Vec<String>,
    /// Adressen mit "; " getrennt, direkt in Outlook-BCC einfügbar.
    pub joined: String,
    /// Einträge ohne gültige E-Mail-Adresse.
    pub missing: Vec<ContactRow>,
}

/// Bereinigt, entfernt Duplikate (ohne Gross-/Kleinschreibung) und sammelt fehlende Adressen.
pub fn email_list(rows: &[ContactRow]) -> EmailList {
    let mut seen = HashSet::new();
    let mut addresses = Vec::new();
    let mut missing = Vec::new();
    for row in rows {
        match row.email.as_deref().map(str::trim).filter(|e| e.contains('@')) {
            Some(email) => {
                if seen.insert(email.to_lowercase()) {
                    addresses.push(email.to_string());
                }
            }
            None => missing.push(row.clone()),
        }
    }
    let joined = addresses.join("; ");
    EmailList { addresses, joined, missing }
}

/// `mailto:?bcc=…`, oder `None`, wenn der Link zu lang würde.
pub fn mailto_bcc(addresses: &[String]) -> Option<String> {
    let encoded: Vec<String> = addresses.iter().map(|a| percent_encode(a)).collect();
    let link = format!("mailto:?bcc={}", encoded.join(","));
    (link.len() <= MAILTO_MAX_LEN).then_some(link)
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

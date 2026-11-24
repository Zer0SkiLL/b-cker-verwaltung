use verwaltung_core::storage::lock::Identity;

const UNKNOWN: &str = "unbekannt";

/// Benutzer und Rechner für die Sperrdatei, aus der Umgebung gelesen.
pub fn identity_from_env() -> Identity {
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| UNKNOWN.to_string());
    let host = gethostname::gethostname()
        .into_string()
        .ok()
        .filter(|h| !h.trim().is_empty())
        .unwrap_or_else(|| UNKNOWN.to_string());
    Identity { user, host }
}

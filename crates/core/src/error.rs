use crate::storage::lock::LockInfo;

/// Alle Fehler des Datenkerns. `Display` liefert deutsche, benutzerverständliche Texte.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Datenbankfehler: {0}")]
    Db(rusqlite::Error),
    #[error("Dateifehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("Die Datei ist schreibgeschützt geöffnet.")]
    ReadOnly,
    #[error("Die Datei wird bereits von {} auf {} verwendet.", .0.user, .0.host)]
    LockHeld(LockInfo),
    #[error("{entity} mit ID {id} wurde nicht gefunden.")]
    NotFound { entity: &'static str, id: i64 },
    #[error("Der Eintrag wird noch {count}-mal verwendet.")]
    InUse { count: i64 },
    #[error("Die Datenbank ist nicht leer. Import nur in eine leere Datenbank möglich.")]
    DatabaseNotEmpty,
    #[error("{0}")]
    Validation(String),
    #[error("Importfehler: {0}")]
    Import(String),
}

impl From<rusqlite::Error> for CoreError {
    fn from(err: rusqlite::Error) -> Self {
        match err.sqlite_error_code() {
            Some(rusqlite::ErrorCode::ReadOnly) => CoreError::ReadOnly,
            _ => CoreError::Db(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;

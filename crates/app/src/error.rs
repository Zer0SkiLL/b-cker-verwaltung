use serde::{Serialize, Serializer};
use verwaltung_core::error::CoreError;

/// Fehler, wie er an die Oberfläche geht: nur der deutsche Text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError(pub String);

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(message: impl Into<String>) -> AppError {
        AppError(message.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> AppError {
        AppError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> AppError {
        AppError(format!("Dateifehler: {err}"))
    }
}

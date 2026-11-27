//! `#[tauri::command]`-Hüllen. Keine Logik: Sperre holen, Befehl aufrufen, Ergebnis zurück.

pub mod catalog;
pub mod data;
pub mod diagnose;
pub mod entities;
pub mod session;

use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::session::Session;

/// Programmzustand für Tauri.
pub struct AppState {
    pub session: Mutex<Session>,
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> AppState {
        AppState {
            session: Mutex::new(Session::default()),
            config_dir,
        }
    }

    /// Führt `f` mit der Sitzung aus.
    pub fn with<T>(&self, f: impl FnOnce(&mut Session) -> AppResult<T>) -> AppResult<T> {
        let mut session = self.session.lock().map_err(|_| {
            AppError::new("Interner Fehler: Sitzung blockiert. Bitte Programm neu starten.")
        })?;
        let result = f(&mut session);
        if let Err(err) = &result {
            // Ohne Pfad und Namen: das Protokoll geht in Bericht und Analysedaten.
            log::warn!(
                "Fehler an die Oberfläche: {}",
                crate::commands::logfile::sanitize(&err.0)
            );
        }
        result
    }
}

pub(crate) fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

/// Schliesst die offene Datei (Sicherung am zweiten Ort, Sperre freigeben).
/// Wird beim Fenster-Schliessen und beim Programmende aufgerufen; mehrfach unschädlich.
pub fn close_session(state: &AppState) {
    if let Err(err) = state.with(|s| s.close(chrono::Local::now())) {
        log::error!(
            "Fehler beim Schliessen: {}",
            crate::commands::logfile::sanitize(&err.0)
        );
    }
}

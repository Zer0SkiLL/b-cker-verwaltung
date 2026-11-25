use std::path::Path;

use chrono::{DateTime, Local, Utc};
use rusqlite::Connection;
use verwaltung_core::error::Result as CoreResult;
use verwaltung_core::storage::backup;
use verwaltung_core::storage::lock::Identity;
use verwaltung_core::storage::workspace::{AccessMode, SecondaryResult, Workspace};

use crate::dto::{BackupStatusDto, LockDto, SecondaryResultDto, SessionInfo};
use crate::error::{AppError, AppResult};

pub const NO_FILE: &str = "Keine Datei geöffnet.";
pub const READ_ONLY: &str = "Die Datei ist nur lesend geöffnet. Änderungen sind nicht möglich.";
pub const FILE_EXISTS: &str = "Die Datei existiert bereits. Bitte „Öffnen“ verwenden.";

/// Die geöffnete Datenbank der Programmsitzung (höchstens eine).
#[derive(Default)]
pub struct Session {
    ws: Option<Workspace>,
    who: Option<Identity>,
    /// Letzter Fehler am zweiten Sicherungsort in dieser Sitzung (für die gelbe Leiste).
    secondary_error: Option<String>,
}

impl Session {
    /// Öffnet eine bestehende Datei; eine offene Sitzung wird vorher geschlossen.
    /// Fehlt die Datei (verschoben, umbenannt), wird bewusst keine neue angelegt.
    pub fn open(&mut self, path: &Path, who: &Identity) -> AppResult<SessionInfo> {
        if !path.is_file() {
            return Err(AppError(format!(
                "Datei nicht gefunden: {}. Bitte „Datei öffnen…“ verwenden.",
                path.display()
            )));
        }
        self.open_path(path, who)
    }

    fn open_path(&mut self, path: &Path, who: &Identity) -> AppResult<SessionInfo> {
        self.close(Local::now())?;
        self.ws = Some(Workspace::open(path, who)?);
        self.who = Some(who.clone());
        self.info(Utc::now())
    }

    /// Legt eine neue Datei an. Verweigert, wenn sie schon existiert.
    pub fn create(&mut self, path: &Path, who: &Identity) -> AppResult<SessionInfo> {
        if path.exists() {
            return Err(AppError::new(FILE_EXISTS));
        }
        self.open_path(path, who)?;
        self.write(verwaltung_core::repo::defaults::seed_new_file)?;
        self.info(Utc::now())
    }

    /// Übernimmt die Sperre der geöffneten Datei und öffnet schreibend.
    pub fn take_over(&mut self, who: &Identity) -> AppResult<SessionInfo> {
        let path = self.workspace()?.path().to_path_buf();
        // Die bisherige (nur lesende) Verbindung zuerst schliessen.
        self.ws = None;
        match Workspace::open_taking_over(&path, who) {
            Ok(ws) => self.ws = Some(ws),
            Err(err) => {
                // Datei wieder nur lesend öffnen (ohne neue Sperre), damit die Ansichten weiter funktionieren.
                return Err(match Workspace::open_read_only_at(&path) {
                    Ok(ws) => {
                        self.ws = Some(ws);
                        AppError(format!("Übernehmen nicht möglich: {err}"))
                    }
                    Err(reopen) => AppError(format!(
                        "Übernehmen nicht möglich: {err}. Datei konnte auch nicht wieder geöffnet werden: {reopen}"
                    )),
                });
            }
        }
        self.who = Some(who.clone());
        // Nach einem Absturz fehlt die Sicherung am zweiten Ort vom Schliessen: jetzt nachholen.
        let outcome = self.writable()?.backup_now(Local::now())?;
        self.note_secondary(&outcome.secondary);
        self.info(Utc::now())
    }

    /// Merkt sich das Ergebnis am zweiten Ort: Fehler bleibt bis zur nächsten gelungenen Sicherung.
    pub fn note_secondary(&mut self, result: &SecondaryResult) {
        match result {
            SecondaryResult::Unavailable(reason) => self.secondary_error = Some(reason.clone()),
            SecondaryResult::Done(_) => self.secondary_error = None,
            SecondaryResult::NotConfigured => {}
        }
    }

    /// Aktueller Zustand; prüft dabei, ob die eigene Sperre noch besteht.
    pub fn info(&mut self, now: DateTime<Utc>) -> AppResult<SessionInfo> {
        let own = self.who.clone();
        let secondary_error = self.secondary_error.clone();
        let ws = self.workspace_mut()?;
        let held_by = match ws.check_lock()? {
            AccessMode::ReadWrite => None,
            AccessMode::ReadOnly { held_by } => Some(LockDto::new(held_by, own.as_ref(), now)),
        };
        Ok(SessionInfo {
            path: ws.path().display().to_string(),
            read_only: held_by.is_some(),
            held_by,
            backup: BackupStatusDto {
                secondary_error,
                ..backup::status(&ws.conn, now)?.into()
            },
        })
    }

    pub fn read<T>(&self, f: impl FnOnce(&Connection) -> CoreResult<T>) -> AppResult<T> {
        Ok(f(&self.workspace()?.conn)?)
    }

    /// Schreibt nur, wenn die eigene Sperre noch besteht.
    pub fn write<T>(&mut self, f: impl FnOnce(&Connection) -> CoreResult<T>) -> AppResult<T> {
        let ws = self.writable()?;
        Ok(f(&ws.conn)?)
    }

    pub fn write_mut<T>(
        &mut self,
        f: impl FnOnce(&mut Connection) -> CoreResult<T>,
    ) -> AppResult<T> {
        let ws = self.writable()?;
        Ok(f(&mut ws.conn)?)
    }

    pub fn workspace_mut(&mut self) -> AppResult<&mut Workspace> {
        self.ws.as_mut().ok_or_else(|| AppError::new(NO_FILE))
    }

    /// Geöffnete Sitzung mit gültiger Sperre (für Sicherung/Wiederherstellen).
    pub fn writable(&mut self) -> AppResult<&mut Workspace> {
        let ws = self.workspace_mut()?;
        if !matches!(ws.check_lock()?, AccessMode::ReadWrite) {
            return Err(AppError::new(READ_ONLY));
        }
        Ok(ws)
    }

    /// Schliesst die Datei (Sicherung am zweiten Ort); `None`, wenn nichts offen war.
    pub fn close(&mut self, now: DateTime<Local>) -> AppResult<Option<SecondaryResultDto>> {
        match self.ws.take() {
            Some(ws) => Ok(Some(ws.close(now)?.into())),
            None => Ok(None),
        }
    }

    fn workspace(&self) -> AppResult<&Workspace> {
        self.ws.as_ref().ok_or_else(|| AppError::new(NO_FILE))
    }
}

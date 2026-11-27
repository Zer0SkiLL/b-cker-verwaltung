//! Datei öffnen/anlegen und für den nächsten Start merken.

use std::path::Path;

use serde::Serialize;
use verwaltung_core::storage::lock::Identity;

use crate::config::save_last_path;
use crate::dto::SessionInfo;
use crate::error::AppResult;
use crate::session::Session;

pub const REMEMBER_FAILED: &str =
    "Die Datei ist geöffnet, wird beim nächsten Start aber nicht vorgeschlagen.";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenResult {
    pub info: SessionInfo,
    /// Hinweis, wenn das Öffnen geklappt hat, aber etwas Nebensächliches nicht.
    pub warning: Option<String>,
}

/// Öffnet (bzw. legt an mit `create`) und merkt sich den Pfad. Scheitert nur das Merken,
/// bleibt die Datei offen und es gibt einen Hinweis statt eines Fehlers.
pub fn open_and_remember(
    s: &mut Session,
    config_dir: &Path,
    path: &Path,
    who: &Identity,
    create: bool,
) -> AppResult<OpenResult> {
    let started = std::time::Instant::now();
    let info = if create {
        s.create(path, who)?
    } else {
        s.open(path, who)?
    };
    s.set_open_ms(started.elapsed().as_millis());
    let warning = save_last_path(config_dir, path)
        .err()
        .map(|_| REMEMBER_FAILED.to_string());
    Ok(OpenResult { info, warning })
}

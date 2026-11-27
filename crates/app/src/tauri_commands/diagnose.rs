//! Hüllen für Frühwarnung und Analysedaten. Die Webhook-Adresse verlässt Rust nie.

use std::path::PathBuf;

use chrono::Local;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::AppState;
use crate::commands::diagnose::{
    self, heartbeat_due, publish_with, settings, spawn_flush, webhook, Kind, Message,
    DIAGNOSE_DEFAULT, DIAGNOSE_UNTIL,
};
use crate::error::{AppError, AppResult};
use crate::identity::identity_from_env;

/// Abstand für das Nachsenden aus dem Ausgang.
const RETRY_EVERY: std::time::Duration = std::time::Duration::from_secs(15 * 60);

fn outbox_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("ausgang"))
        .map_err(|e| AppError(format!("Programmordner unbekannt: {e}")))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnoseInfoDto {
    /// Ob es die Funktion in diesem Build und bis zum Enddatum überhaupt gibt.
    pub available: bool,
    pub enabled: bool,
    /// JJJJ-MM-TT.
    pub until: String,
    pub last_sent: Option<String>,
}

#[tauri::command(async)]
pub fn diagnose_settings(state: State<AppState>) -> DiagnoseInfoDto {
    let s = settings::load(&state.config_dir);
    DiagnoseInfoDto {
        available: webhook().is_some() && Local::now().date_naive() <= DIAGNOSE_UNTIL,
        enabled: s.enabled.unwrap_or(DIAGNOSE_DEFAULT),
        until: DIAGNOSE_UNTIL.to_string(),
        last_sent: s.last_sent,
    }
}

#[tauri::command(async)]
pub fn set_diagnose_enabled(state: State<AppState>, enabled: bool) -> AppResult<()> {
    let s = settings::load(&state.config_dir);
    settings::save(
        &state.config_dir,
        &settings::DiagnoseSettings {
            enabled: Some(enabled),
            ..s
        },
    )?;
    log::info!(
        "Analysedaten senden: {}",
        if enabled { "ein" } else { "aus" }
    );
    Ok(())
}

#[tauri::command(async)]
pub fn diagnose_heartbeat_due(state: State<AppState>) -> bool {
    let s = settings::load(&state.config_dir);
    diagnose::active(&s, Local::now().date_naive()) && heartbeat_due(&s, Local::now())
}

#[tauri::command(async)]
pub fn diagnostics_publish(
    app: AppHandle,
    state: State<AppState>,
    kind: Kind,
    status_block: String,
    report: String,
) -> AppResult<()> {
    let db = state.with(|s| Ok(s.workspace_mut().ok().map(|w| w.path().to_path_buf())))?;
    let msg = Message {
        kind,
        host: identity_from_env().host,
        version: env!("CARGO_PKG_VERSION").to_string(),
        status_block,
        report,
        at: Local::now(),
    };
    publish_with(
        webhook(),
        &state.config_dir,
        &outbox_dir(&app)?,
        db.as_deref(),
        msg,
    );
    Ok(())
}

/// Beim Start und danach alle 15 min: liegengebliebene Nachrichten nachsenden.
pub fn start_retry(app: &AppHandle) {
    let (Some(url), Ok(outbox)) = (webhook(), outbox_dir(app)) else {
        return;
    };
    let config_dir = app.state::<AppState>().config_dir.clone();
    let spawned = std::thread::Builder::new()
        .name("analysedaten".into())
        .spawn(move || loop {
            let s = settings::load(&config_dir);
            if diagnose::active(&s, Local::now().date_naive()) {
                let _ = spawn_flush(url.to_string(), config_dir.clone(), outbox.clone()).join();
            }
            std::thread::sleep(RETRY_EVERY);
        });
    if let Err(e) = spawned {
        log::info!("Nachsenden der Analysedaten nicht gestartet: {e}");
    }
}

/// Protokoll-Ziel: meldet Warnungen und Fehler der Oberfläche als Ereignis `diagnose-ereignis`.
pub fn log_target(app: std::sync::Arc<std::sync::OnceLock<AppHandle>>) -> fern::Dispatch {
    use tauri::Emitter;
    fern::Dispatch::new().chain(fern::Output::call(move |record| {
        let line = record.args().to_string();
        let text = crate::commands::logfile::message_text(&line);
        if let (Some(kind), Some(handle)) = (
            diagnose::event_kind(record.level(), record.target(), text),
            app.get(),
        ) {
            let _ = handle.emit("diagnose-ereignis", serde_json::json!({ "kind": kind }));
        }
    }))
}

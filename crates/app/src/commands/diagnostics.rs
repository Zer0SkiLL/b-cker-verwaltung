//! Angaben für „Problem melden“: relevante Protokollzeilen und System. Enthält keine Vereinsdaten.

use std::path::Path;

use serde::Serialize;

use super::logfile::{excerpt_indices, read_log_lines};
use crate::error::AppResult;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SystemInfoDto {
    pub os: String,
    pub arch: String,
    pub version: String,
}

pub fn system_info() -> SystemInfoDto {
    let info = os_info::get();
    SystemInfoDto {
        os: format!("{} {}", info.os_type(), info.version()),
        arch: std::env::consts::ARCH.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Letzte Warnungen/Fehler (höchstens 40) und die letzten 10 Zeilen, roh und zeitlich geordnet.
pub fn log_tail(log_dir: &Path) -> AppResult<Vec<String>> {
    let lines = read_log_lines(log_dir)?;
    Ok(excerpt_indices(&lines)
        .into_iter()
        .map(|i| lines[i].clone())
        .collect())
}

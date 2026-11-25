//! Angaben für „Problem melden“: relevante Protokollzeilen und System. Enthält keine Vereinsdaten.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::error::AppResult;

/// Höchstens so viele Warnungen/Fehler im Bericht.
const MAX_PROBLEMS: usize = 40;
/// Dazu immer die letzten Zeilen jeder Stufe.
const LAST_LINES: usize = 10;

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

fn is_problem(line: &str) -> bool {
    line.contains("[WARN]") || line.contains("[ERROR]")
}

/// Letzte Warnungen/Fehler (höchstens 40) und die letzten 10 Zeilen, ohne Doppelte, zeitlich geordnet.
/// Liest alle `.log`-Dateien des Ordners (rotierte zuerst, nach Änderungszeit).
pub fn log_tail(log_dir: &Path) -> AppResult<Vec<String>> {
    if !log_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<_> = fs::read_dir(log_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "log"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort();
    let mut lines: Vec<String> = Vec::new();
    for (_, path) in files {
        let text = String::from_utf8_lossy(&fs::read(path)?).into_owned();
        lines.extend(
            text.lines()
                .filter(|l| !l.trim().is_empty())
                .map(String::from),
        );
    }
    let problems: Vec<usize> = (0..lines.len())
        .filter(|&i| is_problem(&lines[i]))
        .collect();
    let mut keep: Vec<usize> = problems[problems.len().saturating_sub(MAX_PROBLEMS)..].to_vec();
    keep.extend(lines.len().saturating_sub(LAST_LINES)..lines.len());
    keep.sort_unstable();
    keep.dedup();
    Ok(keep.into_iter().map(|i| lines[i].clone()).collect())
}

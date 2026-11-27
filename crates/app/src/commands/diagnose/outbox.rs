//! Ausgang: Nachrichten warten hier, bis sie gesendet sind (auch über Neustarts und Offline-Zeiten).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use super::{Message, MAX_OUTBOX};
use crate::error::{AppError, AppResult};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Legt die Nachricht ab; über 20 fallen die ältesten weg.
pub fn enqueue(dir: &Path, msg: &Message) -> AppResult<()> {
    std::fs::create_dir_all(dir)?;
    let stamp = msg.at.format("%Y%m%d-%H%M%S%3f");
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("{stamp}-{}-{n:06}.json", std::process::id());
    let text = serde_json::to_string(msg)
        .map_err(|e| AppError(format!("Nachricht nicht speicherbar: {e}")))?;
    std::fs::write(dir.join(name), text)?;
    let files = files(dir);
    for old in files.iter().take(files.len().saturating_sub(MAX_OUTBOX)) {
        remove(old);
    }
    Ok(())
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

/// Wartende Nachrichten, älteste zuerst; unlesbare Dateien werden entfernt.
pub fn pending(dir: &Path) -> Vec<(PathBuf, Message)> {
    files(dir)
        .into_iter()
        .filter_map(|path| {
            let parsed = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_json::from_str::<Message>(&t).ok());
            if parsed.is_none() {
                remove(&path);
            }
            parsed.map(|m| (path, m))
        })
        .collect()
}

pub fn remove(path: &Path) {
    let _ = std::fs::remove_file(path);
}

//! Wo die Datei liegt (lokal, Netzlaufwerk, Netzpfad), ohne den Pfad selbst zu nennen.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationKind {
    NetworkDrive(char),
    NetworkPath,
    Removable,
    Local,
    Unknown,
}

impl LocationKind {
    pub fn label(&self) -> String {
        match self {
            LocationKind::NetworkDrive(letter) => format!("Netzlaufwerk ({letter}:)"),
            LocationKind::NetworkPath => "Netzpfad".into(),
            LocationKind::Removable => "externes Laufwerk oder Netz".into(),
            LocationKind::Local => "lokal".into(),
            LocationKind::Unknown => "unbekannt".into(),
        }
    }
}

fn drive_letter(path: &str) -> Option<char> {
    let mut chars = path.chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    (chars.next() == Some(':')).then(|| letter.to_ascii_uppercase())
}

/// Rein aus dem Pfad; `drive_is_remote` ist die Antwort von Windows für den Laufwerksbuchstaben.
pub fn classify(path: &str, drive_is_remote: Option<bool>) -> LocationKind {
    if path.starts_with(r"\\") || path.starts_with("//") {
        return LocationKind::NetworkPath;
    }
    if let Some(letter) = drive_letter(path) {
        return match drive_is_remote {
            Some(true) => LocationKind::NetworkDrive(letter),
            Some(false) => LocationKind::Local,
            None => LocationKind::Unknown,
        };
    }
    if ["/Volumes/", "/mnt/", "/media/", "/run/media/"]
        .iter()
        .any(|p| path.starts_with(p))
    {
        return LocationKind::Removable;
    }
    if path.starts_with('/') {
        LocationKind::Local
    } else {
        LocationKind::Unknown
    }
}

#[cfg(windows)]
fn drive_is_remote(path: &str) -> Option<bool> {
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
    use windows_sys::Win32::System::WindowsProgramming::{
        DRIVE_NO_ROOT_DIR, DRIVE_REMOTE, DRIVE_UNKNOWN,
    };
    let letter = drive_letter(path)?;
    let root: Vec<u16> = format!("{letter}:\\").encode_utf16().chain([0]).collect();
    // SAFETY: `root` ist ein nullterminierter UTF-16-Text, der während des Aufrufs lebt.
    let kind = unsafe { GetDriveTypeW(root.as_ptr()) };
    match kind {
        DRIVE_UNKNOWN | DRIVE_NO_ROOT_DIR => None,
        k => Some(k == DRIVE_REMOTE),
    }
}

#[cfg(not(windows))]
fn drive_is_remote(_path: &str) -> Option<bool> {
    None
}

pub fn location_of(path: &Path) -> LocationKind {
    let text = path.to_string_lossy();
    classify(&text, drive_is_remote(&text))
}

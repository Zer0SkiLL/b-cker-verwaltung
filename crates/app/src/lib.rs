//! Tauri-Schicht der Mitgliederverwaltung. Die Logik liegt in einfachen Funktionen
//! auf [`session::Session`] und ist ohne Tauri-Laufzeit testbar.

pub mod config;
pub mod dto;
pub mod error;
pub mod identity;
pub mod session;

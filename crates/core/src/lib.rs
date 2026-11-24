//! Datenkern der Mitgliederverwaltung: Speicherung, Abfragen, Import und Export.

pub mod db;
pub mod error;
pub mod model;
pub mod repo;
pub mod storage;
pub mod text;

pub use error::{CoreError, Result};

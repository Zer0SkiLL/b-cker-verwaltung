//! Datenkern der Mitgliederverwaltung: Speicherung, Abfragen, Import und Export.

pub mod db;
pub mod error;
pub mod model;
pub mod repo;
pub mod storage;

pub use error::{CoreError, Result};

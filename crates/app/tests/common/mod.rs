#![allow(dead_code)]
use std::path::{Path, PathBuf};

use verwaltung_app::session::Session;
use verwaltung_core::storage::lock::Identity;

pub fn who(user: &str) -> Identity {
    Identity {
        user: user.into(),
        host: format!("pc-{user}"),
    }
}

pub fn db_path(dir: &Path) -> PathBuf {
    dir.join("verband.sqlite")
}

/// Neue Datei, schreibend geöffnet von „anna".
pub fn fresh_session(dir: &Path) -> Session {
    let mut session = Session::default();
    session.create(&db_path(dir), &who("anna")).unwrap();
    session
}

pub fn org_input(name: &str) -> verwaltung_core::repo::organisations::OrganisationInput {
    verwaltung_core::repo::organisations::OrganisationInput {
        name: name.into(),
        ..Default::default()
    }
}

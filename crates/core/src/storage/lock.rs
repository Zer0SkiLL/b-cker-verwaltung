use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Inhalt einer Sperrdatei: wer die Datenbank geöffnet hat.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LockInfo {
    pub user: String,
    pub host: String,
    pub created_at: DateTime<Utc>,
}

/// Identität der Person, die eine Datenbank öffnet.
#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    pub user: String,
    pub host: String,
}

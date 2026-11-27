//! Nachricht an einen Discord-Webhook: Embed mit dem Status-Block, dazu der ganze Bericht als Datei.
//! Fehlermeldungen enthalten nie die Adresse (`without_url`).

use std::error::Error as _;
use std::time::Duration;

use serde_json::{json, Value};

use super::{safe_name, Kind, Message};

const TIMEOUT: Duration = Duration::from_secs(10);
/// Discord erlaubt 4096 Zeichen in der Beschreibung; Platz für den Codeblock lassen.
const MAX_BLOCK: usize = 4000;

fn style(kind: Kind) -> (&'static str, u32) {
    match kind {
        Kind::Absturz => ("🔴", 0xD64545),
        Kind::Warnung | Kind::Selbsttest => ("🟠", 0xE08A1E),
        Kind::Lebenszeichen => ("🟢", 0x3BA55C),
    }
}

pub fn file_name(msg: &Message) -> String {
    format!(
        "Problembericht-{}-{}.txt",
        safe_name(&msg.host),
        msg.at.format("%Y%m%d-%H%M")
    )
}

/// JSON für `payload_json` und der Dateiname des Anhangs.
pub fn payload(msg: &Message) -> (Value, String) {
    let (icon, color) = style(msg.kind);
    let block: String = msg.status_block.chars().take(MAX_BLOCK).collect();
    let name = file_name(msg);
    let value = json!({
        "username": "Mitgliederverwaltung",
        "embeds": [{
            "title": format!("{icon} {} · {} · {}", msg.kind.label(), msg.host, msg.version),
            "description": format!("```\n{block}\n```"),
            "color": color,
            "timestamp": msg.at.to_rfc3339(),
        }],
        "attachments": [{ "id": 0, "filename": name }],
    });
    (value, name)
}

fn describe(e: reqwest::Error) -> String {
    let e = e.without_url();
    let mut text = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        text.push_str(&format!(": {s}"));
        source = s.source();
    }
    text
}

pub fn send(url: &str, msg: &Message) -> Result<(), String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (json, name) = payload(msg);
    let file = reqwest::blocking::multipart::Part::bytes(msg.report.clone().into_bytes())
        .file_name(name)
        .mime_str("text/plain; charset=utf-8")
        .map_err(describe)?;
    let form = reqwest::blocking::multipart::Form::new()
        .text("payload_json", json.to_string())
        .part("files[0]", file);
    let client = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(describe)?;
    let response = client.post(url).multipart(form).send().map_err(describe)?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Antwort {}", response.status().as_u16()))
    }
}

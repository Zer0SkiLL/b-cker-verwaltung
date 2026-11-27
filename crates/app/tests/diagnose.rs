//! Analysedaten: Schalter und Enddatum, Ausgang, Discord-Nachricht, Diagnose-Ordner.
//! Gesendet wird nur an einen lokalen Testserver; die echte Adresse kommt nur im Release-Build hinein.
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Mutex, OnceLock};

use chrono::{Duration, Local, NaiveDate, TimeZone};
use verwaltung_app::commands::diagnose::outbox::{enqueue, pending};
use verwaltung_app::commands::diagnose::settings::{load, save, DiagnoseSettings};
use verwaltung_app::commands::diagnose::{
    active_with, discord, event_kind, flush, heartbeat_due, write_folder_file, Kind, Message,
    DIAGNOSE_UNTIL, MAX_OUTBOX, MAX_PER_DAY,
};

/// Sammelt alle Protokollzeilen dieses Testprozesses.
struct Capture(Mutex<Vec<String>>);

impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        self.0
            .lock()
            .unwrap()
            .push(format!("{} {}", record.level(), record.args()));
    }
    fn flush(&self) {}
}

fn captured() -> &'static Capture {
    static LOGGER: OnceLock<&'static Capture> = OnceLock::new();
    LOGGER.get_or_init(|| {
        let logger: &'static Capture = Box::leak(Box::new(Capture(Mutex::new(Vec::new()))));
        log::set_logger(logger).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
        logger
    })
}

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn message(kind: Kind) -> Message {
    Message {
        kind,
        host: "PC-TEST".into(),
        version: "0.6.0".into(),
        status_block: "  [ OK ] Datei   lokal".into(),
        report: "PROBLEMBERICHT · fiktiv".into(),
        at: Local.with_ymd_and_hms(2026, 11, 27, 14, 5, 0).unwrap(),
    }
}

/// Einfacher HTTP-Server: nimmt `count` Anfragen an, antwortet mit `status`, liefert die Rümpfe.
fn server(status: u16, count: usize) -> (String, std::thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/api/webhooks/1/geheim-token",
        listener.local_addr().unwrap()
    );
    let handle = std::thread::spawn(move || {
        let mut bodies = Vec::new();
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            let mut data = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                let n = stream.read(&mut buf).unwrap();
                data.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&data).to_string();
                if let Some(head_end) = text.find("\r\n\r\n") {
                    let len = text[..head_end]
                        .lines()
                        .find_map(|l| {
                            l.to_lowercase()
                                .strip_prefix("content-length: ")
                                .map(|v| v.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if data.len() >= head_end + 4 + len {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            bodies.push(String::from_utf8_lossy(&data).to_string());
            let reply =
                format!("HTTP/1.1 {status} X\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
            stream.write_all(reply.as_bytes()).unwrap();
        }
        bodies
    });
    (url, handle)
}

#[test]
fn active_only_with_webhook_switch_and_date() {
    let s = DiagnoseSettings::default();
    let today = day(2026, 12, 1);
    assert!(active_with(Some("https://x"), &s, today));
    assert!(!active_with(None, &s, today));
    assert!(!active_with(Some(""), &s, today));
    assert!(active_with(Some("https://x"), &s, DIAGNOSE_UNTIL));
    assert!(!active_with(
        Some("https://x"),
        &s,
        DIAGNOSE_UNTIL + Duration::days(1)
    ));
    let off = DiagnoseSettings {
        enabled: Some(false),
        ..s
    };
    assert!(!active_with(Some("https://x"), &off, today));
    assert_eq!(DIAGNOSE_UNTIL, day(2027, 5, 31));
}

#[test]
fn settings_round_trip_and_tolerate_garbage() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(load(dir.path()), DiagnoseSettings::default());
    let s = DiagnoseSettings {
        enabled: Some(false),
        ..Default::default()
    };
    save(dir.path(), &s).unwrap();
    assert_eq!(load(dir.path()), s);
    std::fs::write(dir.path().join("diagnose.json"), "kaputt").unwrap();
    assert_eq!(load(dir.path()), DiagnoseSettings::default());
}

#[test]
fn heartbeat_due_once_a_week() {
    let now = Local::now();
    assert!(heartbeat_due(&DiagnoseSettings::default(), now));
    let recent = DiagnoseSettings {
        last_heartbeat: Some((now - Duration::days(2)).to_rfc3339()),
        ..Default::default()
    };
    assert!(!heartbeat_due(&recent, now));
    let old = DiagnoseSettings {
        last_heartbeat: Some((now - Duration::days(8)).to_rfc3339()),
        ..Default::default()
    };
    assert!(heartbeat_due(&old, now));
}

#[test]
fn event_kind_from_log_records() {
    use log::Level::*;
    assert_eq!(
        event_kind(
            Warn,
            "verwaltung_app::tauri_commands",
            "Fehler an die Oberfläche: X"
        ),
        Some(Kind::Warnung)
    );
    assert_eq!(
        event_kind(
            Error,
            "verwaltung_app::commands::diagnostics",
            "Absturz: boom (a.rs:1)"
        ),
        Some(Kind::Absturz)
    );
    assert_eq!(
        event_kind(
            Warn,
            "verwaltung_app::commands::selftest",
            "Selbsttest: wenig Platz"
        ),
        Some(Kind::Selbsttest)
    );
    assert_eq!(event_kind(Info, "verwaltung_app", "Datei geöffnet"), None);
    // keine Schleife: Versand und Netzwerk-Bibliotheken lösen nichts aus
    assert_eq!(
        event_kind(Warn, "verwaltung_app::commands::diagnose", "x"),
        None
    );
    assert_eq!(
        event_kind(Warn, "verwaltung_app::commands::diagnose::discord", "x"),
        None
    );
    assert_eq!(event_kind(Warn, "reqwest::connect", "x"), None);
    assert_eq!(event_kind(Warn, "hyper_util::client", "x"), None);
    assert_eq!(event_kind(Warn, "rustls::conn", "x"), None);
}

#[test]
fn outbox_keeps_at_most_twenty() {
    let dir = tempfile::tempdir().unwrap();
    for _ in 0..(MAX_OUTBOX + 1) {
        enqueue(dir.path(), &message(Kind::Warnung)).unwrap();
    }
    assert_eq!(pending(dir.path()).len(), MAX_OUTBOX);
}

#[test]
fn payload_has_title_colour_codeblock_and_attachment() {
    let (json, file_name) = discord::payload(&message(Kind::Absturz));
    let embed = &json["embeds"][0];
    assert_eq!(embed["title"], "🔴 Absturz · PC-TEST · 0.6.0");
    assert_eq!(embed["color"], 0xD64545);
    assert!(embed["description"]
        .as_str()
        .unwrap()
        .starts_with("```\n  [ OK ] Datei"));
    assert!(embed["description"].as_str().unwrap().ends_with("\n```"));
    assert_eq!(file_name, "Problembericht-PC-TEST-20261127-1405.txt");
    assert_eq!(json["attachments"][0]["filename"], file_name);
    assert_eq!(
        discord::payload(&message(Kind::Lebenszeichen)).0["embeds"][0]["color"],
        0x3BA55C
    );
    assert_eq!(
        discord::payload(&message(Kind::Warnung)).0["embeds"][0]["title"],
        "🟠 Warnung · PC-TEST · 0.6.0"
    );
    let long = Message {
        status_block: "x".repeat(5000),
        ..message(Kind::Warnung)
    };
    assert!(
        discord::payload(&long).0["embeds"][0]["description"]
            .as_str()
            .unwrap()
            .chars()
            .count()
            <= 4096
    );
    let odd = Message {
        host: "PC Ä/1".into(),
        ..message(Kind::Warnung)
    };
    assert_eq!(
        discord::payload(&odd).1,
        "Problembericht-PC___1-20261127-1405.txt"
    );
}

#[test]
fn flush_sends_and_deletes() {
    let outbox = tempfile::tempdir().unwrap();
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    let (url, server) = server(204, 1);
    let mut s = DiagnoseSettings::default();
    let today = Local::now();
    assert_eq!(flush(outbox.path(), &url, &mut s, today), 1);
    assert!(pending(outbox.path()).is_empty());
    assert!(s.last_sent.is_some());
    let body = &server.join().unwrap()[0];
    assert!(body.contains("Problembericht-PC-TEST-"), "{body}");
    assert!(body.contains("PROBLEMBERICHT · fiktiv"));
    assert!(body.contains("payload_json"));
}

#[test]
fn flush_keeps_messages_on_server_error() {
    let outbox = tempfile::tempdir().unwrap();
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    let (url, server) = server(500, 1);
    let mut s = DiagnoseSettings::default();
    assert_eq!(flush(outbox.path(), &url, &mut s, Local::now()), 0);
    assert_eq!(pending(outbox.path()).len(), 2);
    server.join().unwrap();
}

#[test]
fn heartbeat_send_updates_last_heartbeat() {
    let outbox = tempfile::tempdir().unwrap();
    enqueue(outbox.path(), &message(Kind::Lebenszeichen)).unwrap();
    let (url, server) = server(200, 1);
    let mut s = DiagnoseSettings::default();
    flush(outbox.path(), &url, &mut s, Local::now());
    assert!(s.last_heartbeat.is_some());
    server.join().unwrap();
}

#[test]
fn daily_limit_drops_the_rest() {
    let outbox = tempfile::tempdir().unwrap();
    let now = Local::now();
    let mut s = DiagnoseSettings {
        sent_day: Some(now.date_naive().to_string()),
        sent_count: MAX_PER_DAY,
        ..Default::default()
    };
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    // keine Anfrage: der Server würde blockieren, wenn doch gesendet würde
    assert_eq!(flush(outbox.path(), "http://127.0.0.1:9/x", &mut s, now), 0);
    assert!(pending(outbox.path()).is_empty());
    // neuer Tag, neues Glück
    let mut s2 = DiagnoseSettings {
        sent_day: Some("2000-01-01".into()),
        sent_count: MAX_PER_DAY,
        ..Default::default()
    };
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    let (url, server) = server(204, 1);
    assert_eq!(flush(outbox.path(), &url, &mut s2, now), 1);
    assert_eq!(s2.sent_count, 1);
    server.join().unwrap();
}

#[test]
fn unreachable_webhook_never_appears_in_the_log() {
    let capture = captured();
    let outbox = tempfile::tempdir().unwrap();
    enqueue(outbox.path(), &message(Kind::Warnung)).unwrap();
    let mut s = DiagnoseSettings::default();
    assert_eq!(
        flush(
            outbox.path(),
            "http://127.0.0.1:9/api/webhooks/1/geheim-token",
            &mut s,
            Local::now()
        ),
        0
    );
    let lines = capture.0.lock().unwrap().clone();
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Analysedaten nicht gesendet")),
        "{lines:?}"
    );
    for line in &lines {
        assert!(
            !line.contains("geheim-token"),
            "Webhook im Protokoll: {line}"
        );
        assert!(
            !line.contains("127.0.0.1:9"),
            "Adresse im Protokoll: {line}"
        );
    }
    assert_eq!(pending(outbox.path()).len(), 1);
}

#[test]
fn folder_file_is_written_next_to_the_data_file() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("verband.sqlite");
    write_folder_file(&db, "PC-TEST", "Bericht");
    let file = dir
        .path()
        .join("Mitgliederverwaltung-Diagnose")
        .join("PC-TEST.txt");
    assert_eq!(std::fs::read_to_string(file).unwrap(), "Bericht");
    // nicht beschreibbar: kein Absturz
    std::fs::write(dir.path().join("blockiert"), "x").unwrap();
    write_folder_file(
        &dir.path().join("blockiert").join("a.sqlite"),
        "PC-TEST",
        "Bericht",
    );
}

#[test]
fn publish_without_webhook_only_writes_the_folder_file() {
    use verwaltung_app::commands::diagnose::publish_with;
    let dir = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let outbox = tempfile::tempdir().unwrap();
    let db = dir.path().join("verband.sqlite");
    let handle = publish_with(
        None,
        config.path(),
        outbox.path(),
        Some(&db),
        message(Kind::Warnung),
    );
    assert!(handle.is_none());
    assert!(dir
        .path()
        .join("Mitgliederverwaltung-Diagnose/PC-TEST.txt")
        .exists());
    assert!(pending(outbox.path()).is_empty());
}

#[test]
fn publish_with_webhook_sends_in_the_background() {
    use verwaltung_app::commands::diagnose::publish_with;
    let config = tempfile::tempdir().unwrap();
    let outbox = tempfile::tempdir().unwrap();
    let (url, server) = server(204, 1);
    let handle = publish_with(
        Some(&url),
        config.path(),
        outbox.path(),
        None,
        message(Kind::Absturz),
    )
    .unwrap();
    handle.join().unwrap();
    assert!(pending(outbox.path()).is_empty());
    assert!(load(config.path()).last_sent.is_some());
    assert!(server.join().unwrap()[0].contains("Absturz"));
}

#[test]
fn publish_respects_the_switch() {
    use verwaltung_app::commands::diagnose::publish_with;
    let config = tempfile::tempdir().unwrap();
    let outbox = tempfile::tempdir().unwrap();
    save(
        config.path(),
        &DiagnoseSettings {
            enabled: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(publish_with(
        Some("http://127.0.0.1:9/x"),
        config.path(),
        outbox.path(),
        None,
        message(Kind::Warnung)
    )
    .is_none());
    assert!(pending(outbox.path()).is_empty());
}

#[test]
fn message_text_strips_the_log_prefix() {
    use verwaltung_app::commands::logfile::message_text;
    assert_eq!(
        message_text("[2026-11-27][10:00:00][m::x][ERROR] Absturz: boom"),
        "Absturz: boom"
    );
    assert_eq!(message_text("Absturz: roh"), "Absturz: roh");
}

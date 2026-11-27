//! Programm-Protokoll und Systemangaben für „Problem melden“.
mod common;

use std::sync::{Mutex, OnceLock};

use chrono::Local;
use common::{fresh_session, org_input, write_xlsx, LIST_HEADERS};
use verwaltung_app::commands::addresses::{export_addresses, ExportFormat};
use verwaltung_app::commands::backup::{backup_now, export_all};
use verwaltung_app::commands::diagnostics::{
    install_panic_hook, log_from_ui, log_tail, save_report, system_info,
};
use verwaltung_app::commands::import::{import_apply, import_preview};
use verwaltung_core::query::filter::Filter;
use verwaltung_core::repo::organisations;

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

#[test]
fn logs_contain_no_business_data() {
    let capture = captured();
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| organisations::create(c, &org_input("Bäckerei Geheim AG")))
        .unwrap();
    // Zweiter Ort mit „geheimem“ Namen, danach nicht erreichbar (Datei statt Ordner).
    let usb = dir.path().join("usb-Geheimordner");
    std::fs::create_dir(&usb).unwrap();
    verwaltung_app::commands::backup::set_secondary_dir(&mut s, Some(&usb)).unwrap();
    std::fs::remove_dir(&usb).unwrap();
    std::fs::write(&usb, b"kein Ordner").unwrap();
    backup_now(&mut s, Local::now()).unwrap();
    // Import in eine zweite, leere Datei.
    let dir2 = tempfile::tempdir().unwrap();
    let mut s2 = fresh_session(dir2.path());
    let file = dir2.path().join("liste.xlsx");
    let mut row = vec![""; LIST_HEADERS.len()];
    row[2] = "Confiserie Geheim";
    write_xlsx(&file, &LIST_HEADERS, &[row]);
    let p = import_preview(&mut s2, &file, None).unwrap();
    import_apply(&mut s2, &file, p.mapping, Local::now()).unwrap();
    // Exporte
    let today = Local::now().date_naive();
    export_addresses(
        &s,
        &Filter::default(),
        today,
        &dir.path().join("Geheim-liste.csv"),
        ExportFormat::Csv,
    )
    .unwrap();
    export_all(&s, &dir.path().join("Geheim-alles.xlsx")).unwrap();
    let lines = capture.0.lock().unwrap().clone();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("INFO Export Adressen: 1 Zeilen")),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("INFO Gesamtexport:")),
        "{lines:?}"
    );
    assert!(!lines.is_empty(), "es wurde nichts protokolliert");
    for line in &lines {
        assert!(
            !line.contains("Geheim"),
            "Vereinsdaten im Protokoll: {line}"
        );
        assert!(
            !line.contains(&*dir.path().to_string_lossy()),
            "Pfad im Protokoll: {line}"
        );
    }
}

fn write_log(dir: &std::path::Path, lines: &[String]) {
    std::fs::write(dir.join("Mitgliederverwaltung.log"), lines.join("\n")).unwrap();
}

#[test]
fn log_tail_prefers_warnings_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let lines: Vec<String> = (0..100)
        .map(|i| {
            let level = if i % 5 == 0 { "WARN" } else { "INFO" };
            format!("[2026-11-25][10:00:{i:02}][verwaltung][{level}] Zeile {i}")
        })
        .collect();
    write_log(dir.path(), &lines);
    let tail = log_tail(dir.path()).unwrap();
    let warns = tail.iter().filter(|l| l.contains("[WARN]")).count();
    assert_eq!(warns, 20);
    // die letzten 10 Zeilen jeder Stufe sind dabei
    assert!(tail.iter().any(|l| l.ends_with("Zeile 99")));
    assert!(tail.iter().any(|l| l.ends_with("Zeile 91")));
    // keine Doppelten, zeitliche Reihenfolge
    let mut sorted = tail.clone();
    sorted.dedup();
    assert_eq!(sorted.len(), tail.len());
    assert!(tail.first().unwrap().ends_with("Zeile 0"));
    assert!(tail.last().unwrap().ends_with("Zeile 99"));
}

#[test]
fn log_tail_caps_warnings_at_40() {
    let dir = tempfile::tempdir().unwrap();
    let lines: Vec<String> = (0..60).map(|i| format!("[x][ERROR] Fehler {i}")).collect();
    write_log(dir.path(), &lines);
    assert_eq!(log_tail(dir.path()).unwrap().len(), 40);
}

#[test]
fn log_tail_without_file_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    assert!(log_tail(&dir.path().join("fehlt")).unwrap().is_empty());
}

#[test]
fn system_info_has_version_and_os() {
    let info = system_info();
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    assert!(!info.os.is_empty());
    assert!(!info.arch.is_empty());
}

#[test]
fn opening_and_closing_are_logged() {
    let capture = captured();
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let mut ben = verwaltung_app::session::Session::default();
    ben.open(&common::db_path(dir.path()), &common::who("ben"))
        .unwrap();
    s.close(Local::now()).unwrap();
    let lines = capture.0.lock().unwrap().clone();
    let has = |text: &str| lines.iter().any(|l| l.contains(text));
    assert!(has("INFO Datei geöffnet (schreibend)"), "{lines:?}");
    assert!(has("INFO Datei geöffnet (nur lesend)"), "{lines:?}");
    assert!(has("INFO Datei geschlossen"), "{lines:?}");
}

#[test]
fn clock_behind_newest_backup_is_logged() {
    let capture = captured();
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let saved = backup_now(&mut s, Local::now()).unwrap().local;
    let future = std::path::Path::new(&saved).with_file_name("verband-20991231-120000.sqlite");
    std::fs::copy(&saved, &future).unwrap();
    backup_now(&mut s, Local::now()).unwrap();
    let lines = capture.0.lock().unwrap().clone();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("WARN") && l.contains("Uhrzeit des Computers")),
        "{lines:?}"
    );
}

#[test]
fn panics_are_logged_as_crash() {
    let capture = captured();
    install_panic_hook();
    let _ = std::panic::catch_unwind(|| panic!("boom"));
    let lines = capture.0.lock().unwrap().clone();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("ERROR Absturz: boom (") && l.contains("diagnostics.rs:")),
        "{lines:?}"
    );
}

#[test]
fn messages_from_the_ui_are_logged_short_and_masked() {
    let capture = captured();
    log_from_ui(
        "warn",
        &format!(r"Oberfläche C:\Users\Hans\x {}", "y".repeat(600)),
    );
    log_from_ui("error", "kaputt");
    log_from_ui("sonstwas", "nur Info");
    let lines = capture.0.lock().unwrap().clone();
    let warn = lines
        .iter()
        .find(|l| l.starts_with("WARN Oberfläche"))
        .unwrap();
    assert!(warn.contains(r"C:\Users\…\x"), "{warn}");
    assert!(warn.chars().count() <= 505, "{}", warn.chars().count());
    assert!(lines.iter().any(|l| l == "ERROR kaputt"));
    assert!(lines.iter().any(|l| l == "INFO nur Info"));
}

#[test]
fn save_report_writes_utf8() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Problembericht.txt");
    save_report(&path, "Grüsse ╔═╗").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "Grüsse ╔═╗");
}

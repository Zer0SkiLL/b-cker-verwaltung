//! Lesbares Protokoll, Masken für Benutzerordner und erkannte Programmstarts.
use chrono::NaiveDate;
use verwaltung_app::commands::logfile::{
    log_excerpt, mask_user_dirs, pretty_line, problems_today, read_log_lines, start_history,
};

fn line(date: &str, time: &str, level: &str, text: &str) -> String {
    format!("[{date}][{time}][verwaltung_app::modul][{level}] {text}")
}

#[test]
fn pretty_line_drops_module_and_translates_level() {
    let raw = line(
        "2026-11-26",
        "21:33:53",
        "WARN",
        "Fehler an die Oberfläche: X",
    );
    assert_eq!(
        pretty_line(&raw),
        "26.11. 21:33:53  WARNUNG  Fehler an die Oberfläche: X"
    );
    assert_eq!(
        pretty_line(&line("2026-11-26", "08:01:12", "INFO", "Programmstart")),
        "26.11. 08:01:12  INFO     Programmstart"
    );
    assert_eq!(
        pretty_line(&line("2026-01-05", "09:00:00", "ERROR", "Absturz")),
        "05.01. 09:00:00  FEHLER   Absturz"
    );
}

#[test]
fn pretty_line_keeps_unreadable_lines() {
    assert_eq!(pretty_line("kaputt"), "kaputt");
    assert_eq!(pretty_line("[x][ERROR] Fehler 1"), "[x][ERROR] Fehler 1");
}

#[test]
fn user_folders_are_masked() {
    assert_eq!(
        mask_user_dirs(r"Dateifehler C:\Users\Hans\Desktop\a.sqlite"),
        r"Dateifehler C:\Users\…\Desktop\a.sqlite"
    );
    assert_eq!(mask_user_dirs("/Users/hans/x"), "/Users/…/x");
    assert_eq!(mask_user_dirs("a /home/anna/x b"), "a /home/…/x b");
    assert_eq!(mask_user_dirs(r"c:\users\hans"), r"c:\users\…");
    assert_eq!(mask_user_dirs("Datei geöffnet"), "Datei geöffnet");
    // auch in aufbereiteten Zeilen
    let raw = line("2026-11-26", "10:00:00", "WARN", r"Fehler: C:\Users\Hans\a");
    assert!(pretty_line(&raw).ends_with(r"Fehler: C:\Users\…\a"));
}

#[test]
fn excerpt_shows_gaps_and_keeps_order() {
    let mut lines: Vec<String> = (0..60)
        .map(|i| {
            line(
                "2026-11-26",
                &format!("10:{i:02}:00"),
                "INFO",
                &format!("Zeile {i}"),
            )
        })
        .collect();
    lines[5] = line("2026-11-26", "10:05:00", "WARN", "Achtung");
    let out = log_excerpt(&lines);
    assert_eq!(out[0], "26.11. 10:05:00  WARNUNG  Achtung");
    assert_eq!(out[1], "…");
    assert_eq!(out.len(), 12);
    assert!(out[2].ends_with("Zeile 50"));
    assert!(out.last().unwrap().ends_with("Zeile 59"));
}

#[test]
fn excerpt_without_gap_has_no_ellipsis() {
    let lines: Vec<String> = (0..5)
        .map(|i| line("2026-11-26", "10:00:00", "INFO", &format!("Z {i}")))
        .collect();
    assert!(!log_excerpt(&lines).contains(&"…".to_string()));
}

#[test]
fn start_history_detects_unclean_ends() {
    let lines = vec![
        line(
            "2026-11-25",
            "08:00:00",
            "INFO",
            "Programmstart, Version 0.5.0",
        ),
        line("2026-11-25", "17:00:00", "INFO", "Programmende"),
        line(
            "2026-11-26",
            "08:00:00",
            "INFO",
            "Programmstart, Version 0.5.0",
        ),
        line(
            "2026-11-26",
            "09:00:00",
            "INFO",
            "Datei geöffnet (schreibend)",
        ),
        line(
            "2026-11-27",
            "08:01:00",
            "INFO",
            "Programmstart, Version 0.6.0",
        ),
    ];
    let starts = start_history(&lines, 5);
    assert_eq!(starts.len(), 3);
    assert!(starts[0].at.starts_with("2026-11-27T08:01:00"));
    assert_eq!(starts[0].clean_before, Some(false));
    assert_eq!(starts[1].clean_before, Some(true));
    assert_eq!(starts[2].clean_before, None);
    assert_eq!(start_history(&lines, 2).len(), 2);
}

#[test]
fn problems_today_counts_only_today() {
    let lines = vec![
        line("2026-11-26", "10:00:00", "WARN", "gestern"),
        line("2026-11-27", "10:00:00", "WARN", "heute"),
        line("2026-11-27", "11:00:00", "ERROR", "heute"),
        line("2026-11-27", "12:00:00", "INFO", "heute"),
    ];
    let today = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
    assert_eq!(problems_today(&lines, today), 2);
}

#[test]
fn read_log_lines_reads_all_files_and_skips_empty() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.log"), "eins\n\nzwei\n").unwrap();
    std::fs::write(dir.path().join("andere.txt"), "nein\n").unwrap();
    assert_eq!(read_log_lines(dir.path()).unwrap(), vec!["eins", "zwei"]);
    assert!(read_log_lines(&dir.path().join("fehlt"))
        .unwrap()
        .is_empty());
}

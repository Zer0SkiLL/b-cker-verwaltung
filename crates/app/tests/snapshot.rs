//! Momentaufnahme für den Problembericht: Programm-Prozessbaum, Ort der Datei, Datei-Angaben.
mod common;

use chrono::Local;
use common::{fresh_session, org_input};
use verwaltung_app::commands::diagnostics::{file_facts, snapshot};
use verwaltung_app::commands::location::{classify, LocationKind};
use verwaltung_app::commands::resources::{measure, peak, record_peak, tree_sum};
use verwaltung_core::repo::organisations;

#[test]
fn tree_sum_counts_own_process_and_descendants() {
    let procs = [
        (10, None, 100, 1.0),
        (11, Some(10), 50, 2.0),
        (12, Some(11), 25, 0.5),
        (99, Some(1), 999, 9.0),
    ];
    assert_eq!(tree_sum(10, &procs), (175, 3.5));
    assert_eq!(tree_sum(42, &procs), (0, 0.0));
}

#[test]
fn classify_locations() {
    assert_eq!(
        classify(r"\\server\share\a.sqlite", None),
        LocationKind::NetworkPath
    );
    let s = classify(r"S:\Verein\a.sqlite", Some(true));
    assert_eq!(s, LocationKind::NetworkDrive('S'));
    assert_eq!(s.label(), "Netzlaufwerk (S:)");
    assert_eq!(classify(r"c:\a.sqlite", Some(false)), LocationKind::Local);
    assert_eq!(
        classify("/Volumes/USB/a.sqlite", None),
        LocationKind::Removable
    );
    assert_eq!(classify("/mnt/x/a.sqlite", None), LocationKind::Removable);
    assert_eq!(classify("/home/x/a.sqlite", None), LocationKind::Local);
    assert_eq!(classify(r"S:\a.sqlite", None), LocationKind::Unknown);
    assert_eq!(LocationKind::NetworkPath.label(), "Netzpfad");
    assert_eq!(LocationKind::Local.label(), "lokal");
}

#[test]
fn file_facts_without_file_is_none() {
    let mut s = verwaltung_app::session::Session::default();
    assert!(file_facts(&mut s, Local::now().date_naive()).is_none());
}

#[test]
fn file_facts_describe_the_file_without_its_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| organisations::create(c, &org_input("Bäckerei Geheim AG")))
        .unwrap();
    let facts = file_facts(&mut s, Local::now().date_naive()).unwrap();
    assert!(facts.reachable);
    assert!(facts.size.unwrap() > 0);
    assert_eq!(facts.organisations, Some(1));
    assert_eq!(facts.persons, Some(0));
    assert_eq!(facts.active_members, Some(0));
    assert_eq!(
        facts.schema_version,
        Some(verwaltung_core::db::SCHEMA_VERSION)
    );
    assert!(facts.lock_host.is_none());
    let json = serde_json::to_string(&facts).unwrap();
    assert!(!json.contains("Geheim"), "{json}");
    let dir_name = dir.path().file_name().unwrap().to_string_lossy();
    assert!(!json.contains(&*dir_name), "{json}");
}

#[test]
fn file_facts_name_the_lock_host_but_not_the_user() {
    let dir = tempfile::tempdir().unwrap();
    let _anna = fresh_session(dir.path());
    let mut ben = verwaltung_app::session::Session::default();
    ben.open(&common::db_path(dir.path()), &common::who("ben"))
        .unwrap();
    let facts = file_facts(&mut ben, Local::now().date_naive()).unwrap();
    assert_eq!(facts.lock_host.as_deref(), Some("pc-anna"));
    assert!(facts.lock_since.is_some());
    let json = serde_json::to_string(&facts).unwrap();
    assert!(!json.contains("\"anna\""), "{json}");
}

#[test]
fn measure_reports_memory_and_updates_peak() {
    let r = measure();
    assert!(r.mem_total.unwrap() > 0);
    assert!(r.mem_program.unwrap() > 0);
    assert!(r.program_since.is_some());
    assert!(peak() >= r.mem_program.unwrap());
    record_peak(u64::MAX / 2);
    assert_eq!(peak(), u64::MAX / 2);
}

#[test]
fn snapshot_combines_everything() {
    let dir = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    std::fs::write(
        logs.path().join("a.log"),
        "[2026-11-27][08:00:00][m][INFO] Programmstart, Version 0.5.0\n",
    )
    .unwrap();
    let mut s = fresh_session(dir.path());
    let snap = snapshot(&mut s, logs.path(), "PC-TEST");
    assert_eq!(snap.host, "PC-TEST");
    assert_eq!(snap.starts.len(), 1);
    assert_eq!(snap.log.len(), 1);
    assert!(snap.file.is_some());
    let json = serde_json::to_value(&snap).unwrap();
    assert!(json.get("problemsToday").is_some());
    assert!(json["resources"].get("memProgramPeak").is_some());
}

#[test]
fn threads_are_not_counted_as_extra_processes() {
    // Unter Linux führt sysinfo Threads als eigene Prozesse mit dem Speicher des ganzen Prozesses.
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let threads: Vec<_> = (0..16)
        .map(|_| {
            let stop = stop.clone();
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            })
        })
        .collect();
    let r = measure();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    threads.into_iter().for_each(|t| t.join().unwrap());
    let (program, used) = (r.mem_program.unwrap(), r.mem_used.unwrap());
    assert!(program <= used, "Programm {program} > belegt {used}");
}

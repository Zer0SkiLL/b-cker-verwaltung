//! Programm-Protokoll für „Problem melden“: lesbar aufbereiten, Benutzerordner maskieren,
//! Programmstarts erkennen. Zeilen des Log-Plugins: `[JJJJ-MM-TT][hh:mm:ss][modul][STUFE] Text`.

use std::fs;
use std::path::Path;

use std::sync::LazyLock;

use chrono::{Local, NaiveDate, NaiveDateTime, TimeZone};
use regex::Regex;
use serde::Serialize;

use crate::error::AppResult;

pub const START_MARKER: &str = "Programmstart";
pub const END_MARKER: &str = "Programmende";
/// Höchstens so viele Warnungen/Fehler im Auszug.
const MAX_PROBLEMS: usize = 40;
/// Dazu immer die letzten Zeilen.
const LAST_LINES: usize = 10;
const GAP: &str = "…";
const LEVELS: [&str; 5] = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDto {
    /// Startzeit, RFC 3339 in lokaler Zeit.
    pub at: String,
    /// Ob der Lauf davor mit „Programmende“ endete; `None` beim ältesten Start im Protokoll.
    pub clean_before: Option<bool>,
}

struct Parsed<'a> {
    date: NaiveDate,
    time: &'a str,
    level: &'a str,
    text: &'a str,
}

fn parse(line: &str) -> Option<Parsed<'_>> {
    let mut rest = line;
    let mut groups = Vec::with_capacity(4);
    for _ in 0..4 {
        let inner = rest.strip_prefix('[')?;
        let end = inner.find(']')?;
        groups.push(&inner[..end]);
        rest = &inner[end + 1..];
    }
    let date = NaiveDate::parse_from_str(groups[0], "%Y-%m-%d").ok()?;
    // Das Log-Plugin schreibt je nach Zeitzonen-Einstellung [Modul][STUFE] oder [STUFE][Modul].
    let level = [groups[3], groups[2]]
        .into_iter()
        .find(|g| LEVELS.contains(g))?;
    Some(Parsed {
        date,
        time: groups[1],
        level,
        text: rest.strip_prefix(' ').unwrap_or(rest),
    })
}

/// Nur der Text einer Protokollzeile (ohne Datum, Zeit, Modul, Stufe).
pub fn message_text(line: &str) -> &str {
    parse(line).map_or(line, |p| p.text)
}

/// Alle nicht leeren Zeilen aller `.log`-Dateien des Ordners (rotierte zuerst, nach Änderungszeit).
pub fn read_log_lines(log_dir: &Path) -> AppResult<Vec<String>> {
    if !log_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<_> = fs::read_dir(log_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "log"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    files.sort();
    let mut lines = Vec::new();
    for (_, path) in files {
        let text = String::from_utf8_lossy(&fs::read(path)?).into_owned();
        lines.extend(
            text.lines()
                .filter(|l| !l.trim().is_empty())
                .map(String::from),
        );
    }
    Ok(lines)
}

/// Absolute Pfade (Laufwerk, Netzpfad, `/…/…`) bis zum nächsten Trenner. Leerzeichen gehören
/// dazu (Ordnernamen mit Leerzeichen); Schluss bei Anführungszeichen, Klammern, „: “, „, “, „; “.
static PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(^|[^A-Za-z0-9])((?:[A-Za-z]:[\\/]|\\\\|/[\w.~-]+/)(?:[^"'“”‘’«»<>|():,;\n]|[:,;][^ \n])*)"#)
        .expect("gültiges Muster")
});
/// Inhaber der Sperre in „…bereits von <Name> auf <PC>…“.
static LOCK_HOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(bereits von ).+?( auf )").expect("gültiges Muster"));

/// Entfernt Pfade und den Namen des Sperr-Inhabers aus Texten, die das Programm verlassen
/// (Protokoll-Auszug, Bericht, Analysedaten).
pub fn sanitize(text: &str) -> String {
    let without_paths = PATH.replace_all(text, "${1}‹Pfad›");
    LOCK_HOLDER
        .replace_all(&without_paths, "${1}‹Person›${2}")
        .trim_end()
        .to_string()
}

fn level_label(level: &str) -> &'static str {
    match level {
        "WARN" => "WARNUNG",
        "ERROR" => "FEHLER",
        _ => "INFO",
    }
}

/// `26.11. 21:33:53  WARNUNG  Text`; nicht lesbare Zeilen bleiben (maskiert) wie sie sind.
pub fn pretty_line(line: &str) -> String {
    let text = match parse(line) {
        Some(p) => format!(
            "{}  {:<8} {}",
            format_args!("{} {}", p.date.format("%d.%m."), p.time),
            level_label(p.level),
            p.text
        ),
        None => line.to_string(),
    };
    sanitize(&text)
}

fn is_problem(line: &str) -> bool {
    line.contains("[WARN]") || line.contains("[ERROR]")
}

/// Zeilennummern für den Auszug: letzte 40 Warnungen/Fehler und die letzten 10 Zeilen, geordnet.
pub fn excerpt_indices(lines: &[String]) -> Vec<usize> {
    let problems: Vec<usize> = (0..lines.len())
        .filter(|&i| is_problem(&lines[i]))
        .collect();
    let mut keep = problems[problems.len().saturating_sub(MAX_PROBLEMS)..].to_vec();
    keep.extend(lines.len().saturating_sub(LAST_LINES)..lines.len());
    keep.sort_unstable();
    keep.dedup();
    keep
}

/// Aufbereiteter Auszug, älteste zuerst; ausgelassene Zeilen als „…“.
pub fn log_excerpt(lines: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut previous: Option<usize> = None;
    for i in excerpt_indices(lines) {
        if previous.is_some_and(|p| i > p + 1) {
            out.push(GAP.to_string());
        }
        out.push(pretty_line(&lines[i]));
        previous = Some(i);
    }
    out
}

fn local_rfc3339(date: NaiveDate, time: &str) -> Option<String> {
    let naive =
        NaiveDateTime::parse_from_str(&format!("{date} {time}"), "%Y-%m-%d %H:%M:%S").ok()?;
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|d| d.to_rfc3339())
}

/// Erst ab Version 0.6.0 steht „Programmende“ im Protokoll.
fn writes_end_marker(start_text: &str) -> bool {
    let version = start_text.rsplit(' ').next().unwrap_or("");
    let mut parts = version.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    (major, minor) >= (0, 6)
}

/// Die letzten `limit` Programmstarts, neueste zuerst. Ob der Lauf davor sauber endete, ist
/// nur bekannt, wenn dessen Version „Programmende“ schon protokollierte.
pub fn start_history(lines: &[String], limit: usize) -> Vec<StartDto> {
    let mut starts: Vec<StartDto> = Vec::new();
    let mut ended_since_start = false;
    let mut previous_knows_end: Option<bool> = None;
    for p in lines.iter().filter_map(|l| parse(l)) {
        if p.text.starts_with(START_MARKER) {
            let clean_before =
                previous_knows_end.and_then(|knows| knows.then_some(ended_since_start));
            if let Some(at) = local_rfc3339(p.date, p.time) {
                starts.push(StartDto { at, clean_before });
            }
            previous_knows_end = Some(writes_end_marker(p.text));
            ended_since_start = false;
        } else if p.text.starts_with(END_MARKER) {
            ended_since_start = true;
        }
    }
    starts.into_iter().rev().take(limit).collect()
}

/// Anzahl Warnungen und Fehler mit dem heutigen Datum.
pub fn problems_today(lines: &[String], today: NaiveDate) -> usize {
    lines
        .iter()
        .filter_map(|l| parse(l))
        .filter(|p| p.date == today && matches!(p.level, "WARN" | "ERROR"))
        .count()
}

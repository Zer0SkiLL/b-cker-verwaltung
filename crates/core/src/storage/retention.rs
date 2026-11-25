//! Aufbewahrung der Sicherungen: alle der letzten Tage, danach eine pro Tag, Woche und Monat.
//! Gezählt werden Tage/Wochen/Monate mit Sicherungen, nicht die Uhrzeit (robust gegen Pausen
//! und falsch gehende Uhren). Die neueste Sicherung bleibt immer. Einstellbar pro Datei.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use chrono::Datelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::backup::{list, BackupFile};
use crate::error::{CoreError, Result};
use crate::repo::settings;

pub const KEY_KEEP_ALL_DAYS: &str = "backup.keep_all_days";
pub const KEY_DAILY_DAYS: &str = "backup.daily_days";
pub const KEY_WEEKLY_WEEKS: &str = "backup.weekly_weeks";
pub const KEY_MONTHLY_MONTHS: &str = "backup.monthly_months";
pub const KEY_INTERVAL_MINUTES: &str = "backup.interval_minutes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    /// Alle Sicherungen behalten, die jünger sind als so viele Tage.
    pub keep_all_days: u32,
    /// Danach die neueste pro Kalendertag, bis zu diesem Alter in Tagen.
    pub daily_days: u32,
    /// Danach die neueste pro Kalenderwoche, bis zu diesem Alter in Wochen.
    pub weekly_weeks: u32,
    /// Danach die neueste pro Kalendermonat, bis zu diesem Alter in Monaten (0 = keine).
    pub monthly_months: u32,
    /// Sicherung während der Arbeit alle so viele Minuten, falls geändert (0 = aus).
    pub interval_minutes: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy {
            keep_all_days: 2,
            daily_days: 14,
            weekly_weeks: 8,
            monthly_months: 12,
            interval_minutes: 60,
        }
    }
}

impl RetentionPolicy {
    /// Prüft die Grenzen; liefert eine deutsche Meldung.
    pub fn validate(&self) -> std::result::Result<(), String> {
        let p = self;
        if !(1..=30).contains(&p.keep_all_days) {
            return Err("„Alle Sicherungen behalten“: 1 bis 30 Tage.".into());
        }
        if !(p.keep_all_days..=90).contains(&p.daily_days) {
            return Err(format!(
                "„Eine pro Tag“: {} bis 90 Tage (mindestens so lang wie „Alle Sicherungen behalten“).",
                p.keep_all_days
            ));
        }
        if !(1..=104).contains(&p.weekly_weeks) {
            return Err("„Eine pro Woche“: 1 bis 104 Wochen.".into());
        }
        if p.monthly_months > 120 {
            return Err("„Eine pro Monat“: 0 bis 120 Monate.".into());
        }
        if p.interval_minutes != 0 && !(15..=480).contains(&p.interval_minutes) {
            return Err("„Während der Arbeit sichern“: 0 (aus) oder 15 bis 480 Minuten.".into());
        }
        Ok(())
    }
}

/// Liest die Einstellungen der Datei; fehlende Tabelle oder Werte ergeben den Standard.
pub fn load_policy(conn: &Connection) -> RetentionPolicy {
    let d = RetentionPolicy::default();
    let get = |key: &str, default: u32| {
        settings::get(conn, key)
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    };
    let policy = RetentionPolicy {
        keep_all_days: get(KEY_KEEP_ALL_DAYS, d.keep_all_days),
        daily_days: get(KEY_DAILY_DAYS, d.daily_days),
        weekly_weeks: get(KEY_WEEKLY_WEEKS, d.weekly_weeks),
        monthly_months: get(KEY_MONTHLY_MONTHS, d.monthly_months),
        interval_minutes: get(KEY_INTERVAL_MINUTES, d.interval_minutes),
    };
    if policy.validate().is_ok() {
        policy
    } else {
        d
    }
}

/// Speichert gültige Einstellungen.
pub fn save_policy(conn: &Connection, policy: &RetentionPolicy) -> Result<()> {
    policy.validate().map_err(CoreError::Validation)?;
    for (key, value) in [
        (KEY_KEEP_ALL_DAYS, policy.keep_all_days),
        (KEY_DAILY_DAYS, policy.daily_days),
        (KEY_WEEKLY_WEEKS, policy.weekly_weeks),
        (KEY_MONTHLY_MONTHS, policy.monthly_months),
        (KEY_INTERVAL_MINUTES, policy.interval_minutes),
    ] {
        settings::set(conn, key, &value.to_string())?;
    }
    Ok(())
}

/// Rang jedes Schlüssels (Tag, Woche, Monat) unter den vorkommenden, neuester zuerst.
fn ranks<K: Eq + Hash>(
    sorted: &[&BackupFile],
    key: impl Fn(&BackupFile) -> K,
) -> HashMap<K, usize> {
    let mut out = HashMap::new();
    for f in sorted {
        let next = out.len();
        out.entry(key(f)).or_insert(next);
    }
    out
}

fn week_of(f: &BackupFile) -> (i32, u32) {
    let w = f.created.iso_week();
    (w.year(), w.week())
}

fn month_of(f: &BackupFile) -> (i32, u32) {
    (f.created.year(), f.created.month())
}

/// Welche Sicherungen gelöscht werden. Gezählt werden nur Tage, Wochen und Monate, an denen
/// gesichert wurde – nicht die Zeit seit heute. Eine Pause oder eine falsch gehende Uhr kostet so
/// keine Sicherungen. `keep` (die eben erstellte) und die neueste bleiben immer.
/// `files` beliebig sortiert; rein, ohne Dateizugriff.
pub fn to_delete(files: &[BackupFile], keep: &Path, policy: &RetentionPolicy) -> Vec<PathBuf> {
    let mut sorted: Vec<&BackupFile> = files.iter().collect();
    // Neueste zuerst; bei gleichem Zeitstempel ist die Datei mit Suffix (-2, -3 … -10) die neuere:
    // längerer Name zuerst, bei gleicher Länge der grössere.
    let key = |f: &&BackupFile| (f.created, f.path.as_os_str().len(), f.path.clone());
    sorted.sort_by_key(|f| std::cmp::Reverse(key(f)));
    let day_rank = ranks(&sorted, |f| f.created.date());
    let week_rank = ranks(&sorted, week_of);
    let month_rank = ranks(&sorted, month_of);
    let mut days = HashSet::new();
    let mut weeks = HashSet::new();
    let mut months = HashSet::new();
    let mut delete = Vec::new();
    for (i, f) in sorted.iter().enumerate() {
        let day = day_rank[&f.created.date()];
        // Die neueste bleibt immer, belegt aber trotzdem ihren Tag/ihre Woche/ihren Monat.
        let in_tier = if day < policy.keep_all_days as usize {
            true
        } else if day < policy.daily_days as usize {
            days.insert(f.created.date())
        } else if week_rank[&week_of(f)] < policy.weekly_weeks as usize {
            weeks.insert(week_of(f))
        } else if month_rank[&month_of(f)] < policy.monthly_months as usize {
            months.insert(month_of(f))
        } else {
            false
        };
        if !(in_tier || i == 0 || f.path == keep) {
            delete.push(f.path.clone());
        }
    }
    delete
}

/// Dünnt die Sicherungen eines Ordners aus (nur die eigenen, erkennbar am Dateistamm).
/// `keep` ist die eben erstellte Sicherung.
pub fn apply_retention(
    dir: &Path,
    stem: &str,
    keep: &Path,
    policy: &RetentionPolicy,
) -> Result<()> {
    for path in to_delete(&list(dir, stem)?, keep, policy) {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate, NaiveDateTime};

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }

    fn file(t: NaiveDateTime) -> BackupFile {
        BackupFile {
            path: PathBuf::from(format!("d-{}.sqlite", t.format("%Y%m%d-%H%M%S"))),
            created: t,
        }
    }

    fn kept(files: &[BackupFile], p: &RetentionPolicy) -> Vec<NaiveDateTime> {
        kept_protecting(files, &files_newest(files).path, p)
    }

    fn kept_protecting(
        files: &[BackupFile],
        keep: &Path,
        p: &RetentionPolicy,
    ) -> Vec<NaiveDateTime> {
        let gone = to_delete(files, keep, p);
        let mut out: Vec<_> = files
            .iter()
            .filter(|f| !gone.contains(&f.path))
            .map(|f| f.created)
            .collect();
        out.sort();
        out
    }

    fn files_newest(files: &[BackupFile]) -> &BackupFile {
        files.iter().max_by_key(|f| f.created).unwrap()
    }

    // Mittwoch, 25.11.2026, 12:00
    fn now() -> NaiveDateTime {
        at(2026, 11, 25, 12, 0)
    }

    fn only_today_all() -> RetentionPolicy {
        RetentionPolicy {
            keep_all_days: 1,
            daily_days: 1,
            ..Default::default()
        }
    }

    #[test]
    fn keeps_all_of_the_newest_days_with_backups() {
        let files: Vec<_> = (0..10)
            .map(|h| file(now() - Duration::hours(h * 4)))
            .collect();
        assert_eq!(kept(&files, &RetentionPolicy::default()).len(), 10);
    }

    #[test]
    fn keeps_newest_per_day_after_keep_all_days() {
        let mut files: Vec<_> = [9, 13, 17]
            .iter()
            .map(|h| file(at(2026, 11, 20, *h, 0)))
            .collect();
        files.push(file(at(2026, 11, 24, 9, 0)));
        files.push(file(now()));
        assert_eq!(
            kept(&files, &RetentionPolicy::default()),
            vec![at(2026, 11, 20, 17, 0), at(2026, 11, 24, 9, 0), now()]
        );
    }

    #[test]
    fn keep_all_counts_calendar_days_with_backups() {
        // Heute und der letzte Tag davor mit Sicherungen behalten alles, auch nach einer Pause.
        let files = vec![
            file(at(2026, 11, 2, 9, 0)),
            file(at(2026, 11, 2, 15, 0)),
            file(at(2026, 11, 1, 9, 0)),
            file(at(2026, 11, 1, 15, 0)),
            file(now()),
        ];
        assert_eq!(
            kept(&files, &RetentionPolicy::default()),
            vec![
                at(2026, 11, 1, 15, 0),
                at(2026, 11, 2, 9, 0),
                at(2026, 11, 2, 15, 0),
                now()
            ]
        );
    }

    #[test]
    fn keeps_newest_per_iso_week_across_sunday_monday() {
        // So 01.11.2026 23:59 (Woche 44) und Mo 02.11.2026 00:00 (Woche 45): beide Wochen bleiben.
        let files = vec![
            file(at(2026, 11, 1, 23, 59)),
            file(at(2026, 10, 31, 10, 0)),
            file(at(2026, 11, 2, 0, 0)),
            file(at(2026, 11, 3, 8, 0)),
            file(now()),
        ];
        assert_eq!(
            kept(&files, &only_today_all()),
            vec![at(2026, 11, 1, 23, 59), at(2026, 11, 3, 8, 0), now()]
        );
    }

    #[test]
    fn keeps_newest_per_month_for_the_newest_months_with_backups() {
        let p = RetentionPolicy {
            weekly_weeks: 1,
            monthly_months: 2,
            ..only_today_all()
        };
        let files = vec![
            file(at(2026, 5, 3, 9, 0)),
            file(at(2026, 5, 20, 9, 0)),
            file(at(2025, 10, 10, 9, 0)), // dritter Monat mit Sicherungen → weg
            file(now()),
        ];
        assert_eq!(kept(&files, &p), vec![at(2026, 5, 20, 9, 0), now()]);
    }

    #[test]
    fn monthly_zero_deletes_after_weekly_tier() {
        let p = RetentionPolicy {
            weekly_weeks: 1,
            monthly_months: 0,
            ..only_today_all()
        };
        let files = vec![file(at(2026, 8, 1, 9, 0)), file(now())];
        assert_eq!(kept(&files, &p), vec![now()]);
    }

    #[test]
    fn clock_jump_forward_keeps_history() {
        // Ein Jahr lang je eine Sicherung pro Monat, dann springt die Uhr zwei Jahre vor.
        let mut files: Vec<_> = (1..=11).map(|m| file(at(2026, m, 15, 12, 0))).collect();
        let jumped = file(at(2028, 11, 25, 12, 0));
        files.push(jumped.clone());
        assert!(to_delete(&files, &jumped.path, &RetentionPolicy::default()).is_empty());
    }

    #[test]
    fn long_pause_keeps_daily_backups() {
        // 20 Tage mit Sicherungen, dann drei Monate Pause: die Tagesstufe bleibt erhalten.
        let mut files: Vec<_> = (1..=14).map(|d| file(at(2026, 8, d, 12, 0))).collect();
        files.push(file(now()));
        let k = kept(&files, &RetentionPolicy::default());
        assert_eq!(k.len(), 15);
    }

    #[test]
    fn clock_jump_backward_protects_the_new_backup() {
        let wrong = file(at(2000, 1, 1, 0, 0));
        let files = vec![file(now()), file(now() - Duration::hours(1)), wrong.clone()];
        let k = kept_protecting(&files, &wrong.path, &RetentionPolicy::default());
        assert!(k.contains(&wrong.created));
    }

    #[test]
    fn newest_always_kept_even_if_very_old() {
        let only = file(at(2020, 1, 1, 0, 0));
        assert!(to_delete(&[only.clone()], &only.path, &RetentionPolicy::default()).is_empty());
    }

    #[test]
    fn same_second_suffix_keeps_one_per_day() {
        let t = at(2026, 11, 18, 9, 0);
        let a = BackupFile {
            path: PathBuf::from("d-20261118-090000.sqlite"),
            created: t,
        };
        let b = BackupFile {
            path: PathBuf::from("d-20261118-090000-2.sqlite"),
            created: t,
        };
        let newest = file(now());
        let gone = to_delete(
            &[a.clone(), b, newest.clone()],
            &newest.path,
            &only_today_all(),
        );
        assert_eq!(gone, vec![a.path]);
    }

    #[test]
    fn validate_limits() {
        let ok = RetentionPolicy::default();
        assert!(ok.validate().is_ok());
        assert!(RetentionPolicy {
            daily_days: 1,
            ..ok
        }
        .validate()
        .is_err());
        assert!(RetentionPolicy {
            interval_minutes: 10,
            ..ok
        }
        .validate()
        .is_err());
        assert!(RetentionPolicy {
            interval_minutes: 0,
            ..ok
        }
        .validate()
        .is_ok());
        assert!(RetentionPolicy {
            keep_all_days: 0,
            ..ok
        }
        .validate()
        .is_err());
        assert!(RetentionPolicy {
            monthly_months: 121,
            ..ok
        }
        .validate()
        .is_err());
    }

    #[test]
    fn load_policy_without_settings_table_is_default() {
        let conn = Connection::open_in_memory().unwrap();
        assert_eq!(load_policy(&conn), RetentionPolicy::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let conn = crate::db::open_in_memory().unwrap();
        let p = RetentionPolicy {
            keep_all_days: 3,
            daily_days: 20,
            weekly_weeks: 10,
            monthly_months: 24,
            interval_minutes: 30,
        };
        save_policy(&conn, &p).unwrap();
        assert_eq!(load_policy(&conn), p);
        let bad = RetentionPolicy {
            interval_minutes: 5,
            ..p
        };
        assert!(save_policy(&conn, &bad).is_err());
    }
}

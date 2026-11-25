//! Aufbewahrung der Sicherungen nach Alter: alle der letzten Tage, danach eine pro Tag, Woche
//! und Monat. Die neueste Sicherung bleibt immer. Einstellbar pro Datei (Tabelle `settings`).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Duration, NaiveDateTime};
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

fn months_between(now: NaiveDateTime, then: NaiveDateTime) -> i64 {
    (now.year() as i64 * 12 + now.month() as i64) - (then.year() as i64 * 12 + then.month() as i64)
}

/// Welche Sicherungen gelöscht werden. `files` beliebig sortiert; rein, ohne Dateizugriff.
pub fn to_delete(
    files: &[BackupFile],
    now: NaiveDateTime,
    policy: &RetentionPolicy,
) -> Vec<PathBuf> {
    let mut sorted: Vec<&BackupFile> = files.iter().collect();
    // Neueste zuerst; bei gleichem Zeitstempel ist die Datei mit Suffix (-2, -3 … -10) die neuere:
    // längerer Name zuerst, bei gleicher Länge der grössere.
    let key = |f: &&BackupFile| (f.created, f.path.as_os_str().len(), f.path.clone());
    sorted.sort_by_key(|f| std::cmp::Reverse(key(f)));
    let mut days = HashSet::new();
    let mut weeks = HashSet::new();
    let mut months = HashSet::new();
    let mut delete = Vec::new();
    for (i, f) in sorted.iter().enumerate() {
        let age = now - f.created;
        // Die neueste bleibt immer, belegt aber trotzdem ihren Tag/ihre Woche/ihren Monat.
        let in_tier = if age < Duration::days(policy.keep_all_days as i64) {
            true
        } else if age < Duration::days(policy.daily_days as i64) {
            days.insert(f.created.date())
        } else if age < Duration::weeks(policy.weekly_weeks as i64) {
            let w = f.created.iso_week();
            weeks.insert((w.year(), w.week()))
        } else if months_between(now, f.created) < policy.monthly_months as i64 {
            months.insert((f.created.year(), f.created.month()))
        } else {
            false
        };
        let keep = in_tier || i == 0;
        if !keep {
            delete.push(f.path.clone());
        }
    }
    delete
}

/// Dünnt die Sicherungen eines Ordners aus (nur die eigenen, erkennbar am Dateistamm).
pub fn apply_retention(
    dir: &Path,
    stem: &str,
    now: NaiveDateTime,
    policy: &RetentionPolicy,
) -> Result<()> {
    for path in to_delete(&list(dir, stem)?, now, policy) {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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

    fn kept(files: &[BackupFile], now: NaiveDateTime, p: &RetentionPolicy) -> Vec<NaiveDateTime> {
        let gone = to_delete(files, now, p);
        let mut out: Vec<_> = files
            .iter()
            .filter(|f| !gone.contains(&f.path))
            .map(|f| f.created)
            .collect();
        out.sort();
        out
    }

    // Mittwoch, 25.11.2026, 12:00
    fn now() -> NaiveDateTime {
        at(2026, 11, 25, 12, 0)
    }

    #[test]
    fn keeps_all_younger_than_keep_all_days() {
        let files: Vec<_> = (0..10)
            .map(|h| file(now() - Duration::hours(h * 4)))
            .collect();
        assert_eq!(kept(&files, now(), &RetentionPolicy::default()).len(), 10);
    }

    #[test]
    fn keeps_newest_per_day_in_daily_tier() {
        let day = at(2026, 11, 20, 0, 0);
        let files: Vec<_> = [9, 13, 17]
            .iter()
            .map(|h| file(day + Duration::hours(*h)))
            .collect();
        assert_eq!(
            kept(&files, now(), &RetentionPolicy::default()),
            vec![at(2026, 11, 20, 17, 0)]
        );
    }

    #[test]
    fn exactly_keep_all_days_old_is_daily_tier() {
        let a = file(now() - Duration::days(2));
        let b = file(now() - Duration::days(2) - Duration::hours(1));
        let newest = file(now());
        let k = kept(&[a.clone(), b, newest], now(), &RetentionPolicy::default());
        assert_eq!(k, vec![a.created, now()]);
    }

    #[test]
    fn keeps_newest_per_iso_week_across_sunday_monday() {
        // So 01.11.2026 23:59 (Woche 44) und Mo 02.11.2026 00:00 (Woche 45): beide bleiben.
        let sun = file(at(2026, 11, 1, 23, 59));
        let sat = file(at(2026, 10, 31, 10, 0));
        let mon = file(at(2026, 11, 2, 0, 0));
        let tue = file(at(2026, 11, 3, 8, 0));
        let p = RetentionPolicy {
            daily_days: 14,
            ..Default::default()
        };
        // Alter 22–25 Tage → Wochenstufe.
        let k = kept(&[sun.clone(), sat, mon, tue.clone()], now(), &p);
        assert_eq!(k, vec![sun.created, tue.created]);
    }

    #[test]
    fn keeps_newest_per_month_then_deletes_older() {
        let p = RetentionPolicy::default();
        let files = vec![
            file(at(2026, 5, 3, 9, 0)),
            file(at(2026, 5, 20, 9, 0)),
            file(at(2025, 10, 10, 9, 0)), // 13 Monate alt → weg
            file(now()),
        ];
        assert_eq!(kept(&files, now(), &p), vec![at(2026, 5, 20, 9, 0), now()]);
    }

    #[test]
    fn newest_always_kept_even_if_very_old() {
        let only = file(at(2020, 1, 1, 0, 0));
        assert!(to_delete(&[only], now(), &RetentionPolicy::default()).is_empty());
    }

    #[test]
    fn future_backup_is_kept() {
        let files = vec![file(now() + Duration::days(3)), file(now())];
        assert!(to_delete(&files, now(), &RetentionPolicy::default()).is_empty());
    }

    #[test]
    fn monthly_zero_deletes_after_weekly_tier() {
        let p = RetentionPolicy {
            monthly_months: 0,
            ..Default::default()
        };
        let files = vec![file(at(2026, 8, 1, 9, 0)), file(now())];
        assert_eq!(kept(&files, now(), &p), vec![now()]);
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
        let gone = to_delete(
            &[a.clone(), b, file(now())],
            now(),
            &RetentionPolicy::default(),
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

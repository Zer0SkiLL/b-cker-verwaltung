//! Deutet einzelne Excel-Zellen tolerant: Texte, Ja/Nein-Werte, Daten und Zahlen.

use chrono::NaiveDate;

use super::sheet::Cell;
use crate::text::format_date;

const YES: &[&str] = &["x", "ja", "j", "1", "true", "wahr"];
const NO: &[&str] = &["nein", "n", "0", "false", "falsch"];
/// Ganze Zahlen in diesem Bereich gelten als Jahreszahl, nicht als Excel-Datumswert.
const YEAR_RANGE: std::ops::RangeInclusive<i64> = 1800..=2999;

#[derive(Debug, Clone, PartialEq)]
pub enum FlagResult {
    Yes,
    No,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateResult {
    None,
    Date(NaiveDate),
    /// Nur ein Jahr angegeben; 01.01. angenommen.
    YearOnly(NaiveDate),
    Invalid(String),
}

pub fn as_text(cell: &Cell) -> Option<String> {
    let text = match cell {
        Cell::Empty => return None,
        Cell::Text(t) => t.trim().to_string(),
        Cell::Number(n) if n.fract() == 0.0 && n.abs() < 1e15 => format!("{}", *n as i64),
        Cell::Number(n) => n.to_string(),
        Cell::Bool(b) => if *b { "ja" } else { "nein" }.to_string(),
        Cell::Date(d) => format_date(*d),
    };
    (!text.is_empty()).then_some(text)
}

pub fn as_flag(cell: &Cell) -> FlagResult {
    match cell {
        Cell::Bool(b) => {
            if *b {
                FlagResult::Yes
            } else {
                FlagResult::No
            }
        }
        Cell::Number(n) => {
            if *n != 0.0 {
                FlagResult::Yes
            } else {
                FlagResult::No
            }
        }
        _ => match as_text(cell) {
            None => FlagResult::No,
            Some(t) => {
                let lower = t.to_lowercase();
                if YES.contains(&lower.as_str()) {
                    FlagResult::Yes
                } else if NO.contains(&lower.as_str()) {
                    FlagResult::No
                } else {
                    FlagResult::Unknown(t)
                }
            }
        },
    }
}

pub fn as_date(cell: &Cell) -> DateResult {
    match cell {
        Cell::Date(d) => DateResult::Date(*d),
        Cell::Number(n) => number_date(*n),
        _ => match as_text(cell) {
            None => DateResult::None,
            Some(t) => text_date(&t),
        },
    }
}

pub fn as_int(cell: &Cell) -> Result<Option<i64>, String> {
    match cell {
        Cell::Number(n) if n.fract() == 0.0 => Ok(Some(*n as i64)),
        _ => match as_text(cell) {
            None => Ok(None),
            Some(t) => t.parse::<i64>().map(Some).map_err(|_| t),
        },
    }
}

fn number_date(n: f64) -> DateResult {
    if n.fract() == 0.0 && YEAR_RANGE.contains(&(n as i64)) {
        return year_only(n as i32).unwrap_or(DateResult::Invalid(n.to_string()));
    }
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30).expect("gültig");
    epoch
        .checked_add_days(chrono::Days::new(n.trunc().max(0.0) as u64))
        .filter(|_| n >= 1.0)
        .map(DateResult::Date)
        .unwrap_or(DateResult::Invalid(n.to_string()))
}

fn text_date(t: &str) -> DateResult {
    // chrono akzeptiert bei %Y auch zweistellige Jahre (19 → Jahr 0019); nur vierstellige zulassen.
    let four_digit_year = t.rsplit('.').next().is_some_and(|y| y.len() == 4);
    if let (true, Ok(d)) = (four_digit_year, NaiveDate::parse_from_str(t, "%d.%m.%Y")) {
        return DateResult::Date(d);
    }
    if t.len() == 4 {
        if let Some(r) = t.parse::<i32>().ok().and_then(year_only) {
            return r;
        }
    }
    DateResult::Invalid(t.to_string())
}

fn year_only(year: i32) -> Option<DateResult> {
    NaiveDate::from_ymd_opt(year, 1, 1).map(DateResult::YearOnly)
}

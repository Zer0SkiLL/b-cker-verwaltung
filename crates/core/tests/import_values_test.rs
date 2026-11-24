use chrono::NaiveDate;
use verwaltung_core::import::sheet::Cell;
use verwaltung_core::import::values::{as_date, as_flag, as_int, as_text, DateResult, FlagResult};

fn t(s: &str) -> Cell {
    Cell::Text(s.into())
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

#[test]
fn text_trims_and_renders_whole_floats() {
    assert_eq!(as_text(&t("  Bern ")), Some("Bern".into()));
    assert_eq!(as_text(&t("   ")), None);
    assert_eq!(as_text(&Cell::Empty), None);
    assert_eq!(as_text(&Cell::Number(8000.0)), Some("8000".into()));
    assert_eq!(as_text(&Cell::Number(1.5)), Some("1.5".into()));
    assert_eq!(as_text(&Cell::Date(d(2020, 1, 2))), Some("02.01.2020".into()));
}

#[test]
fn flag_variants() {
    for yes in [t("x"), t("X"), t(" ja "), t("J"), t("1"), t("true"), t("Wahr"), Cell::Number(2.0), Cell::Bool(true)] {
        assert_eq!(as_flag(&yes), FlagResult::Yes, "{yes:?}");
    }
    for no in [Cell::Empty, t(""), t("nein"), t("N"), t("0"), t("false"), t("falsch"), Cell::Number(0.0), Cell::Bool(false)] {
        assert_eq!(as_flag(&no), FlagResult::No, "{no:?}");
    }
    assert_eq!(as_flag(&t("vielleicht")), FlagResult::Unknown("vielleicht".into()));
}

#[test]
fn date_variants() {
    assert_eq!(as_date(&Cell::Empty), DateResult::None);
    assert_eq!(as_date(&t("5.3.2019")), DateResult::Date(d(2019, 3, 5)));
    assert_eq!(as_date(&t("05.03.2019")), DateResult::Date(d(2019, 3, 5)));
    assert_eq!(as_date(&t("2019")), DateResult::YearOnly(d(2019, 1, 1)));
    assert_eq!(as_date(&Cell::Number(2019.0)), DateResult::YearOnly(d(2019, 1, 1)));
    assert_eq!(as_date(&Cell::Number(43529.0)), DateResult::Date(d(2019, 3, 5)));
    assert_eq!(as_date(&Cell::Date(d(2019, 3, 5))), DateResult::Date(d(2019, 3, 5)));
    assert_eq!(as_date(&t("31.02.2019")), DateResult::Invalid("31.02.2019".into()));
}

#[test]
fn int_rejects_text() {
    assert_eq!(as_int(&Cell::Number(3.0)), Ok(Some(3)));
    assert_eq!(as_int(&t(" 4 ")), Ok(Some(4)));
    assert_eq!(as_int(&Cell::Empty), Ok(None));
    assert!(as_int(&t("viele")).is_err());
    assert!(as_int(&Cell::Number(2.5)).is_err());
}

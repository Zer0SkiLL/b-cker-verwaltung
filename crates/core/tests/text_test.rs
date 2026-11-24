use chrono::NaiveDate;
use verwaltung_core::text::{format_date, normalize_search};

#[test]
fn normalize_search_lowercases_umlauts() {
    assert_eq!(
        normalize_search(&[Some("Bäckerei MÜLLER"), None, Some("  "), Some("Zürich")]),
        "bäckerei müller zürich"
    );
}

#[test]
fn format_date_swiss() {
    assert_eq!(
        format_date(NaiveDate::from_ymd_opt(2024, 3, 5).unwrap()),
        "05.03.2024"
    );
}

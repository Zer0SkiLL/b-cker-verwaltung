use verwaltung_core::contact::{email_list, mailto_bcc, MAILTO_MAX_LEN};
use verwaltung_core::query::filter::{ContactRow, EntryKind};

fn row(id: i64, email: Option<&str>) -> ContactRow {
    ContactRow {
        kind: EntryKind::Person,
        id,
        display_name: format!("P{id}"),
        organisation: None,
        street: None,
        zip: None,
        city: None,
        po_box: None,
        email: email.map(String::from),
        phone: None,
        mobile: None,
    }
}

#[test]
fn dedupes_case_insensitive_and_trims() {
    let list = email_list(&[row(1, Some(" A@x.ch")), row(2, Some("a@X.ch")), row(3, Some("b@x.ch"))]);
    assert_eq!(list.addresses, vec!["A@x.ch", "b@x.ch"]);
}

#[test]
fn invalid_and_empty_go_to_missing() {
    let list = email_list(&[row(1, None), row(2, Some("  ")), row(3, Some("kein-at")), row(4, Some("ok@x.ch"))]);
    let missing: Vec<i64> = list.missing.iter().map(|r| r.id).collect();
    assert_eq!(missing, vec![1, 2, 3]);
    assert_eq!(list.addresses, vec!["ok@x.ch"]);
}

#[test]
fn joined_uses_semicolon_space() {
    let list = email_list(&[row(1, Some("a@x.ch")), row(2, Some("b@x.ch"))]);
    assert_eq!(list.joined, "a@x.ch; b@x.ch");
}

#[test]
fn mailto_none_when_too_long() {
    let many: Vec<String> = (0..MAILTO_MAX_LEN).map(|i| format!("person{i}@example.ch")).collect();
    assert_eq!(mailto_bcc(&many), None);
}

#[test]
fn mailto_encodes_plus_and_umlaut() {
    let link = mailto_bcc(&["a+b@x.ch".to_string(), "jürg@x.ch".to_string()]).unwrap();
    assert_eq!(link, "mailto:?bcc=a%2Bb%40x.ch,j%C3%BCrg%40x.ch");
}

mod common;

use common::{fresh_session, org_input, write_xlsx, LIST_HEADERS};
use verwaltung_app::commands::import::{
    export_report, field_key, import_apply, import_preview, parse_field, ALL_FIELDS,
};
use verwaltung_core::repo::organisations;

fn row<'a>(values: &[(usize, &'a str)]) -> Vec<&'a str> {
    let mut row = vec![""; LIST_HEADERS.len()];
    for (i, v) in values {
        row[*i] = v;
    }
    row
}

fn sample(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("liste.xlsx");
    let rows = vec![
        row(&[(3, "Beck AG"), (8, "Bern")]),
        row(&[
            (3, "Muster"),
            (4, "Hans"),
            (5, "Bäckerei Muster"),
            (8, "Thun"),
        ]),
        row(&[(3, "Brot GmbH"), (17, "05.03.19")]),
    ];
    write_xlsx(&path, &LIST_HEADERS, &rows);
    path
}

#[test]
fn preview_uses_default_mapping_for_known_headers() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let preview = import_preview(&mut s, &sample(dir.path()), None).unwrap();
    assert_eq!(preview.headers.len(), 23);
    assert_eq!(preview.mapping[3], "name");
    assert_eq!(preview.mapping[13], "fr_baeck");
    assert_eq!(preview.rows, 3);
    assert_eq!(preview.persons, 1);
    assert!(preview.database_empty);
    assert!(
        preview.report.iter().any(|r| r.line == 4),
        "{:?}",
        preview.report
    );
    assert_eq!(preview.sample.len(), 3);
    assert_eq!(preview.fields.len(), ALL_FIELDS.len());
}

#[test]
fn preview_reports_not_empty_database() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| organisations::create(c, &org_input("Schon da")))
        .unwrap();
    let file = sample(dir.path());
    let preview = import_preview(&mut s, &file, None).unwrap();
    assert!(!preview.database_empty);
    let err = import_apply(&mut s, &file, preview.mapping).unwrap_err();
    assert!(
        err.0.starts_with("Die Datenbank ist nicht leer"),
        "{}",
        err.0
    );
}

#[test]
fn wrong_mapping_length_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let err = import_preview(&mut s, &sample(dir.path()), Some(vec!["name".into()])).unwrap_err();
    assert_eq!(err.0, "Die Zuordnung passt nicht zur Datei.");
}

#[test]
fn unknown_field_key_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let mut mapping = vec!["ignore".to_string(); 23];
    mapping[0] = "xyz".into();
    let err = import_preview(&mut s, &sample(dir.path()), Some(mapping)).unwrap_err();
    assert_eq!(err.0, "Unbekanntes Feld: xyz");
}

#[test]
fn apply_imports_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let file = sample(dir.path());
    let preview = import_preview(&mut s, &file, None).unwrap();
    let summary = import_apply(&mut s, &file, preview.mapping).unwrap();
    assert_eq!(summary.organisations, 3);
    assert_eq!(summary.persons, 1);
    assert_eq!(s.read(organisations::list).unwrap().len(), 3);
}

#[test]
fn field_keys_roundtrip() {
    for field in ALL_FIELDS {
        assert_eq!(parse_field(field_key(field)), Some(field));
    }
    assert_eq!(
        field_key(verwaltung_core::import::mapping::Field::LegacyId),
        "legacy_id"
    );
}

#[test]
fn report_export_writes_csv() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let preview = import_preview(&mut s, &sample(dir.path()), None).unwrap();
    let target = dir.path().join("bericht.csv");
    export_report(&preview.report, &target).unwrap();
    let text = String::from_utf8_lossy(&std::fs::read(&target).unwrap()).to_string();
    assert!(text.contains("Zeile;Spalte;Meldung"), "{text}");
}

#[test]
fn import_into_new_file_reuses_default_categories() {
    use verwaltung_core::model::LookupKind;
    use verwaltung_core::repo::{labels, lookups};
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let path = dir.path().join("kategorien.xlsx");
    let rows = vec![
        row(&[(3, "Beck AG"), (9, "Mitglied mit Geschäft")]),
        row(&[(3, "Brot GmbH"), (9, "Mitglied ohne SBC")]),
    ];
    write_xlsx(&path, &LIST_HEADERS, &rows);
    let preview = import_preview(&mut s, &path, None).unwrap();
    import_apply(&mut s, &path, preview.mapping).unwrap();
    let categories = s
        .read(|c| lookups::list(c, LookupKind::MembershipCategory))
        .unwrap();
    assert_eq!(categories.len(), 4, "{categories:?}");
    let provisional = s
        .read(|c| {
            let groups = labels::list_groups(c)?;
            let group = groups.iter().find(|g| g.name == "Aufnahme").unwrap();
            labels::list_labels(c, group.id)
        })
        .unwrap();
    assert_eq!(provisional.len(), 1);
}

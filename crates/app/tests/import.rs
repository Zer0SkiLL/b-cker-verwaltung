mod common;

use chrono::Local;
use common::{fresh_session, org_input, write_xlsx, LIST_HEADERS};
use verwaltung_app::commands::import::{
    export_preview, export_report, field_key, import_apply, import_preview, import_undo,
    parse_field, ALL_FIELDS,
};
use verwaltung_core::model::LookupKind;
use verwaltung_core::repo::{labels, lookups, organisations};

/// Spalte nach Kopfzeile.
fn col(name: &str) -> usize {
    LIST_HEADERS.iter().position(|h| *h == name).unwrap()
}

fn row<'a>(values: &[(&str, &'a str)]) -> Vec<&'a str> {
    let mut row = vec![""; LIST_HEADERS.len()];
    for (h, v) in values {
        row[col(h)] = v;
    }
    row
}

/// Zwei Zeilen desselben Betriebs, ein Einzelmitglied, eine leere Zeile (nur Ort).
fn sample(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("liste.xlsx");
    let firm = |name: &'static str, kat: &'static str| {
        vec![
            ("Firma", "Beck AG"),
            ("Adresse", "Weg 1"),
            ("PLZ", "9000"),
            ("Name", name),
            ("Kategorie", kat),
        ]
    };
    let rows = vec![
        row(&firm("Keller", "kein Mitglied")),
        row(&firm("Huber", "kein Mitglied")),
        row(&[
            ("Name", "Einzeln"),
            ("Kategorie", "Ehrenmitglied"),
            ("Mandat", "Präsident"),
        ]),
        row(&[("Ort", "Bern")]),
    ];
    write_xlsx(&path, &LIST_HEADERS, &rows);
    path
}

#[test]
fn preview_groups_persons_under_org_and_counts_add_up() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let p = import_preview(&mut s, &sample(dir.path()), None).unwrap();
    assert_eq!(p.headers.len(), LIST_HEADERS.len());
    assert_eq!(p.mapping[col("Firma")], "company");
    assert_eq!(p.mapping[col("Adresszusatz")], "ignore");
    assert_eq!(p.rows, 4);
    assert_eq!(p.organisations.len(), 1);
    assert_eq!(p.organisations[0].lines, vec![2, 3]);
    assert_eq!(p.organisations[0].persons.len(), 2);
    assert_eq!(p.organisations[0].persons[0].function, "Kontaktperson");
    assert_eq!(p.merged, 1);
    assert_eq!(p.persons, 2);
    assert_eq!(p.individuals.len(), 1);
    assert_eq!(p.individuals[0].person.labels, vec!["Mandat: Präsident"]);
    assert_eq!(p.skipped.len(), 1);
    assert_eq!(p.reconcile_error, None);
    // 1 Betrieb (erste Zeile) + 1 zusammengefasst + 1 Einzelmitglied + 1 übersprungen = 4
    assert_eq!(
        p.organisations.len() + p.merged + p.individuals.len() + p.skipped.len(),
        p.rows
    );
    assert!(p.database_empty);
    assert_eq!(p.fields.len(), ALL_FIELDS.len());
}

#[test]
fn preview_lists_new_entries_case_insensitively() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let p = import_preview(&mut s, &sample(dir.path()), None).unwrap();
    // „kein Mitglied“ gibt es als Vorgabe „Kein Mitglied“; „Ehrenmitglied“ ist neu.
    assert_eq!(p.new_entries.categories, vec!["Ehrenmitglied"]);
    assert_eq!(p.new_entries.functions, vec!["Kontaktperson"]);
    assert_eq!(p.new_entries.labels, vec!["Mandat: Präsident"]);
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
    let err = import_apply(&mut s, &file, preview.mapping, Local::now()).unwrap_err();
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
    let err =
        import_preview(&mut s, &sample(dir.path()), Some(vec!["company".into()])).unwrap_err();
    assert_eq!(err.0, "Die Zuordnung passt nicht zur Datei.");
}

#[test]
fn unknown_field_key_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let mut mapping = vec!["ignore".to_string(); LIST_HEADERS.len()];
    mapping[0] = "xyz".into();
    let err = import_preview(&mut s, &sample(dir.path()), Some(mapping)).unwrap_err();
    assert_eq!(err.0, "Unbekanntes Feld: xyz");
}

#[test]
fn apply_creates_backup_then_undo_restores_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let file = sample(dir.path());
    let preview = import_preview(&mut s, &file, None).unwrap();
    let summary = import_apply(&mut s, &file, preview.mapping, Local::now()).unwrap();
    assert_eq!(
        (summary.organisations, summary.persons, summary.individuals),
        (1, 2, 1)
    );
    assert!(std::path::Path::new(&summary.undo_backup).exists());
    assert_eq!(s.read(organisations::list).unwrap().len(), 1);
    import_undo(
        &mut s,
        std::path::Path::new(&summary.undo_backup),
        Local::now(),
    )
    .unwrap();
    assert_eq!(s.read(organisations::list).unwrap().len(), 0);
    // Vorgaben der neuen Datei sind wieder da (Stand vor dem Import).
    let cats = s
        .read(|c| lookups::list(c, LookupKind::MembershipCategory))
        .unwrap();
    assert_eq!(cats.len(), 4);
}

#[test]
fn undo_refuses_foreign_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let foreign = dir.path().join("fremd.sqlite");
    std::fs::write(&foreign, b"x").unwrap();
    let err = import_undo(&mut s, &foreign, Local::now()).unwrap_err();
    assert_eq!(err.0, "Ungültige Sicherung für das Rückgängigmachen.");
}

#[test]
fn import_into_new_file_reuses_default_categories() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let file = sample(dir.path());
    let preview = import_preview(&mut s, &file, None).unwrap();
    import_apply(&mut s, &file, preview.mapping, Local::now()).unwrap();
    let cats = s
        .read(|c| lookups::list(c, LookupKind::MembershipCategory))
        .unwrap();
    // 4 Vorgaben + „Ehrenmitglied“; „kein Mitglied“ landet bei „Kein Mitglied“.
    assert_eq!(cats.len(), 5, "{cats:?}");
    let provisional = s
        .read(|c| {
            let groups = labels::list_groups(c)?;
            let group = groups.iter().find(|g| g.name == "Aufnahme").unwrap();
            labels::list_labels(c, group.id)
        })
        .unwrap();
    assert_eq!(provisional.len(), 1);
}

#[test]
fn export_preview_has_one_row_per_source_row() {
    use calamine::{open_workbook, Reader, Xlsx};
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let file = sample(dir.path());
    let out = dir.path().join("vorschau.xlsx");
    export_preview(&mut s, &file, None, &out).unwrap();
    let mut wb: Xlsx<_> = open_workbook(&out).unwrap();
    let range = wb.worksheet_range_at(0).unwrap().unwrap();
    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|r| r.iter().map(|c| c.to_string()).collect())
        .collect();
    assert_eq!(
        rows[0],
        vec![
            "Zeile",
            "Ergebnis",
            "Betrieb",
            "Person",
            "Kategorie",
            "Labels",
            "Hinweise"
        ]
    );
    assert_eq!(rows.len(), 5);
    let results: Vec<&str> = rows[1..].iter().map(|r| r[1].as_str()).collect();
    assert_eq!(
        results,
        vec![
            "Betrieb",
            "Zusammengefasst",
            "Einzelmitglied",
            "Übersprungen"
        ]
    );
}

#[test]
fn field_keys_roundtrip() {
    for field in ALL_FIELDS {
        assert_eq!(parse_field(field_key(field)), Some(field));
    }
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

/// Prüft eine echte (private) Liste lokal: `MV_MUSTERDATEI=<pfad> cargo test -p verwaltung_app
/// --test import musterdatei -- --ignored --nocapture`. Gibt nur Zahlen und Arten von Hinweisen aus.
#[test]
#[ignore]
fn musterdatei() {
    let Ok(file) = std::env::var("MV_MUSTERDATEI") else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let file = std::path::PathBuf::from(file);
    let p = import_preview(&mut s, &file, None).unwrap();
    let mut kinds: Vec<String> = p
        .report
        .iter()
        .map(|r| r.message.split(':').next().unwrap_or("").to_string())
        .collect();
    kinds.sort();
    kinds.dedup();
    println!(
        "Zeilen {}, Betriebe {}, zusammengefasst {}, Personen {}, Einzelmitglieder {}, übersprungen {}, Abgleich {:?}",
        p.rows, p.organisations.len(), p.merged, p.persons, p.individuals.len(), p.skipped.len(), p.reconcile_error
    );
    println!(
        "Nicht zugeordnete Spalten: {}",
        p.mapping.iter().filter(|m| *m == "ignore").count()
    );
    println!("Arten von Hinweisen: {kinds:?}");
    println!(
        "Neu: Kategorien {}, Kantonalverbände {}, Funktionen {}, Labels {}",
        p.new_entries.categories.len(),
        p.new_entries.cantonal.len(),
        p.new_entries.functions.len(),
        p.new_entries.labels.len()
    );
    let summary = import_apply(&mut s, &file, p.mapping, Local::now()).unwrap();
    println!(
        "Übernommen: {} Betriebe, {} Personen, {} Einzelmitglieder",
        summary.organisations, summary.persons, summary.individuals
    );
}

#[test]
fn preview_shows_notes_of_merged_rows_without_person() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let path = dir.path().join("abweichung.xlsx");
    let base = |kat: &'static str| {
        vec![
            ("Firma", "Beck AG"),
            ("Adresse", "Weg 1"),
            ("PLZ", "9000"),
            ("Kategorie", kat),
        ]
    };
    write_xlsx(
        &path,
        &LIST_HEADERS,
        &[
            row(&base("Mitglied mit Geschäft")),
            row(&base("Kein Mitglied")),
        ],
    );
    let p = import_preview(&mut s, &path, None).unwrap();
    let notes = &p.organisations[0].notes;
    assert!(
        notes
            .iter()
            .any(|n| n.contains("Abweichende Angabe für Kategorie")),
        "{notes:?}"
    );
}

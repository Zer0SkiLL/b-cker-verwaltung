mod common;

use chrono::NaiveDate;
use common::{db_path, fresh_session, org_input, who};
use verwaltung_app::commands::addresses::{
    delete_group, export_addresses, filter_options, list_groups, load_group, save_group,
    search_addresses, ExportFormat,
};
use verwaltung_app::session::{Session, READ_ONLY};
use verwaltung_core::model::AppliesTo;
use verwaltung_core::query::filter::Filter;
use verwaltung_core::repo::{labels, organisations, persons};

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 24).unwrap()
}

fn org_with_mail(s: &mut Session, name: &str, mail: &str) {
    let mut input = org_input(name);
    input.email = Some(mail.into());
    s.write(|c| organisations::create(c, &input)).unwrap();
}

#[test]
fn search_returns_rows_and_email_summary() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    org_with_mail(&mut s, "Beck AG", "info@beck.ch");
    org_with_mail(&mut s, "Brot GmbH", "hallo@brot.ch");
    let person = persons::PersonInput {
        last_name: "Keller".into(),
        ..Default::default()
    };
    s.write(|c| persons::create(c, &person)).unwrap();
    let result = search_addresses(&s, &Filter::default(), today()).unwrap();
    assert_eq!(result.rows.len(), 3);
    assert_eq!(result.email_count, 2);
    assert_eq!(result.missing_email.len(), 1);
    assert!(result.joined_emails.contains("; "));
    assert!(result.mailto.unwrap().starts_with("mailto:?bcc="));
}

#[test]
fn empty_result_has_no_mailto() {
    let dir = tempfile::tempdir().unwrap();
    let s = fresh_session(dir.path());
    let result = search_addresses(&s, &Filter::default(), today()).unwrap();
    assert!(result.rows.is_empty());
    assert_eq!(result.mailto, None);
    assert_eq!(result.joined_emails, "");
}

#[test]
fn too_many_addresses_no_mailto() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    for i in 0..200 {
        org_with_mail(
            &mut s,
            &format!("Bäckerei {i}"),
            &format!("bestellung.{i}@baeckerei-beispiel.ch"),
        );
    }
    let result = search_addresses(&s, &Filter::default(), today()).unwrap();
    assert_eq!(result.email_count, 200);
    assert_eq!(result.mailto, None);
}

#[test]
fn export_csv_and_xlsx_write_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    org_with_mail(&mut s, "Beck AG", "info@beck.ch");
    let csv = dir.path().join("liste.csv");
    let xlsx = dir.path().join("liste.xlsx");
    export_addresses(&s, &Filter::default(), today(), &csv, ExportFormat::Csv).unwrap();
    export_addresses(&s, &Filter::default(), today(), &xlsx, ExportFormat::Xlsx).unwrap();
    let bytes = std::fs::read(&csv).unwrap();
    assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(String::from_utf8_lossy(&bytes[3..]).starts_with("Typ;Name;"));
    assert!(xlsx.exists());
}

#[test]
fn groups_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let filter = Filter {
        text: Some("beck".into()),
        label_ids: vec![3],
        ..Default::default()
    };
    let id = save_group(&mut s, "Prüfungsexperten", &filter).unwrap();
    let groups = list_groups(&s).unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "Prüfungsexperten");
    assert_eq!(load_group(&s, id).unwrap(), filter);
    delete_group(&mut s, id).unwrap();
    assert!(list_groups(&s).unwrap().is_empty());
}

#[test]
fn save_group_is_refused_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let _anna = fresh_session(dir.path());
    let mut ben = Session::default();
    ben.open(&db_path(dir.path()), &who("ben")).unwrap();
    let err = save_group(&mut ben, "X", &Filter::default()).unwrap_err();
    assert_eq!(err.0, READ_ONLY);
}

#[test]
fn filter_options_lists_labels_by_group() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    s.write(|c| {
        let g = labels::create_group(c, "Rollen", AppliesTo::Person)?;
        labels::create_label(c, g.id, "Vorstand")?;
        labels::create_label(c, g.id, "Prüfungsexperte")
    })
    .unwrap();
    let options = filter_options(&s).unwrap();
    let rollen = options
        .label_groups
        .iter()
        .find(|g| g.group.name == "Rollen")
        .unwrap();
    assert_eq!(rollen.labels.len(), 2);
}

#[test]
fn rename_group_command() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let id = save_group(&mut s, "Alt", &Filter::default()).unwrap();
    verwaltung_app::commands::addresses::rename_group(&mut s, id, "Neu").unwrap();
    assert_eq!(list_groups(&s).unwrap()[0].name, "Neu");
}

#[test]
fn count_active_members_reads_only_and_counts_members() {
    use verwaltung_app::commands::addresses::count_active_members;
    use verwaltung_core::model::MemberRef;
    use verwaltung_core::repo::memberships::{self, MembershipInput};
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let org = s
        .write(|c| organisations::create(c, &org_input("Beck AG")))
        .unwrap();
    s.write(|c| {
        memberships::create(
            c,
            &MembershipInput {
                member: MemberRef::Organisation(org.id),
                category_id: None,
                joined_on: None,
                left_on: None,
            },
        )
    })
    .unwrap();
    assert_eq!(count_active_members(&s, today()).unwrap(), 1);
}

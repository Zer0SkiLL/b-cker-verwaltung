mod common;

use common::{fresh_session, org_input};
use verwaltung_app::commands::catalog::{
    create_label, create_label_group, create_lookup, delete_label_group, delete_lookup,
    list_label_tree, list_lookups, rename_lookup,
};
use verwaltung_core::model::{AppliesTo, LookupKind, MemberRef};
use verwaltung_core::repo::assignments::{self, AssignmentInput};
use verwaltung_core::repo::memberships::{self, MembershipInput};
use verwaltung_core::repo::organisations;

#[test]
fn delete_used_lookup_needs_confirm() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let cat = create_lookup(&mut s, LookupKind::MembershipCategory, "Aktivmitglied").unwrap();
    s.write(|c| {
        let org = organisations::create(c, &org_input("Beck AG"))?;
        let input = MembershipInput {
            member: MemberRef::Organisation(org.id),
            category_id: Some(cat.id),
            joined_on: None,
            left_on: None,
        };
        memberships::create(c, &input)
    })
    .unwrap();
    let listed = list_lookups(&s, LookupKind::MembershipCategory).unwrap();
    assert_eq!(listed[0].usage, 1);
    let err = delete_lookup(&mut s, cat.id, false).unwrap_err();
    assert!(err.0.contains("noch 1-mal verwendet"), "{}", err.0);
    delete_lookup(&mut s, cat.id, true).unwrap();
    assert!(list_lookups(&s, LookupKind::MembershipCategory).unwrap().is_empty());
}

#[test]
fn label_tree_counts_usage() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let group = create_label_group(&mut s, "Ausbildung", AppliesTo::Organisation).unwrap();
    let label = create_label(&mut s, group.id, "Lehrbetrieb").unwrap();
    create_label(&mut s, group.id, "ÜK-Zentrum").unwrap();
    s.write(|c| {
        let org = organisations::create(c, &org_input("Beck AG"))?;
        let input = AssignmentInput {
            label_id: label.id,
            target: MemberRef::Organisation(org.id),
            assigned_on: None,
            note: None,
        };
        assignments::assign(c, &input)
    })
    .unwrap();
    let tree = list_label_tree(&s).unwrap();
    assert_eq!(tree.len(), 1);
    let usages: Vec<i64> = tree[0].labels.iter().map(|l| l.usage).collect();
    assert_eq!(usages.iter().sum::<i64>(), 1);
    assert!(delete_label_group(&mut s, group.id, false).is_err());
    delete_label_group(&mut s, group.id, true).unwrap();
    assert!(list_label_tree(&s).unwrap().is_empty());
}

#[test]
fn rename_to_empty_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = fresh_session(dir.path());
    let f = create_lookup(&mut s, LookupKind::Function, "Inhaber").unwrap();
    let err = rename_lookup(&mut s, f.id, "   ").unwrap_err();
    assert!(!err.0.is_empty());
    assert_eq!(rename_lookup(&mut s, f.id, "Inhaberin").unwrap().name, "Inhaberin");
}

//! Frei pflegbare Auswahllisten und Labels.

use serde::Serialize;
use verwaltung_core::model::{AppliesTo, Label, LabelGroup, LookupKind, LookupValue};
use verwaltung_core::repo::{labels, lookups};

use crate::error::AppResult;
use crate::session::Session;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookupUsage {
    pub value: LookupValue,
    pub usage: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabelUsage {
    pub label: Label,
    pub usage: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabelGroupTree {
    pub group: LabelGroup,
    pub labels: Vec<LabelUsage>,
}

pub fn list_lookups(s: &Session, kind: LookupKind) -> AppResult<Vec<LookupUsage>> {
    s.read(|c| {
        lookups::list(c, kind)?
            .into_iter()
            .map(|value| {
                let usage = lookups::usage_count(c, value.id)?;
                Ok(LookupUsage { value, usage })
            })
            .collect()
    })
}

pub fn create_lookup(s: &mut Session, kind: LookupKind, name: &str) -> AppResult<LookupValue> {
    s.write(|c| lookups::create(c, kind, name))
}

pub fn rename_lookup(s: &mut Session, id: i64, name: &str) -> AppResult<LookupValue> {
    s.write(|c| lookups::rename(c, id, name))
}

pub fn delete_lookup(s: &mut Session, id: i64, confirm: bool) -> AppResult<()> {
    s.write(|c| lookups::delete(c, id, confirm))
}

pub fn list_label_tree(s: &Session) -> AppResult<Vec<LabelGroupTree>> {
    s.read(|c| {
        labels::list_groups(c)?
            .into_iter()
            .map(|group| {
                let labels = labels::list_labels(c, group.id)?
                    .into_iter()
                    .map(|label| {
                        let usage = labels::label_usage_count(c, label.id)?;
                        Ok(LabelUsage { label, usage })
                    })
                    .collect::<verwaltung_core::Result<Vec<_>>>()?;
                Ok(LabelGroupTree { group, labels })
            })
            .collect()
    })
}

pub fn create_label_group(
    s: &mut Session,
    name: &str,
    applies_to: AppliesTo,
) -> AppResult<LabelGroup> {
    s.write(|c| labels::create_group(c, name, applies_to))
}

pub fn create_label(s: &mut Session, group_id: i64, name: &str) -> AppResult<Label> {
    s.write(|c| labels::create_label(c, group_id, name))
}

pub fn rename_label(s: &mut Session, id: i64, name: &str) -> AppResult<Label> {
    s.write(|c| labels::rename_label(c, id, name))
}

pub fn delete_label(s: &mut Session, id: i64, confirm: bool) -> AppResult<()> {
    s.write(|c| labels::delete_label(c, id, confirm))
}

pub fn delete_label_group(s: &mut Session, id: i64, confirm: bool) -> AppResult<()> {
    s.write(|c| labels::delete_group(c, id, confirm))
}

pub fn rename_label_group(s: &mut Session, id: i64, name: &str) -> AppResult<LabelGroup> {
    s.write(|c| labels::rename_group(c, id, name))
}

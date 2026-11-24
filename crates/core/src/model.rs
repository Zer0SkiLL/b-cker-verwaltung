//! Gemeinsame, reine Datentypen.

use serde::{Deserialize, Serialize};

/// Art eines frei pflegbaren Listeneintrags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LookupKind {
    OrgType,
    Function,
    MembershipCategory,
    CantonalAssociation,
}

impl LookupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LookupKind::OrgType => "org_type",
            LookupKind::Function => "function",
            LookupKind::MembershipCategory => "membership_category",
            LookupKind::CantonalAssociation => "cantonal_association",
        }
    }

    pub fn parse(s: &str) -> Option<LookupKind> {
        [
            Self::OrgType,
            Self::Function,
            Self::MembershipCategory,
            Self::CantonalAssociation,
        ]
        .into_iter()
        .find(|k| k.as_str() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LookupValue {
    pub id: i64,
    pub kind: LookupKind,
    pub name: String,
    pub sort: i64,
}

/// Für welche Eintragsart eine Label-Gruppe gilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppliesTo {
    Person,
    Organisation,
    Both,
}

impl AppliesTo {
    pub fn as_str(self) -> &'static str {
        match self {
            AppliesTo::Person => "person",
            AppliesTo::Organisation => "organisation",
            AppliesTo::Both => "both",
        }
    }

    pub fn parse(s: &str) -> Option<AppliesTo> {
        [Self::Person, Self::Organisation, Self::Both]
            .into_iter()
            .find(|a| a.as_str() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelGroup {
    pub id: i64,
    pub name: String,
    pub applies_to: AppliesTo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub group_id: i64,
    pub name: String,
}

/// Verweis auf ein Mitglied bzw. Label-Ziel: Betrieb oder Person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRef {
    Organisation(i64),
    Person(i64),
}

impl MemberRef {
    /// (organisation_id, person_id) für die beiden Fremdschlüsselspalten.
    pub fn columns(self) -> (Option<i64>, Option<i64>) {
        match self {
            MemberRef::Organisation(id) => (Some(id), None),
            MemberRef::Person(id) => (None, Some(id)),
        }
    }

    pub fn from_columns(organisation_id: Option<i64>, person_id: Option<i64>) -> Option<MemberRef> {
        match (organisation_id, person_id) {
            (Some(o), None) => Some(MemberRef::Organisation(o)),
            (None, Some(p)) => Some(MemberRef::Person(p)),
            _ => None,
        }
    }
}

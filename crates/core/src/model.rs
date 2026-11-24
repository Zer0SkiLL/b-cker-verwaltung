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
        [Self::OrgType, Self::Function, Self::MembershipCategory, Self::CantonalAssociation]
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
        [Self::Person, Self::Organisation, Self::Both].into_iter().find(|a| a.as_str() == s)
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

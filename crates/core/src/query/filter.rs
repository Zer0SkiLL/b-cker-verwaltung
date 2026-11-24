use chrono::NaiveDate;
use rusqlite::{params_from_iter, Connection};
use serde::{Deserialize, Serialize};

use super::conditions::{build, Subject};
use crate::error::Result;
use crate::repo::{affiliations, organisations, persons};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Person,
    Organisation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MembershipStatus {
    #[default]
    Any,
    Active,
    Exited,
}

/// Kombinierbare Suchkriterien. Leere Listen bedeuten "keine Einschränkung".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Filter {
    pub text: Option<String>,
    pub kinds: Vec<EntryKind>,
    pub label_ids: Vec<i64>,
    pub team_ids: Vec<i64>,
    pub function_ids: Vec<i64>,
    pub org_type_ids: Vec<i64>,
    pub category_ids: Vec<i64>,
    pub cantonal_ids: Vec<i64>,
    pub membership: MembershipStatus,
}

impl Filter {
    fn wants(&self, kind: EntryKind) -> bool {
        self.kinds.is_empty() || self.kinds.contains(&kind)
    }

    /// Personenbezogene Kriterien schliessen Betriebe aus.
    fn excludes_organisations(&self) -> bool {
        !self.team_ids.is_empty() || !self.function_ids.is_empty()
    }
}

/// Ein Treffer: Person oder Betrieb mit Kontaktdaten für Listen, Mail und Serienbrief.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactRow {
    pub kind: EntryKind,
    pub id: i64,
    pub display_name: String,
    pub organisation: Option<String>,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub po_box: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
}

pub fn search(conn: &Connection, filter: &Filter, today: NaiveDate) -> Result<Vec<ContactRow>> {
    let mut rows = Vec::new();
    if filter.wants(EntryKind::Organisation) && !filter.excludes_organisations() {
        for id in matching_ids(conn, filter, Subject::Organisation, today)? {
            rows.push(organisation_row(&organisations::get(conn, id)?));
        }
    }
    if filter.wants(EntryKind::Person) {
        for id in matching_ids(conn, filter, Subject::Person, today)? {
            rows.push(person_row(conn, id, today)?);
        }
    }
    rows.sort_by_cached_key(|r| (r.display_name.to_lowercase(), r.id));
    Ok(rows)
}

fn matching_ids(
    conn: &Connection,
    filter: &Filter,
    s: Subject,
    today: NaiveDate,
) -> Result<Vec<i64>> {
    let w = build(conn, filter, s, today)?;
    let table = match s {
        Subject::Person => "persons p",
        Subject::Organisation => "organisations o",
    };
    let sql = format!("SELECT id FROM {table} WHERE {}", w.sql());
    let mut stmt = conn.prepare(&sql)?;
    let ids = stmt.query_map(params_from_iter(w.params.iter()), |r| r.get(0))?;
    Ok(ids.collect::<rusqlite::Result<Vec<i64>>>()?)
}

fn organisation_row(o: &organisations::Organisation) -> ContactRow {
    ContactRow {
        kind: EntryKind::Organisation,
        id: o.id,
        display_name: o.name.clone(),
        organisation: None,
        street: o.street.clone(),
        zip: o.zip.clone(),
        city: o.city.clone(),
        po_box: o.po_box.clone(),
        email: o.email.clone(),
        phone: o.phone.clone(),
        mobile: None,
    }
}

/// Personen ohne eigene Adresse bzw. Mail erhalten jene ihres ersten aktuellen Betriebs.
fn person_row(conn: &Connection, id: i64, today: NaiveDate) -> Result<ContactRow> {
    let p = persons::get(conn, id)?;
    let orgs = affiliations::for_person(conn, id, false, today)?
        .iter()
        .map(|a| organisations::get(conn, a.organisation_id))
        .collect::<Result<Vec<_>>>()?;
    let names: Vec<&str> = orgs.iter().map(|o| o.name.as_str()).collect();
    let first = orgs.first();
    let has_address = p.street.is_some() || p.zip.is_some() || p.city.is_some();
    let from_org = |pick: fn(&organisations::Organisation) -> Option<String>| first.and_then(pick);
    Ok(ContactRow {
        kind: EntryKind::Person,
        id,
        display_name: p.display_name(),
        organisation: (!names.is_empty()).then(|| names.join(", ")),
        street: if has_address {
            p.street.clone()
        } else {
            from_org(|o| o.street.clone())
        },
        zip: if has_address {
            p.zip.clone()
        } else {
            from_org(|o| o.zip.clone())
        },
        city: if has_address {
            p.city.clone()
        } else {
            from_org(|o| o.city.clone())
        },
        po_box: if has_address {
            None
        } else {
            from_org(|o| o.po_box.clone())
        },
        email: p.email.clone().or_else(|| from_org(|o| o.email.clone())),
        phone: p.phone.clone().or_else(|| from_org(|o| o.phone.clone())),
        mobile: p.mobile.clone(),
    })
}

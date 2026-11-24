//! Baut die WHERE-Bedingungen für Personen und Betriebe. Werte werden nur als Parameter gebunden.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rusqlite::types::Value;
use rusqlite::Connection;

use super::filter::{Filter, MembershipStatus};
use crate::error::Result;
use crate::text::normalize_search;

/// Gesammelte Bedingungen (mit AND verknüpft) und ihre Parameter in Reihenfolge.
#[derive(Default)]
pub(super) struct Where {
    pub clauses: Vec<String>,
    pub params: Vec<Value>,
}

impl Where {
    fn push(&mut self, clause: String, params: impl IntoIterator<Item = Value>) {
        self.clauses.push(clause);
        self.params.extend(params);
    }

    pub fn sql(&self) -> String {
        if self.clauses.is_empty() {
            "1 = 1".into()
        } else {
            self.clauses.join(" AND ")
        }
    }
}

/// Welche Tabelle gerade gefiltert wird.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Subject {
    Person,
    Organisation,
}

impl Subject {
    fn alias(self) -> &'static str {
        match self {
            Subject::Person => "p",
            Subject::Organisation => "o",
        }
    }

    fn fk(self) -> &'static str {
        match self {
            Subject::Person => "person_id",
            Subject::Organisation => "organisation_id",
        }
    }
}

const CURRENT: &str = "(a.valid_to IS NULL OR a.valid_to >= ?)";

pub(super) fn build(conn: &Connection, f: &Filter, s: Subject, today: NaiveDate) -> Result<Where> {
    let mut w = Where::default();
    let day = Value::Text(today.to_string());
    text(&mut w, f, s);
    labels(conn, &mut w, f, s)?;
    if s == Subject::Person {
        any_of(&mut w, "EXISTS (SELECT 1 FROM team_members tm WHERE tm.person_id = p.id AND tm.team_id IN ({}))", &f.team_ids, &[]);
        let sql = format!("EXISTS (SELECT 1 FROM affiliations a WHERE a.person_id = p.id AND {CURRENT} AND a.function_id IN ({{}}))");
        any_of(&mut w, &sql, &f.function_ids, std::slice::from_ref(&day));
    }
    org_column(&mut w, s, "org_type_id", &f.org_type_ids, &day);
    org_column(&mut w, s, "cantonal_association_id", &f.cantonal_ids, &day);
    membership(&mut w, f, s, &day);
    Ok(w)
}

fn text(w: &mut Where, f: &Filter, s: Subject) {
    let normalized = normalize_search(&[f.text.as_deref()]);
    for word in normalized.split_whitespace() {
        w.push(
            format!("instr({}.search_text, ?) > 0", s.alias()),
            [Value::Text(word.into())],
        );
    }
}

/// Labels derselben Gruppe mit ODER, verschiedener Gruppen mit UND.
fn labels(conn: &Connection, w: &mut Where, f: &Filter, s: Subject) -> Result<()> {
    if f.label_ids.is_empty() {
        return Ok(());
    }
    let mut groups: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    let mut stmt = conn.prepare("SELECT group_id FROM labels WHERE id = ?1")?;
    for id in &f.label_ids {
        match stmt.query_row([id], |r| r.get::<_, i64>(0)) {
            Ok(group) => groups.entry(group).or_default().push(*id),
            Err(rusqlite::Error::QueryReturnedNoRows) => w.push("0 = 1".into(), []),
            Err(e) => return Err(e.into()),
        }
    }
    let template = format!(
        "EXISTS (SELECT 1 FROM label_assignments la WHERE la.{} = {}.id AND la.label_id IN ({{}}))",
        s.fk(),
        s.alias()
    );
    for ids in groups.values() {
        any_of(w, &template, ids, &[]);
    }
    Ok(())
}

/// Betriebs-Spalte: Betriebe direkt, Personen über eine aktuelle Zugehörigkeit.
fn org_column(w: &mut Where, s: Subject, column: &str, ids: &[i64], day: &Value) {
    let template = match s {
        Subject::Organisation => format!("o.{column} IN ({{}})"),
        Subject::Person => format!(
            "EXISTS (SELECT 1 FROM affiliations a JOIN organisations ao ON ao.id = a.organisation_id \
             WHERE a.person_id = p.id AND {CURRENT} AND ao.{column} IN ({{}}))"
        ),
    };
    let leading = if s == Subject::Person {
        vec![day.clone()]
    } else {
        vec![]
    };
    any_of(w, &template, ids, &leading);
}

const ACTIVE: &str = "(m.left_on IS NULL OR m.left_on > ?)";
const LEFT: &str = "(m.left_on IS NOT NULL AND m.left_on <= ?)";

/// Kategorie und Status müssen von derselben Mitgliedschaft erfüllt werden.
/// "Ausgetreten" heisst zusätzlich: keine aktive Mitgliedschaft (wieder Eingetretene zählen nicht).
fn membership(w: &mut Where, f: &Filter, s: Subject, day: &Value) {
    let status = match f.membership {
        MembershipStatus::Any => None,
        MembershipStatus::Active => Some(ACTIVE),
        MembershipStatus::Exited => Some(LEFT),
    };
    if status.is_none() && f.category_ids.is_empty() {
        return;
    }
    let (owner, mut params) = owner_clause(s, day);
    let mut parts = vec![owner];
    if let Some(cond) = status {
        parts.push(cond.into());
        params.push(day.clone());
    }
    if !f.category_ids.is_empty() {
        parts.push(format!(
            "m.category_id IN ({})",
            placeholders(f.category_ids.len())
        ));
        params.extend(f.category_ids.iter().map(|id| Value::Integer(*id)));
    }
    w.push(
        format!(
            "EXISTS (SELECT 1 FROM memberships m WHERE {})",
            parts.join(" AND ")
        ),
        params,
    );
    if f.membership == MembershipStatus::Exited {
        let (owner, mut params) = owner_clause(s, day);
        params.push(day.clone());
        w.push(
            format!("NOT EXISTS (SELECT 1 FROM memberships m WHERE {owner} AND {ACTIVE})"),
            params,
        );
    }
}

/// Wem eine Mitgliedschaft "gehört": dem Betrieb bzw. der Person oder ihren aktuellen Betrieben.
fn owner_clause(s: Subject, day: &Value) -> (String, Vec<Value>) {
    match s {
        Subject::Organisation => ("m.organisation_id = o.id".to_string(), vec![]),
        Subject::Person => (
            format!(
                "(m.person_id = p.id OR m.organisation_id IN \
                 (SELECT a.organisation_id FROM affiliations a WHERE a.person_id = p.id AND {CURRENT}))"
            ),
            vec![day.clone()],
        ),
    }
}

/// Fügt `template` mit `{}` als Platzhalterliste ein, falls `ids` nicht leer ist.
fn any_of(w: &mut Where, template: &str, ids: &[i64], leading: &[Value]) {
    if ids.is_empty() {
        return;
    }
    let clause = template.replacen("{}", &placeholders(ids.len()), 1);
    let params = leading
        .iter()
        .cloned()
        .chain(ids.iter().map(|id| Value::Integer(*id)));
    w.push(clause, params);
}

fn placeholders(n: usize) -> String {
    vec!["?"; n].join(", ")
}

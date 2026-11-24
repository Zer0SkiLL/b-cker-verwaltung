//! Schreibt einen geprüften Import-Plan in einer einzigen Transaktion in die Datenbank.

use rusqlite::Connection;

use super::mapping::{ImportPlan, PlannedRow, ReportEntry, FLAG_GROUP};
use crate::error::{CoreError, Result};
use crate::model::{AppliesTo, LookupKind, MemberRef};
use crate::repo::affiliations::{self, AffiliationInput};
use crate::repo::assignments::{self, AssignmentInput};
use crate::repo::memberships::{self, MembershipInput};
use crate::repo::organisations::{self, OrganisationInput};
use crate::repo::{labels, lookups, persons};

/// Funktion, mit der importierte Personen ihrem Betrieb zugeordnet werden.
pub const IMPORT_FUNCTION: &str = "Inhaber";

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSummary {
    pub organisations: usize,
    pub persons: usize,
    pub report: Vec<ReportEntry>,
}

/// Importiert alles oder nichts. Nur in eine Datenbank ohne Betriebe und Personen.
pub fn apply(conn: &mut Connection, plan: &ImportPlan) -> Result<ImportSummary> {
    let existing: i64 = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM organisations) + (SELECT COUNT(*) FROM persons)",
        [],
        |r| r.get(0),
    )?;
    if existing > 0 {
        return Err(CoreError::DatabaseNotEmpty);
    }
    let tx = conn.transaction()?;
    let mut summary = ImportSummary {
        organisations: 0,
        persons: 0,
        report: plan.report.clone(),
    };
    for row in &plan.rows {
        import_row(&tx, row).map_err(|e| CoreError::Import(format!("Zeile {}: {e}", row.line)))?;
        summary.organisations += 1;
        summary.persons += usize::from(row.person.is_some());
    }
    tx.commit()?;
    Ok(summary)
}

fn import_row(conn: &Connection, row: &PlannedRow) -> Result<()> {
    let cantonal_association_id = match &row.cantonal {
        Some(name) => {
            Some(lookups::find_or_create(conn, LookupKind::CantonalAssociation, name)?.id)
        }
        None => None,
    };
    let org = organisations::create(
        conn,
        &OrganisationInput {
            cantonal_association_id,
            ..row.organisation.clone()
        },
    )?;
    let category_id = match &row.category {
        Some(name) => Some(lookups::find_or_create(conn, LookupKind::MembershipCategory, name)?.id),
        None => None,
    };
    let member = MemberRef::Organisation(org.id);
    memberships::create(
        conn,
        &MembershipInput {
            member,
            category_id,
            joined_on: row.joined,
            left_on: row.left,
        },
    )?;
    for flag in &row.flags {
        let label = labels::find_or_create_label(conn, FLAG_GROUP, AppliesTo::Organisation, flag)?;
        assignments::assign(
            conn,
            &AssignmentInput {
                label_id: label.id,
                target: member,
                assigned_on: None,
                note: None,
            },
        )?;
    }
    if let Some(person_input) = &row.person {
        let person = persons::create(conn, person_input)?;
        let function_id = lookups::find_or_create(conn, LookupKind::Function, IMPORT_FUNCTION)?.id;
        let link = AffiliationInput {
            person_id: person.id,
            organisation_id: org.id,
            function_id,
            valid_from: None,
            valid_to: None,
        };
        affiliations::create(conn, &link)?;
    }
    Ok(())
}

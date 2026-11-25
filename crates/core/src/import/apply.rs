//! Schreibt einen geprüften Import-Plan in einer einzigen Transaktion in die Datenbank.

use chrono::NaiveDate;
use rusqlite::Connection;

use super::lookup::{label_or_create, lookup_or_create};
use super::mapping::{ReportEntry, FLAG_GROUP};
use super::plan::{ImportPlan, PlannedIndividual, PlannedOrganisation, PlannedPerson};
use crate::error::{CoreError, Result};
use crate::model::{AppliesTo, LookupKind, MemberRef};
use crate::repo::affiliations::{self, AffiliationInput};
use crate::repo::assignments::{self, AssignmentInput};
use crate::repo::memberships::{self, MembershipInput};
use crate::repo::organisations::{self, OrganisationInput};
use crate::repo::persons;

/// Funktion, mit der importierte Personen ihrem Betrieb zugeordnet werden (danach änderbar).
pub const IMPORT_FUNCTION: &str = "Kontaktperson";
/// Label-Gruppe für die Spalte „Mandat“.
pub const MANDATE_GROUP: &str = "Mandat";

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSummary {
    pub organisations: usize,
    pub persons: usize,
    pub individuals: usize,
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
        individuals: 0,
        report: plan.report.clone(),
    };
    for org in &plan.organisations {
        import_organisation(&tx, org)
            .map_err(|e| CoreError::Import(format!("Zeile {}: {e}", org.lines[0])))?;
        summary.organisations += 1;
        summary.persons += org.persons.len();
    }
    for ind in &plan.individuals {
        import_individual(&tx, ind)
            .map_err(|e| CoreError::Import(format!("Zeile {}: {e}", ind.person.line)))?;
        summary.individuals += 1;
    }
    tx.commit()?;
    log::info!(
        "Import: {} Betriebe, {} Personen, {} Einzelmitglieder",
        summary.organisations,
        summary.persons,
        summary.individuals
    );
    Ok(summary)
}

fn import_organisation(conn: &Connection, org: &PlannedOrganisation) -> Result<()> {
    let cantonal_association_id = match &org.cantonal {
        Some(name) => Some(lookup_or_create(conn, LookupKind::CantonalAssociation, name)?.id),
        None => None,
    };
    let created = organisations::create(
        conn,
        &OrganisationInput {
            cantonal_association_id,
            ..org.input.clone()
        },
    )?;
    let member = MemberRef::Organisation(created.id);
    membership(conn, member, &org.category, org.joined, org.left)?;
    for flag in &org.flags {
        assign(
            conn,
            label_or_create(conn, FLAG_GROUP, AppliesTo::Organisation, flag)?.id,
            member,
        )?;
    }
    mandates(conn, &org.mandates, member)?;
    for person in &org.persons {
        let id = create_person(conn, person)?;
        let function_id = lookup_or_create(conn, LookupKind::Function, IMPORT_FUNCTION)?.id;
        affiliations::create(
            conn,
            &AffiliationInput {
                person_id: id,
                organisation_id: created.id,
                function_id,
                valid_from: None,
                valid_to: None,
            },
        )?;
    }
    Ok(())
}

fn import_individual(conn: &Connection, ind: &PlannedIndividual) -> Result<()> {
    let id = create_person(conn, &ind.person)?;
    membership(
        conn,
        MemberRef::Person(id),
        &ind.category,
        ind.joined,
        ind.left,
    )
}

fn create_person(conn: &Connection, person: &PlannedPerson) -> Result<i64> {
    let created = persons::create(conn, &person.input)?;
    mandates(conn, &person.mandates, MemberRef::Person(created.id))?;
    Ok(created.id)
}

fn membership(
    conn: &Connection,
    member: MemberRef,
    category: &Option<String>,
    joined_on: Option<NaiveDate>,
    left_on: Option<NaiveDate>,
) -> Result<()> {
    let category_id = match category {
        Some(name) => Some(lookup_or_create(conn, LookupKind::MembershipCategory, name)?.id),
        None => None,
    };
    memberships::create(
        conn,
        &MembershipInput {
            member,
            category_id,
            joined_on,
            left_on,
        },
    )
    .map(|_| ())
}

fn mandates(conn: &Connection, names: &[String], target: MemberRef) -> Result<()> {
    for name in names {
        assign(
            conn,
            label_or_create(conn, MANDATE_GROUP, AppliesTo::Both, name)?.id,
            target,
        )?;
    }
    Ok(())
}

fn assign(conn: &Connection, label_id: i64, target: MemberRef) -> Result<()> {
    assignments::assign(
        conn,
        &AssignmentInput {
            label_id,
            target,
            assigned_on: None,
            note: None,
        },
    )
    .map(|_| ())
}

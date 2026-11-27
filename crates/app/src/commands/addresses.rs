use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use verwaltung_core::contact::{email_list, mailto_bcc};
use verwaltung_core::export::{contact_table, write_csv, write_xlsx};
use verwaltung_core::model::{Label, LabelGroup, LookupKind, LookupValue};
use verwaltung_core::query::filter::{search, ContactRow, Filter};
use verwaltung_core::query::saved_groups;
use verwaltung_core::repo::teams::Team;
use verwaltung_core::repo::{labels, lookups, memberships, teams};

use crate::error::AppResult;
use crate::session::Session;

const SHEET_NAME: &str = "Adressen";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub rows: Vec<ContactRow>,
    pub email_count: usize,
    pub missing_email: Vec<ContactRow>,
    pub joined_emails: String,
    pub mailto: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Xlsx,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GroupDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LabelGroupWithLabels {
    pub group: LabelGroup,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOptions {
    pub label_groups: Vec<LabelGroupWithLabels>,
    pub teams: Vec<Team>,
    pub functions: Vec<LookupValue>,
    pub org_types: Vec<LookupValue>,
    pub categories: Vec<LookupValue>,
    pub cantonal: Vec<LookupValue>,
}

/// Treffer plus alles, was zum Kontaktieren der Liste nötig ist.
/// Anzahl aktiver Mitglieder (nur lesend), für den Meilenstein in der Oberfläche.
pub fn count_active_members(s: &Session, today: NaiveDate) -> AppResult<i64> {
    s.read(|c| memberships::count_active_members(c, today))
}

pub fn search_addresses(s: &Session, filter: &Filter, today: NaiveDate) -> AppResult<SearchResult> {
    let rows = s.read(|c| search(c, filter, today))?;
    let emails = email_list(&rows);
    let mailto = if emails.addresses.is_empty() {
        None
    } else {
        mailto_bcc(&emails.addresses)
    };
    Ok(SearchResult {
        email_count: emails.addresses.len(),
        missing_email: emails.missing,
        joined_emails: emails.joined,
        mailto,
        rows,
    })
}

/// Exportiert die gefilterte Liste (Adressliste für Serienbriefe).
pub fn export_addresses(
    s: &Session,
    filter: &Filter,
    today: NaiveDate,
    path: &Path,
    format: ExportFormat,
) -> AppResult<()> {
    let started = std::time::Instant::now();
    let rows = s.read(|c| search(c, filter, today))?;
    let table = contact_table(&rows);
    match format {
        ExportFormat::Csv => write_csv(&table, path)?,
        ExportFormat::Xlsx => write_xlsx(&table, path, SHEET_NAME)?,
    }
    log::info!(
        "Export Adressen: {} Zeilen, {} ms",
        table.rows.len(),
        started.elapsed().as_millis()
    );
    Ok(())
}

pub fn list_groups(s: &Session) -> AppResult<Vec<GroupDto>> {
    let groups = s.read(saved_groups::list)?;
    Ok(groups
        .into_iter()
        .map(|(id, name)| GroupDto { id, name })
        .collect())
}

pub fn save_group(s: &mut Session, name: &str, filter: &Filter) -> AppResult<i64> {
    s.write(|c| saved_groups::save(c, name, filter))
}

pub fn load_group(s: &Session, id: i64) -> AppResult<Filter> {
    s.read(|c| saved_groups::load(c, id))
}

pub fn delete_group(s: &mut Session, id: i64) -> AppResult<()> {
    s.write(|c| saved_groups::delete(c, id))
}

/// Auswahlwerte für die Filterleiste.
pub fn filter_options(s: &Session) -> AppResult<FilterOptions> {
    s.read(|c| {
        let label_groups = labels::list_groups(c)?
            .into_iter()
            .map(|group| {
                let labels = labels::list_labels(c, group.id)?;
                Ok(LabelGroupWithLabels { group, labels })
            })
            .collect::<verwaltung_core::Result<Vec<_>>>()?;
        Ok(FilterOptions {
            label_groups,
            teams: teams::list(c)?,
            functions: lookups::list(c, LookupKind::Function)?,
            org_types: lookups::list(c, LookupKind::OrgType)?,
            categories: lookups::list(c, LookupKind::MembershipCategory)?,
            cantonal: lookups::list(c, LookupKind::CantonalAssociation)?,
        })
    })
}

pub fn rename_group(s: &mut Session, id: i64, name: &str) -> AppResult<()> {
    s.write(|c| saved_groups::rename(c, id, name))
}

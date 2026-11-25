//! Legt eine Datei mit fiktiven, festen Beispieldaten an (für Handbuch-Bildschirmfotos).
//! Aufruf: cargo run -p verwaltung_core --example beispieldaten -- <pfad>

use std::path::Path;
use std::process::ExitCode;

use chrono::{Local, NaiveDate, Utc};
use rusqlite::Connection;
use verwaltung_core::model::{AppliesTo, LookupKind, MemberRef};
use verwaltung_core::query::filter::Filter;
use verwaltung_core::query::saved_groups;
use verwaltung_core::repo::affiliations::{self, AffiliationInput};
use verwaltung_core::repo::assignments::{self, AssignmentInput};
use verwaltung_core::repo::defaults::{
    self, INTAKE_GROUP, MEMBERSHIP_CATEGORIES, PROVISIONAL_LABEL,
};
use verwaltung_core::repo::memberships::{self, MembershipInput};
use verwaltung_core::repo::organisations::{self, OrganisationInput};
use verwaltung_core::repo::persons::{self, PersonInput};
use verwaltung_core::repo::settings::{self, LAST_SECONDARY_BACKUP, SECONDARY_BACKUP_DIR};
use verwaltung_core::repo::{labels, lookups, teams};
use verwaltung_core::storage::lock::Identity;
use verwaltung_core::storage::workspace::Workspace;
use verwaltung_core::Result;

/// Name, Ort, PLZ, Kanton, Kategorie (Index in MEMBERSHIP_CATEGORIES).
const BETRIEBE: [(&str, &str, &str, &str, usize); 15] = [
    ("Bäckerei Muster AG", "St. Gallen", "9000", "SG", 0),
    ("Confiserie Beispiel GmbH", "Chur", "7000", "GR", 0),
    ("Backstube Probe", "Appenzell", "9050", "AI", 0),
    ("Bäckerei Exempel", "Herisau", "9100", "AR", 0),
    ("Brotzeit Muster", "Glarus", "8750", "GL", 0),
    ("Bäckerei Fiktiv AG", "Frauenfeld", "8500", "TG", 0),
    ("Konditorei Vorlage", "Wil", "9500", "SG", 0),
    ("Bäckerei Platzhalter", "Davos", "7270", "GR", 0),
    ("Confiserie Testmann", "Arbon", "9320", "TG", 1),
    ("Beck Beispielhof", "Rapperswil", "8640", "SG", 1),
    ("Bäckerei Musterli", "Trogen", "9043", "AR", 2),
    ("Lehrbäckerei Demo", "Landquart", "7302", "GR", 3),
    ("Bäckerei Neuanfang", "Gossau", "9200", "SG", 0),
    ("Confiserie Frischling", "Weinfelden", "8570", "TG", 1),
    ("Bäckerei Abschied", "Buchs", "9470", "SG", 0),
];
/// Index 12 und 13 sind provisorisch (Label, ohne Eintrittsdatum), Index 14 ist ausgetreten.
const PROVISORISCH: [usize; 2] = [12, 13];
const AUSGETRETEN: usize = 14;

const VORNAMEN: [&str; 25] = [
    "Anna", "Beat", "Claudia", "Daniel", "Eva", "Fabian", "Gabriela", "Hans", "Iris", "Jonas",
    "Karin", "Lukas", "Monika", "Nico", "Olivia", "Peter", "Rahel", "Simon", "Tanja", "Urs",
    "Vera", "Walter", "Yvonne", "Marco", "Sandra",
];
const NACHNAMEN: [&str; 25] = [
    "Muster",
    "Beispiel",
    "Probe",
    "Exempel",
    "Vorlage",
    "Fiktiv",
    "Platzhalter",
    "Testmann",
    "Musterli",
    "Demo",
    "Muster",
    "Beispiel",
    "Probe",
    "Exempel",
    "Vorlage",
    "Fiktiv",
    "Platzhalter",
    "Testmann",
    "Musterli",
    "Demo",
    "Neuanfang",
    "Frischling",
    "Abschied",
    "Beispielhof",
    "Muster",
];

fn date(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

fn label(conn: &Connection, group: &str, applies: AppliesTo, name: &str) -> Result<i64> {
    Ok(labels::find_or_create_label(conn, group, applies, name)?.id)
}

fn assign(conn: &Connection, label_id: i64, target: MemberRef) -> Result<()> {
    let input = AssignmentInput {
        label_id,
        target,
        assigned_on: None,
        note: None,
    };
    assignments::assign(conn, &input).map(|_| ())
}

fn betriebe(conn: &Connection) -> Result<Vec<i64>> {
    let categories: Vec<i64> = MEMBERSHIP_CATEGORIES
        .iter()
        .map(|name| {
            lookups::find_or_create(conn, LookupKind::MembershipCategory, name).map(|v| v.id)
        })
        .collect::<Result<_>>()?;
    let provisional = label(conn, INTAKE_GROUP, AppliesTo::Both, PROVISIONAL_LABEL)?;
    let mut ids = Vec::new();
    for (index, (name, city, zip, canton, category)) in BETRIEBE.iter().enumerate() {
        let canton_id = lookups::find_or_create(conn, LookupKind::CantonalAssociation, canton)?.id;
        let org = organisations::create(
            conn,
            &OrganisationInput {
                name: (*name).into(),
                street: Some(format!("Musterstrasse {}", index + 1)),
                zip: Some((*zip).into()),
                city: Some((*city).into()),
                phone: Some(format!("071 000 00 {:02}", index + 1)),
                email: Some(format!("info{}@beispiel.ch", index + 1)),
                cantonal_association_id: Some(canton_id),
                ..Default::default()
            },
        )?;
        let provisorisch = PROVISORISCH.contains(&index);
        memberships::create(
            conn,
            &MembershipInput {
                member: MemberRef::Organisation(org.id),
                category_id: Some(categories[*category]),
                joined_on: if provisorisch {
                    None
                } else {
                    date(2010 + index as i32, 5, 1)
                },
                left_on: if index == AUSGETRETEN {
                    date(2025, 12, 31)
                } else {
                    None
                },
            },
        )?;
        if provisorisch {
            assign(conn, provisional, MemberRef::Organisation(org.id))?;
        }
        ids.push(org.id);
    }
    Ok(ids)
}

fn personen(conn: &Connection, orgs: &[i64]) -> Result<Vec<i64>> {
    let functions: Vec<i64> = ["Inhaber/in", "Geschäftsführer/in", "Lernende/r"]
        .iter()
        .map(|name| lookups::find_or_create(conn, LookupKind::Function, name).map(|v| v.id))
        .collect::<Result<_>>()?;
    let mut ids = Vec::new();
    for (index, (first, last)) in VORNAMEN.iter().zip(NACHNAMEN.iter()).enumerate() {
        let person = persons::create(
            conn,
            &PersonInput {
                first_name: Some((*first).into()),
                last_name: (*last).into(),
                email: Some(format!(
                    "{}.{}@beispiel.ch",
                    first.to_lowercase(),
                    index + 1
                )),
                mobile: Some(format!("079 000 00 {:02}", index + 1)),
                ..Default::default()
            },
        )?;
        affiliations::create(
            conn,
            &AffiliationInput {
                person_id: person.id,
                organisation_id: orgs[index % orgs.len()],
                function_id: functions[index / orgs.len()],
                valid_from: None,
                valid_to: None,
            },
        )?;
        ids.push(person.id);
    }
    Ok(ids)
}

fn labels_gruppen_team(conn: &Connection, orgs: &[i64], people: &[i64]) -> Result<()> {
    let lehrbetrieb = label(conn, "Ausbildung", AppliesTo::Organisation, "Lehrbetrieb")?;
    label(conn, "Ausbildung", AppliesTo::Organisation, "ÜK-Zentrum")?;
    let vorstand = label(conn, "Gremien", AppliesTo::Person, "Vorstand")?;
    label(conn, "Gremien", AppliesTo::Person, "Revisor/in")?;
    for org in orgs.iter().step_by(3) {
        assign(conn, lehrbetrieb, MemberRef::Organisation(*org))?;
    }
    for person in &people[..5] {
        assign(conn, vorstand, MemberRef::Person(*person))?;
    }
    let lehr = Filter {
        label_ids: vec![lehrbetrieb],
        ..Default::default()
    };
    saved_groups::save(conn, "Lehrbetriebe", &lehr)?;
    let vs = Filter {
        label_ids: vec![vorstand],
        ..Default::default()
    };
    saved_groups::save(conn, "Vorstand", &vs)?;
    let team = teams::create(
        conn,
        "Prüfungsexperten",
        Some("Experten für die Lehrabschlussprüfung"),
    )?;
    for (person, role) in people[5..9]
        .iter()
        .zip(["Leitung", "Experte", "Expertin", "Experte"])
    {
        teams::add_member(conn, team.id, *person, Some(role))?;
    }
    Ok(())
}

fn fill(conn: &Connection) -> Result<(usize, usize)> {
    defaults::seed_new_file(conn)?;
    let orgs = betriebe(conn)?;
    let people = personen(conn, &orgs)?;
    labels_gruppen_team(conn, &orgs, &people)?;
    Ok((orgs.len(), people.len()))
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Aufruf: beispieldaten <pfad>");
        return ExitCode::FAILURE;
    };
    let path = Path::new(&path);
    if path.exists() {
        eprintln!("Die Datei existiert bereits: {}", path.display());
        return ExitCode::FAILURE;
    }
    let who = Identity {
        user: "Beispieldaten".into(),
        host: "beispiel".into(),
    };
    let result = Workspace::open(path, &who).and_then(|ws| {
        let counts = fill(&ws.conn)?;
        ws.close(Local::now())?;
        // Erst nach dem Schliessen: ein zweiter Sicherungsort (fiktiv, wie unter Windows), damit die
        // Bilder nicht überall die Erinnerung „Kein zweiter Sicherungsort“ zeigen. Vor dem Schliessen
        // würde das Programm dorthin zu sichern versuchen.
        let conn = Connection::open(path)?;
        settings::set(
            &conn,
            SECONDARY_BACKUP_DIR,
            "E:\\Sicherung Mitgliederverwaltung",
        )?;
        settings::set(&conn, LAST_SECONDARY_BACKUP, &Utc::now().to_rfc3339())?;
        Ok(counts)
    });
    match result {
        Ok((orgs, people)) => {
            println!("Beispieldaten: {orgs} Betriebe, {people} Personen");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Beispieldaten konnten nicht angelegt werden: {err}");
            ExitCode::FAILURE
        }
    }
}

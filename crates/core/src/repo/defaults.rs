//! Vorgaben, die eine neu angelegte Datei mitbringt (Kategorien des Vereins, Label „provisorisch“).

use rusqlite::{params, Connection};

use super::{labels, lookups};
use crate::error::Result;
use crate::model::{AppliesTo, LookupKind};

pub const MEMBERSHIP_CATEGORIES: [&str; 4] = [
    "Mitglied mit Geschäft",
    "Mitglied ohne Geschäft",
    "Mitglied ohne SBC",
    "Kein Mitglied",
];
pub const INTAKE_GROUP: &str = "Aufnahme";
pub const PROVISIONAL_LABEL: &str = "provisorisch";

/// Legt die Vorgaben an; bereits vorhandene Einträge bleiben unverändert (mehrfach aufrufbar).
pub fn seed_new_file(conn: &Connection) -> Result<()> {
    for (index, name) in MEMBERSHIP_CATEGORIES.iter().enumerate() {
        let value = lookups::find_or_create(conn, LookupKind::MembershipCategory, name)?;
        conn.execute(
            "UPDATE lookup_values SET sort = ?1 WHERE id = ?2",
            params![index as i64, value.id],
        )?;
    }
    labels::find_or_create_label(conn, INTAKE_GROUP, AppliesTo::Both, PROVISIONAL_LABEL)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn names(conn: &Connection) -> Vec<String> {
        lookups::list(conn, LookupKind::MembershipCategory)
            .unwrap()
            .into_iter()
            .map(|v| v.name)
            .collect()
    }

    #[test]
    fn seed_creates_categories_in_order() {
        let conn = db::open_in_memory().unwrap();
        seed_new_file(&conn).unwrap();
        assert_eq!(names(&conn), MEMBERSHIP_CATEGORIES.to_vec());
    }

    #[test]
    fn seed_creates_intake_label() {
        let conn = db::open_in_memory().unwrap();
        seed_new_file(&conn).unwrap();
        let groups = labels::list_groups(&conn).unwrap();
        let group = groups
            .iter()
            .find(|g| g.name == INTAKE_GROUP)
            .expect("Gruppe fehlt");
        assert_eq!(group.applies_to, AppliesTo::Both);
        let list = labels::list_labels(&conn, group.id).unwrap();
        assert_eq!(
            list.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
            vec![PROVISIONAL_LABEL]
        );
    }

    #[test]
    fn seed_is_idempotent() {
        let conn = db::open_in_memory().unwrap();
        seed_new_file(&conn).unwrap();
        seed_new_file(&conn).unwrap();
        assert_eq!(names(&conn).len(), 4);
        let group = labels::list_groups(&conn)
            .unwrap()
            .into_iter()
            .find(|g| g.name == INTAKE_GROUP)
            .unwrap();
        assert_eq!(labels::list_labels(&conn, group.id).unwrap().len(), 1);
    }
}

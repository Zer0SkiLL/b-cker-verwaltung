# Bereit für den ersten Einsatz – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Der Excel-Import passt zur echten Liste (mit vollständiger Vorschau und Rückgängig), die Sicherung arbeitet mit einstellbarer Aufbewahrung nach Alter und Sicherung während der Arbeit, und „Problem melden“ erzeugt einen Bericht ohne Vereinsdaten.

**Architecture:**
- **Import:** Der Core plant den Import als reine Funktion (`plan`) in eine neue, gegliederte Struktur (Betriebe mit Personen, Einzelmitglieder, übersprungene Zeilen) mit Zeilenabgleich. `apply` schreibt sie in einer Transaktion. Die App-Schicht ergänzt Vorschau-DTO, „neu angelegt“ (Abgleich mit der Datei), Excel-Export der Vorschau, Sicherung vor dem Import und Rückgängig.
- **Sicherung:** Die Aufbewahrung ist eine reine Funktion über Zeitstempel. Die Policy liegt in `settings`. `Session` bekommt „sichern, falls geändert“ und merkt sich Fehler am zweiten Ort.
- **Protokoll:** über die `log`-Fassade, im Programm mit `tauri-plugin-log`.
- **Aktionsliste:** im Frontend als Ringpuffer in `localStorage`, gespeist aus `call()` und dem Router.

**Tech Stack:** Rust (rusqlite, rust_xlsxwriter, calamine schon vorhanden; neu `log`, `tauri-plugin-log`, `os_info`), Svelte 5/TS, Vitest, `node:test`, WebdriverIO.

**Spec:** `docs/superpowers/specs/2026-11-25-erster-einsatz-design.md`

## Global Constraints

- Alle Texte für Nutzer sind deutsch in Schweizer Schreibweise („ss“). Das Handbuch spricht in der Sie-Form, Knopfnamen stehen fett und wörtlich.
- **Keine Vereinsdaten** in Protokoll, Aktionsliste oder Bericht: keine Argumente oder Rückgabewerte von Befehlen, keine Namen, Adressen, Mailadressen und keine Dateipfade in eigenen Protokollzeilen.
- `docs/documentation/` (Musterdatei, PDF) nie committen und keine Inhalte ausgeben. Lesen ist nur lokal zur Prüfung erlaubt (Task 5).
- Migrationen nur ergänzend: `0003` fügt Spalten hinzu.
- Standards der Aufbewahrung: alle 2 Tage, täglich 14 Tage, wöchentlich 8 Wochen, monatlich 12 Monate, Intervall 60 Minuten.
- Grenzen: A 1–30, B A–90, C 1–104, D 0–120, Intervall 0 oder 15–480.
- Funktion importierter Personen: „Kontaktperson“. Label-Gruppe „Mandat“ (für beide).
- Kategorien und Listen werden beim Import ohne Gross-/Kleinschreibung abgeglichen. `find_or_create` und `UNIQUE` bleiben unverändert.
- TDD. Alle CI-Befehle müssen grün sein (siehe CLAUDE.md, Abschnitt „Befehle“).
- Commit-Stil gemäss CLAUDE.md, Branch `feat/erster-einsatz`. Pushen auf diesen Branch ist erlaubt, ein Merge erst nach Rückfrage.

## Review Focus

1. **Datenschutz:** Ein Fehlertext aus Rust enthält einen Betriebsnamen (z.B. „Name ist bereits vergeben: Bäckerei X“?). Die Aktionsliste übernimmt Fehlermeldungen, das ist laut Spec zulässig, weil die Nutzerin sie sieht. Argumente und Rückgabewerte dürfen aber nie erscheinen, auch nicht über `unhandledrejection` mit einem Objekt als Grund.
2. **Aufbewahrung an Grenzen:** Eine Sicherung ist genau A Tage alt, eine Wochengrenze fällt auf Sonntag/Montag, Sicherungen haben gleiche Minute mit Suffix, die Uhr springt (Sicherung „in der Zukunft“), es gibt nur eine einzige Sicherung. Keiner dieser Fälle darf die neueste löschen.
3. **Import-Zusammenfassung:** Gleiche Firma mit anderer Schreibweise oder Leerzeichen („Bäckerei  Muster“) und gleicher Adresse wird zusammengefasst. Gleiche Firma an anderer Adresse bleibt getrennt. Der Zeilenabgleich geht in allen Mischungen auf.
4. **Sichern, falls geändert:** Der Zähler darf nicht durch die eigene Sicherung (Schreiben von `last_secondary_at`) „geändert“ melden. Bei nur lesend oder nach verlorener Sperre wird nicht gesichert.
5. **Rückgängig nach dem Import:** Wird die Seite gewechselt und später erneut importiert, darf ein alter Rückgängig-Pfad nicht wieder auftauchen. Rückgängig verwendet nur die Sicherung des letzten Imports dieser Sitzung.

---

### Task 1: Neue Felder beim Betrieb

**Files:**
- Create: `crates/core/src/db/0003_org_numbers.sql`
- Modify:
  - `crates/core/src/db/mod.rs`
  - `crates/core/src/repo/organisations.rs` (Input, Struct, SQL, `search_text` inkl. Handelsregister-Nr)
  - `crates/core/src/export_full.rs`
  - `ui/src/lib/types.ts`
  - `ui/src/components/OrganisationFields.svelte`
  - die Beispieldaten (`crates/core/examples/beispieldaten.rs`: Handelsregister-Nr bei 3 Betrieben, Muster `CHE-000.000.00X`)
- Test: `crates/core/tests` bzw. die Tests in `organisations.rs`, der Gesamtexport-Test, `ui` OrganisationDetail-Test

**Interfaces:**
- Produces: `OrganisationInput { …, commercial_register_no: Option<String>, cost_center_no: Option<String> }`, ebenso in `Organisation`. TS: `commercialRegisterNo`/`costCenterNo` bzw. die bestehende Konvention der Felder (snake_case wie die übrigen Org-Felder in `types.ts` prüfen und übernehmen).

- [ ] **Step 1: Failing Tests**
  - `create_and_get_keeps_numbers`
  - `search_finds_commercial_register_no`: Filter-Text „CHE-123“ findet den Betrieb.
  - `migration_adds_columns_to_existing_file`: Eine Datei mit Stand 0002 öffnen, die Spalten existieren, vorhandene Betriebe haben NULL.
  - Gesamtexport hat die Spalten „Handelsregister-Nr“ und „Kostenstellen-Nr“.
- [ ] **Step 2:** `cargo test --workspace`. Erwartet: FAIL.
- [ ] **Step 3: Umsetzung**
  - Migration `ALTER TABLE organisations ADD COLUMN commercial_register_no TEXT; … cost_center_no TEXT;`
  - Repo und Export erweitern.
  - Formular: Felder **Handelsregister-Nr** und **Kostenstellen-Nr** nach „Alt-ID (alte Liste)“.
- [ ] **Step 4:** UI-Test: Das Formular zeigt beide Felder und sendet sie beim Speichern mit.
- [ ] **Step 5:** Alle Prüfungen grün. Commit `feat: Betriebe haben Felder für Handelsregister-Nr und Kostenstellen-Nr`

### Task 2: Import-Plan nach dem echten Aufbau (Core, rein)

**Files:**
- Modify: `crates/core/src/import/mapping.rs` (bei Bedarf aufteilen in `mapping.rs` = Felder/Zuordnung und `plan.rs` = Deutung/Zusammenfassen/Abgleich; Dateien unter 400 Zeilen halten), `crates/core/src/import/mod.rs`
- Test: `crates/core/tests/import_plan.rs` (neu; Excel-Dateien im Test mit `rust_xlsxwriter` erzeugen)

**Interfaces:**
- Produces:
  - `enum Field`: `LegacyId, CantonalAssociation, Company, LastName, FirstName, Addition, Street, Zip, City, Category, Email, Website, Apprenticeship, FrBaeck, FrConf, Dh, Notes, Joined, Left, Branches, Phone, Mobile, CostCenterNo, Mandate, CommercialRegisterNo, ApprenticeCount, Ignore`
  - Standard-Zuordnung genau nach Spec 5.1. `adresszusatz` und unbekannte Spalten werden `Ignore`.
  - Strukturen:
    ```rust
    pub struct PlannedPerson { pub line: usize, pub input: PersonInput, pub mandates: Vec<String> }
    pub struct PlannedOrganisation {
        pub lines: Vec<usize>, pub input: OrganisationInput, pub category: Option<String>,
        pub cantonal: Option<String>, pub flags: Vec<&'static str>, pub mandates: Vec<String>,
        pub joined: Option<NaiveDate>, pub left: Option<NaiveDate>, pub persons: Vec<PlannedPerson>,
    }
    pub struct PlannedIndividual { pub person: PlannedPerson, pub category: Option<String>,
        pub joined: Option<NaiveDate>, pub left: Option<NaiveDate> }
    pub struct SkippedRow { pub line: usize, pub reason: String }
    pub struct ImportPlan { pub data_rows: usize, pub organisations: Vec<PlannedOrganisation>,
        pub individuals: Vec<PlannedIndividual>, pub skipped: Vec<SkippedRow>, pub report: Vec<ReportEntry> }
    pub fn plan(sheet: &Sheet, mapping: &Mapping) -> ImportPlan
    pub fn reconcile(plan: &ImportPlan) -> Result<(), String> // Spec 5.7 Punkt 3
    ```
    Mandate stehen bei der Person, wenn die Zeile eine Person hat, sonst beim Betrieb bzw. beim Einzelmitglied.

- [ ] **Step 1: Failing Tests** (Kopfzeilen exakt wie Spec 5.1, inklusive dreier leerer Kopfzeilen am Ende)
  - `default_mapping_matches_real_headers`: jede Kopfzeile auf das erwartete Feld, `Adresszusatz` und leere auf `Ignore`.
  - `same_company_same_address_is_merged`: 2 Zeilen, verschiedene Personen → 1 Betrieb, `lines == [2,3]`, 2 Personen, Bericht „2 Zeilen zu einem Betrieb zusammengefasst“.
  - `merge_ignores_case_and_spaces`: „Bäckerei  Muster“ und „bäckerei muster“ an derselben Adresse werden zusammengefasst.
  - `same_company_other_address_stays_separate`
  - `deviating_contact_goes_to_person`: Zeile 3 hat eine andere Mail, ein anderes Telefon, eine andere OBC-ID → Person aus Zeile 3 bekommt Mail und Telefon, die Bemerkungen enthalten „OBC-ID: 77“. Der Betrieb behält die Werte aus Zeile 2.
  - `deviating_category_is_reported`: Bericht „Abweichende Angabe für Kategorie (erste Zeile 2 gilt)“.
  - `deviation_without_person_goes_to_org_notes`
  - `empty_values_filled_from_later_rows`
  - `row_without_company_is_individual`: mit Adresse, Mail, Kategorie, Eintritt.
  - `individual_flags_are_ignored_with_note`
  - `first_name_only_becomes_last_name_with_note`
  - `row_without_company_and_name_is_skipped`
  - `mandate_split_to_person`: „Präsident; Vorstand“ → `["Präsident","Vorstand"]` bei der Person.
  - `mandate_without_person_to_org`
  - `numbers_and_addition_land_in_org`
  - `reconcile_ok_for_mixed_plan`: Alle Fallarten gemischt, `reconcile` ergibt `Ok`.
  - `reconcile_detects_mismatch`: Plan von Hand verfälscht (eine Zeile entfernt) → `Err("Der Abgleich der Zeilen geht nicht auf …")`.
- [ ] **Step 2:** FAIL prüfen (Kompilierfehler der neuen Typen zählen als FAIL; einmal laufen lassen).
- [ ] **Step 3: Umsetzung**
  - Schlüssel fürs Zusammenfassen: `(norm(firma), norm(plz), norm(adresse))`, mit `norm` = trim, lowercase, Mehrfach-Leerzeichen zu einem.
  - Eine Reihenfolge bleibt erhalten: Betriebe in der Reihenfolge ihrer ersten Zeile.
  - Bestehende Hilfen (`as_text`, `as_date`, `as_flag`, Datumsprüfung „Eintritt nach Austritt“, „Lehrbetrieb ohne Fachrichtung“) weiterverwenden.
- [ ] **Step 4:** `cargo test -p verwaltung_core`. Erwartet: PASS.
- [ ] **Step 5:** Kein eigener Commit: Die App-Aufrufer kompilieren erst nach Task 3/4 wieder. Task 2–4 werden **gemeinsam** committet (`feat: Excel-Import versteht Firma, Personen und Einzelmitglieder der Mitgliederliste`), sobald `cargo test --workspace` grün ist.

### Task 3: Import schreiben (Core apply)

**Files:**
- Modify: `crates/core/src/import/apply.rs`
- Create: `crates/core/src/import/lookup.rs` (Abgleich ohne Gross-/Kleinschreibung)
- Test: `crates/core/tests/import_apply.rs` (neu) und das bestehende `crates/app/tests/import.rs` anpassen

**Interfaces:**
- Consumes: `ImportPlan` (Task 2).
- Produces:
  - `pub const IMPORT_FUNCTION: &str = "Kontaktperson"`
  - `pub const MANDATE_GROUP: &str = "Mandat"`
  - `pub fn lookup_or_create(conn, kind: LookupKind, name: &str) -> Result<LookupValue>` und `pub fn label_or_create(conn, group: &str, applies: AppliesTo, name: &str) -> Result<Label>`, beide ohne Gross-/Kleinschreibung (`lower(name) = lower(?)`; für Umlaute zusätzlich Rust-seitig `to_lowercase` vergleichen, weil SQLite `lower` nur ASCII kennt)
  - `pub fn existing_names(conn, kind) -> Result<Vec<String>>` als Hilfe für „neu angelegt“ (Task 4)
  - `ImportSummary { organisations, persons, individuals, report }`

- [ ] **Step 1: Failing Tests**
  - `kein_mitglied_matches_default_category`: neue Datei mit Vorgaben, Import mit Kategorie „kein Mitglied“ → 4 Kategorien, der Betrieb hat „Kein Mitglied“.
  - `umlaut_case_matches`: „MITGLIED MIT GESCHÄFT“ landet ebenfalls bei der Vorgabe.
  - `persons_get_kontaktperson`
  - `individual_gets_own_membership`
  - `mandate_labels_on_person`: Gruppe „Mandat“ mit „Präsident“ und „Vorstand“, an der Person.
  - `apply_is_all_or_nothing` (bestehendes Verhalten bleibt)
  - `apply_refuses_non_empty_db` (bestehend)
- [ ] **Step 2:** FAIL, Umsetzung, PASS.
- [ ] **Step 3:** Commit gemeinsam mit Task 2 und 4 (siehe Task 2, Step 5).

### Task 4: Import-Vorschau, Export, Sicherung davor und Rückgängig (App-Schicht)

**Files:**
- Modify: `crates/app/src/commands/import.rs` (bei Bedarf in `import/preview.rs`, `import/export.rs` aufteilen), `crates/app/src/tauri_commands/data.rs` (bzw. dort, wo die Import-Befehle registriert sind), `crates/app/src/main.rs` (neue Befehle registrieren)
- Test: `crates/app/tests/import.rs`

**Interfaces:**
- Consumes: Task 2 und 3.
- Produces (Serde camelCase):
  ```rust
  pub struct PreviewPerson { pub line: usize, pub name: String, pub function: String, pub labels: Vec<String>, pub notes: Vec<String> }
  pub struct PreviewOrganisation { pub lines: Vec<usize>, pub name: String, pub city: Option<String>, pub category: Option<String>, pub labels: Vec<String>, pub persons: Vec<PreviewPerson>, pub notes: Vec<String> }
  pub struct PreviewIndividual { pub person: PreviewPerson, pub city: Option<String>, pub category: Option<String> }
  pub struct NewEntries { pub categories: Vec<String>, pub cantonal: Vec<String>, pub functions: Vec<String>, pub labels: Vec<String> /* "Gruppe: Label" */ }
  pub struct ImportPreview { headers, mapping, fields, rows: usize, organisations: Vec<PreviewOrganisation>, merged: usize, persons: usize, individuals: Vec<PreviewIndividual>, skipped: Vec<SkippedDto>, new_entries: NewEntries, reconciled: Result-artig: pub reconcile_error: Option<String>, report, database_empty: bool }
  pub fn import_preview(s, path, mapping) -> AppResult<ImportPreview>   // ersetzt sample
  pub fn export_preview(s, path_xlsx_in, mapping, out: &Path) -> AppResult<()> // Spec 5.7.4
  pub struct ImportSummaryDto { organisations, persons, individuals, report, undo_backup: String }
  pub fn import_apply(s, path, mapping, now) -> AppResult<ImportSummaryDto>  // sichert vorher lokal, verweigert bei reconcile-Fehler
  pub fn import_undo(s, backup: &Path, now) -> AppResult<()>                // = restore_backup, nur Pfade im lokalen Sicherungsordner
  ```
  `notes` einer Vorschau-Zeile sind die Berichtseinträge dieser Zeile(n).

- [ ] **Step 1: Failing Tests**
  - `preview_groups_persons_under_org`
  - `preview_lists_new_entries_case_insensitively`: In der Datei gibt es „Kein Mitglied“, die Excel hat „kein Mitglied“ und „Ehrenmitglied“ → `categories == ["Ehrenmitglied"]`. „Kontaktperson“ steht unter `functions`, solange es die Funktion noch nicht gibt.
  - `preview_counts_add_up`: `rows == erste Zeilen + zusammengefasste + Einzelmitglieder + übersprungene`.
  - `apply_refuses_when_reconcile_fails`: Fehler über einen Test-Hook; falls zu aufwendig, nur die reine Prüffunktion aus Task 2, dann Ruling.
  - `apply_creates_backup_and_returns_path`: Die Datei existiert im lokalen Sicherungsordner.
  - `undo_restores_empty_file`: nach `import_undo` 0 Betriebe und 0 Personen.
  - `undo_refuses_foreign_path`: ein Pfad ausserhalb des Sicherungsordners → Fehler „Ungültige Sicherung“.
  - `export_preview_has_one_row_per_source_row`: per calamine zurücklesen, Spalten Zeile/Ergebnis/Betrieb/Person/Kategorie/Labels/Hinweise.
- [ ] **Step 2:** FAIL, Umsetzung, PASS. `ALL_FIELDS`, `field_key` und `parse_field` an die neuen Felder anpassen. Keys: `company`, `last_name`, `cost_center_no`, `mandate`, `commercial_register_no`, die übrigen wie bisher.
- [ ] **Step 3:** Tauri-Wrapper `export_import_preview`, `import_undo` und angepasstes `import_apply`. Alle Prüfungen grün.
- [ ] **Step 4: Commit** `feat: Import zeigt eine vollständige Vorschau, sichert vorher und lässt sich rückgängig machen`

### Task 5: Import-Oberfläche

**Files:**
- Modify: `ui/src/pages/settings/ImportWizard.svelte` (aufteilen: `ImportPreview.svelte` für Liste und Filter, `ImportSummary.svelte` für die Zusammenfassung), `ui/src/lib/api.ts`, `ui/src/lib/types.ts`, `ui/src/lib/importMapping.ts` (Feldnamen auf Deutsch für die neuen Keys)
- Test: `ui/src/pages/settings/ImportWizard.test.ts` (neu oder erweitert)

**Interfaces:**
- Consumes: DTOs aus Task 4.
- Feldnamen in der Auswahl: Firma (Betrieb), Nachname, Vorname, Zusatz, Kostenstellen-Nr, Mandat (Labels), Handelsregister-Nr, und die bisherigen.

- [ ] **Step 1: Failing Tests**
  - Die Zusammenfassung zeigt „3 Zeilen → 1 Betriebe (davon 1 aus mehreren Zeilen zusammengefasst), 2 Personen, 1 Einzelmitglieder, 0 Zeilen übersprungen“. Grammatik: Singular/Plural sauber, z.B. mit kleiner Hilfsfunktion `plural(n, 'Betrieb', 'Betriebe')`, als Unit-Test in `lib`.
  - „Neu angelegt werden“ listet die Einträge pro Liste. Ohne neue Einträge erscheint „nichts Neues“.
  - Der Filter **Nur mit Hinweisen** blendet Betriebe ohne Hinweise aus.
  - **Übernehmen** fragt mit den Zahlen nach (`confirmAction`). Bei Abbrechen wird `importApply` nicht aufgerufen.
  - Bei `reconcileError` ist **Übernehmen** gesperrt, und die Meldung wird angezeigt.
  - Nach dem Import erscheint **Import rückgängig machen**. Es fragt nach und ruft `importUndo(undoBackup)` auf. Danach steht „Import rückgängig gemacht.“, und die Daten werden neu geladen (`bumpData`).
  - **Vorschau als Excel speichern…** öffnet den Speichern-Dialog (`pickSaveFile` o.ä. aus `desktop.ts`) und ruft den Export auf.
- [ ] **Step 2:** FAIL, Umsetzung, PASS. `vitest`, `check`.
- [ ] **Step 3: Musterdatei lokal prüfen**, von Hand, ohne Commit und ohne Ausgabe von Inhalten:
  - `pnpm tauri dev` mit einer frischen Testdatei, die Musterdatei importieren.
  - Alternativ ein kurzer Rust-Testlauf mit `#[ignore]`, der den Pfad aus einer Umgebungsvariable liest und nur Zahlen ausgibt. Er darf ins Repo, weil er ohne Variable nichts tut.
  - Ins Ledger: Zeilen, Betriebe, Personen, Einzelmitglieder, übersprungene Zeilen, Art der Hinweise (ohne Namen).
- [ ] **Step 4: Commit** `feat: Import-Seite mit Zusammenfassung, Filter, Rückfrage und Rückgängig`

### Task 6: Aufbewahrung nach Alter (Core)

**Files:**
- Create: `crates/core/src/storage/retention.rs`
- Modify: `crates/core/src/storage/backup.rs` (`prune` ersetzen), `crates/core/src/storage/workspace.rs` (drei Aufrufer), `crates/core/src/repo/settings.rs` (Keys)
- Test: in `retention.rs` und `crates/core/tests/backup*.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct RetentionPolicy { pub keep_all_days: u32, pub daily_days: u32, pub weekly_weeks: u32, pub monthly_months: u32, pub interval_minutes: u32 }
  impl Default for RetentionPolicy // 2, 14, 8, 12, 60
  impl RetentionPolicy { pub fn validate(&self) -> Result<(), String> } // Grenzen aus Global Constraints, deutsche Meldungen
  pub fn load_policy(conn) -> RetentionPolicy   // fehlende Tabelle/Werte/Unlesbares → Default
  pub fn save_policy(conn, &RetentionPolicy) -> Result<()> // validiert
  pub fn to_delete(files: &[BackupFile], now: NaiveDateTime, policy: &RetentionPolicy) -> Vec<PathBuf> // rein
  pub fn apply_retention(dir, stem, now, policy) -> Result<()>
  ```
  Settings-Keys: `backup.keep_all_days`, `backup.daily_days`, `backup.weekly_weeks`, `backup.monthly_months`, `backup.interval_minutes`.

- [ ] **Step 1: Failing Tests** für `to_delete` (Tabellentests mit festem `now`)
  - Alle jünger als A Tage bleiben.
  - Von mehreren am selben Tag (Alter zwischen A und B) bleibt die neueste.
  - Pro ISO-Woche bleibt die neueste, auch über die Grenze Sonntag 23:59 / Montag 00:00.
  - Pro Monat bleibt die neueste. Älter als D Monate wird gelöscht.
  - **Die neueste bleibt immer**, auch wenn sie älter als D Monate ist.
  - Eine Sicherung in der Zukunft (Uhrsprung) bleibt.
  - Genau A Tage alt fällt in die Tagesstufe (Grenze festlegen und testen).
  - D = 0 löscht alles nach der Wochenstufe (ausser der neuesten).
  - `validate`: B < A ergibt Fehler, Intervall 10 ergibt Fehler, 0 ist ok.
  - `load_policy` ohne `settings`-Tabelle ergibt Default.
- [ ] **Step 2:** FAIL, Umsetzung, PASS.
- [ ] **Step 3:** `BACKUP_KEEP` entfernen. Die drei Aufrufer nutzen `apply_retention` mit `load_policy` (beim Öffnen vor der Migration → Default, falls nicht lesbar). Bestehende Backup-Tests anpassen, die „20“ voraussetzen.
- [ ] **Step 4: Commit** `feat: Sicherungen werden nach Alter ausgedünnt (einstellbar)`

### Task 7: Sichern während der Arbeit, Fehler am zweiten Ort, Übernehmen

**Files:**
- Modify:
  - `crates/core/src/storage/workspace.rs` (Zähler, `backup_if_changed`)
  - `crates/app/src/session.rs` (Feld `secondary_error`, `take_over` sichert am zweiten Ort)
  - `crates/app/src/dto.rs` (`BackupStatusDto.secondary_error: Option<String>`)
  - `crates/app/src/commands/backup.rs` (`backup_if_changed`, `get_retention`, `set_retention`)
  - Tauri-Wrapper und `main.rs`
  - `ui/src/App.svelte` (Zeitgeber)
  - `ui/src/lib/session.ts` (`noticesFor`: neue Meldung)
  - `ui/src/components/NoticeBar.svelte`
  - `ui/src/pages/settings/Backup.svelte` (Bereich Aufbewahrung, ggf. eigene Komponente `RetentionSettings.svelte`)
- Test: `crates/app/tests/backup.rs`, `ui/src/lib/session.test.ts`, `ui/src/pages/settings/Backup.test.ts`, `ui/src/components/NoticeBar.test.ts`

**Interfaces:**
- Produces:
  - `Workspace::backup_if_changed(&mut self, now) -> Result<Option<BackupOutcome>>`. Merkt `total_changes` **nach** der Sicherung; bei nur lesend `Ok(None)`.
  - `Session::secondary_error() -> Option<String>`, gesetzt bei `Unavailable` aus `backup_now`/`backup_if_changed`/`take_over`, gelöscht bei `Done`.
  - Befehle `backup_if_changed() -> Option<BackupNowDto>`, `get_retention() -> RetentionPolicy`, `set_retention(policy)`.
  - TS: `Notice.kind` erweitert um `'secondaryError'`, Text „Zweiter Sicherungsort nicht erreichbar – letzte Sicherung dort am <TT.MM.JJJJ HH:MM | noch nie>“, Knopf **Jetzt sichern**.

- [ ] **Step 1: Failing Tests (Rust)**
  - `no_backup_without_change`: zweimal aufrufen → beim zweiten `None`.
  - `backup_after_change`: Betrieb anlegen → `Some`, danach ohne Änderung `None`, **auch wenn ein zweiter Ort eingerichtet ist**. Das prüft, dass `last_secondary_at` nicht zählt.
  - `read_only_never_backs_up`
  - `secondary_error_set_and_cleared`: zweiter Ort zeigt auf einen gelöschten Ordner → Fehler gesetzt; Ordner wieder anlegen und `backup_now` → Fehler weg.
  - `take_over_backs_up_secondary`
  - `set_retention_validates_and_persists`, `set_retention_refused_read_only`
- [ ] **Step 2: Failing Tests (UI)**
  - `noticesFor` mit `secondaryError` liefert die Meldung.
  - NoticeBar: **Jetzt sichern** ruft `backupNow` auf.
  - Backup-Seite: Felder vorbelegt mit `getRetention`. Ungültige Eingabe zeigt die Meldung aus Rust. **Standard wiederherstellen** setzt 2/14/8/12/60. Bei nur lesend gesperrt.
  - App-Zeitgeber: Mit Fake-Timern ruft er nach `interval_minutes` `backupIfChanged` auf, bei 0 nie. Danach ruft er `refreshSession()` auf, damit die Leiste den Fehler zeigt. Die Logik dafür kommt in `ui/src/lib/autosave.ts`, damit sie testbar ist (`startAutoBackup(getInterval, deps) -> stop`).
- [ ] **Step 3:** Umsetzung, alles grün. Commit `feat: Sicherung während der Arbeit und sofortiger Hinweis, wenn der zweite Sicherungsort fehlt`

### Task 8: Programm-Protokoll

**Files:**
- Modify:
  - `Cargo.toml` (Workspace) bzw. `crates/core/Cargo.toml` (`log`), `crates/app/Cargo.toml` (`log`, `tauri-plugin-log = "2"`, `os_info`)
  - `crates/app/src/main.rs` (Plugin: Ziel LogDir + Stdout, `max_file_size` 1 MB, `RotationStrategy::KeepOne`, Level Info)
  - `crates/app/capabilities/default.json` nur falls nötig (Protokoll aus dem Frontend wird nicht gebraucht)
  - Protokollzeilen in `workspace.rs` (Sicherung/Übernehmen/Sperre verloren), `restore.rs`, `import/apply.rs`, `tauri_commands` (Fehler zurück an die Oberfläche: Befehlsname + Meldung, zentral in einer Hilfsfunktion)
- Create: `crates/app/src/commands/diagnostics.rs` mit `log_tail(log_dir: &Path) -> AppResult<Vec<String>>` (Regel aus Spec 7.4) und `system_info() -> SystemInfoDto { os: String, arch: String, version: String }`
- Test: `crates/app/tests/diagnostics.rs`

**Interfaces:**
- Produces: Tauri-Befehle `diagnostics_log_tail() -> Vec<String>` (liest `app.path().app_log_dir()`) und `diagnostics_system() -> SystemInfoDto`.

- [ ] **Step 1: Failing Tests**
  - `log_tail_prefers_warn_error`: Temp-Datei mit 100 Zeilen gemischt → höchstens 40 WARN/ERROR plus die letzten 10, ohne Doppelte, in Reihenfolge.
  - `log_tail_without_file_is_empty`
  - `logs_contain_no_org_name`: Ein Test-Logger sammelt Zeilen (`log::set_boxed_logger` einmal pro Testprozess, eigene Testdatei). Session anlegen, Betrieb „Bäckerei Geheim AG“ speichern, Sicherung, Import einer Mini-Datei → keine Zeile enthält „Geheim“.
  - `system_info_has_version`
- [ ] **Step 2:** Umsetzung, alles grün. Commit `feat: Das Programm führt ein Protokoll für Problemberichte`

### Task 9: Problem melden

**Files:**
- Create:
  - `ui/src/lib/diagnostics.ts` (Ringpuffer, Aufzeichnen, Bericht bauen)
  - `ui/src/lib/diagnostics.test.ts`
  - `ui/src/components/ReportDialog.svelte`
  - `ui/src/components/ReportDialog.test.ts`
- Modify:
  - `ui/src/lib/api.ts` (`call` zeichnet auf; neue Befehle)
  - `ui/src/lib/router.ts` (`navigate`/`replace`/`back` zeichnen den Seitennamen auf)
  - `ui/src/main.ts` (`error`/`unhandledrejection`)
  - `ui/src/components/Sidebar.svelte` (Eintrag **Problem melden** unter „Hilfe“)
  - `ui/src/components/HelpPanel.svelte` (Knopf im Fuss)
  - `ui/src/App.svelte` (Dialog einhängen)
  - `ui/src/lib/help.ts` nur falls nötig

**Interfaces:**
- Produces:
  ```ts
  export type ActionEntry = { at: string; kind: 'page' | 'command' | 'error'; name: string; ok?: boolean; ms?: number; message?: string };
  export function recordPage(route: Route): void;            // Name aus PAGE_NAMES, keine IDs
  export function recordCommand(name: string, ok: boolean, ms: number, message?: string): void;
  export function recordError(message: string): void;          // nur String; Objekte → Klassenname
  export function recentActions(limit = 30): ActionEntry[];
  export function buildReport(input: { description: string; now: Date; system: SystemInfo; route: Route; session: SessionInfo | null; retention: RetentionPolicy | null; actions: ActionEntry[]; log: string[] }): string; // Format Spec 7.2
  export const reportOpen: Writable<boolean>;
  ```
  Speicher-Key `mv.aktionen`, max. 50, alle Zugriffe in try/catch.

- [ ] **Step 1: Failing Tests (`diagnostics.test.ts`)**
  - Der Ringpuffer hält höchstens 50 Einträge, die ältesten fallen weg.
  - Ein werfendes `localStorage` führt nicht zu einem Fehler; der Puffer funktioniert dann nur im Speicher.
  - `recordPage({page:'organisation', id: 42})` ergibt „Betrieb (Detail)“, ohne „42“.
  - **Datenschutz:** Über das echte `call` (mit gemocktem `invoke`) `saveOrganisation({name:'Bäckerei Geheim', email:'x@y.ch'})` und `searchAddresses({text:'Geheim'})` aufrufen. Danach enthält `buildReport(…)` weder „Geheim“ noch „x@y.ch“, aber „save_organisation – OK“.
  - `unhandledrejection` mit dem Objekt `{name:'Geheim'}` → Eintrag ohne „Geheim“.
  - `buildReport` enthält alle Abschnitte aus Spec 7.2. Ohne Beschreibung steht „(keine Angabe)“. Ohne Datei steht „keine Datei“. Bei einem Fehler am zweiten Ort steht „Fehler: …“.
- [ ] **Step 2: Failing Tests (`ReportDialog.test.ts`)**
  - Öffnen über `reportOpen` zeigt das Feld **Was ist passiert?** und den Bericht.
  - Tippen aktualisiert den Bericht.
  - **Bericht kopieren** ruft `copyText` mit dem Bericht auf und zeigt „Kopiert.“.
  - Esc schliesst den Dialog. Ist die Hilfe offen, bleibt sie offen; das deckt `dialog[open]` in `help.ts` ab, der Dialog ist also ein `<dialog open>`.
  - Der Eintrag in der Sidebar und der Knopf in der Hilfe-Leiste öffnen den Dialog.
- [ ] **Step 3:** Umsetzung, alles grün. Commit `feat: Problem melden erstellt einen Bericht zum Kopieren`

### Task 10: Handbuch, Bildschirmfotos, Abschluss

**Files:**
- Modify:
  - `docs/handbuch/08-sicherung-wiederherstellen.md`, `16-datensicherung.md`, `02b-einrichten.md`, `17-excel-import.md`, `11-betrieb.md`, `20-haeufige-fragen.md`, `inhalt.json`
  - Kapitel 01/03/12, falls „Inhaber“ als Import-Funktion vorkommt
  - `e2e/specs/90-bildschirmfotos.e2e.mjs` (Motive: Datensicherung mit Aufbewahrung, Dialog Problem melden)
  - `docs/handbuch/bilder/*`
- Create: `docs/handbuch/21-problem-melden.md`

- [ ] **Step 1:** Kapitel nach Spec §8 schreiben:
  - die beiden Tabellen und das Beispiel („Ein Fehler vom Montag wird erst am Freitag bemerkt …“)
  - neue Felder, Import-Ablauf mit Testdatei-Empfehlung, Vorschau, Rückgängig
  - Problem melden (Windows + Umschalt + S)

  Knopfnamen gegen den Quelltext prüfen.
- [ ] **Step 2:** `node --test 'scripts/*.test.mjs'`, `vitest` (Handbuch-Test: Anzahl Kapitel anpassen), `check`.
- [ ] **Step 3:** Push des Branches. Workflow **Bildschirmfotos** auf dem Branch auslösen.
  - Er ist noch `workflow_dispatch`-only und liegt auf main, darum geht `gh workflow run bildschirmfotos.yml --ref feat/erster-einsatz`.
  - Danach `node scripts/bildschirmfotos.mjs`, die Bilder ansehen und committen: `docs: Bildschirmfotos aktualisieren`.
- [ ] **Step 4: Commit** `docs: Handbuch zu Import, Sicherungsstrategie und Problem melden`. CI auf dem Branch muss grün sein.

# Oberfläche (Tauri 2 + Svelte) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Aus dem fertigen Datenkern `verwaltung_core` ein bedienbares Desktop-Programm machen, das die Excel-Mitgliederliste im Alltag ersetzt.

**Architecture:** Neue Crate `crates/app` (`verwaltung_app`) ist die Tauri-2-Schicht. Ihre Logik liegt in
einfachen Rust-Funktionen auf einer `Session` (ohne Tauri testbar). Die `#[tauri::command]`-Hüllen in
`src/tauri_commands.rs` sind reine Weiterleitung. Frontend `ui/` (Vite + Svelte 5 + TypeScript) ruft die
Befehle über `invoke` auf. Es enthält keine Fachregeln, nur Anzeige, Formular-Umwandlung und Navigation.

**Tech Stack:** Rust 2021, tauri 2, tauri-plugin-dialog 2, tauri-plugin-opener 2, tauri-plugin-clipboard-manager 2,
gethostname 0.5, serde/serde_json, chrono, tempfile (dev). Node 22, pnpm, svelte 5, vite 6, typescript 5,
vitest 3, @testing-library/svelte 5, jsdom, @tauri-apps/api 2 + passende Plugin-Pakete.

**Spec:** `docs/superpowers/specs/2026-11-24-oberflaeche-design.md` (freigegeben). Sie ergänzt
`docs/superpowers/specs/2026-11-24-mitgliederverwaltung-design.md` (§6 Kernabläufe, §8 Backup-Konzept).

## Global Constraints

- Oberfläche vollständig Deutsch (Schweiz): „ss" statt „ß", Datum **TT.MM.JJJJ** in Anzeige und Eingabe, CHF.
- Betriebskosten 0: keine Netzwerkzugriffe zur Laufzeit, keine Telemetrie, keine Online-Schriften/CDNs.
- Keine Fachlogik im Frontend. Regeln (Validierung, Filter, Mail-Liste, Import) kommen aus `verwaltung_core`.
- Vor **jedem** Schreibbefehl `Workspace::check_lock()`. Nur bei `AccessMode::ReadWrite` schreiben,
  sonst Fehler `Die Datei ist nur lesend geöffnet. Änderungen sind nicht möglich.`
- Wiederherstellen nur über `Workspace::restore(&mut self, …)` auf der bestehenden Sitzung.
- Import-Zuordnung positionsbasiert: Eintrag i ↔ Spalte i, Standard aus `Mapping::default_for`.
- Beim Beenden `Workspace::close(now)`.
- Fehlertexte aus `CoreError` unverändert (`to_string()`) an die Oberfläche geben.
- Daten zwischen Rust und TS: Datum als `"YYYY-MM-DD"` (chrono-Standard), Zeitpunkte als RFC 3339.
  Umwandlung nach TT.MM.JJJJ nur in `ui/src/lib/dates.ts`.
- Dateien < 400 Zeilen (max. 800), Funktionen < 50 Zeilen, unveränderliche Datenflüsse (neue Objekte statt Mutation im TS).
- TDD: Test zuerst, rot sehen, dann grün. Coverage ≥ 80 % für `verwaltung_app` (ohne `tauri_commands.rs`/`main.rs`)
  und für `ui/src/lib`.
- Commits `<type>: <description>`, Abschluss `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Rust-Werkzeuge ggf. erst mit `source ~/.cargo/env` in den PATH holen.

## Review Focus

1. **Sperre geht zwischen zwei Klicks verloren** (Vertretung übernimmt): Der nächste Schreibversuch wird mit
   deutschem Hinweis abgelehnt, und die Sitzungsinfo meldet danach „nur lesen". Test in Task 2.
2. **Leere/Leerzeichen-Felder und ungültige Datumseingaben** („31.02.2024", „1.2.24", „2024"): Leere Felder
   werden zu `null`, nicht `""`. Ungültige Daten werden als Feldfehler gemeldet und nicht gespeichert.
   Tests in Task 7 (`forms.ts`, `dates.ts`).
3. **Ergebnisliste leer oder ganz ohne Mail; mailto zu lang**: Die Buttons sind deaktiviert bzw. zeigen
   „Zu viele Adressen für das Mailprogramm – bitte kopieren". Tests in Task 3 (Rust) und Task 8 (Vitest).
4. **Import in nicht-leere Datenbank**: Das zeigt die Vorschau schon vor dem Übernehmen
   (`database_empty: false`), und das Übernehmen liefert den Core-Text. Test in Task 6.
5. **Wiederherstellen/Import bei nur-lesend und Neuladen danach**: Beides wird abgelehnt, solange nur
   gelesen werden darf. Nach erfolgreichem Wiederherstellen laden alle Ansichten neu
   (Store-Version erhöht). Tests in Task 6 und Task 11.

---

## File Structure

```
Cargo.toml                       workspace: members += "crates/app"
crates/app/
  Cargo.toml  build.rs  tauri.conf.json  capabilities/default.json  icons/ (tauri icon)
  src/main.rs                    Tauri-Builder, Plugins, State, Close-Hook
  src/lib.rs                     pub mod config, dto, error, identity, session, commands
  src/error.rs                   AppError (serialisiert als String)
  src/identity.rs                Identity aus Umgebung
  src/config.rs                  zuletzt geöffnete Datei merken
  src/session.rs                 Session: open/create/take_over/info/read/write/close
  src/dto.rs                     serialisierbare Abbilder nicht-serde Core-Typen
  src/commands/{mod,addresses,entities,catalog,teams,backup,import}.rs
  src/tauri_commands.rs          #[tauri::command]-Hüllen (keine Logik)
  tests/{common/mod.rs, config.rs, session.rs, addresses.rs, entities.rs, catalog.rs, teams.rs, backup.rs, import.rs}
ui/
  package.json  vite.config.ts  tsconfig.json  index.html  svelte.config.js
  src/main.ts  src/App.svelte  src/app.css
  src/lib/{types.ts, api.ts, dates.ts, forms.ts, filters.ts, router.ts, session.ts, contact.ts, importMapping.ts}
  src/lib/*.test.ts
  src/components/{Sidebar, NoticeBar, DateInput, ConfirmDialog, LabelPicker}.svelte
  src/pages/{Start, Addresses, OrganisationDetail, PersonDetail, Teams, Groups, Settings}.svelte
  src/pages/settings/{Catalog, Backup, ImportWizard, FullExport}.svelte
```

---

### Task 1: Gerüst Tauri-App + Svelte-Frontend, zuletzt geöffnete Datei

**Files:**
- Modify: `Cargo.toml` (members)
- Create: `crates/app/**` (Cargo.toml, build.rs, tauri.conf.json, capabilities, icons, src/main.rs, src/lib.rs, src/error.rs, src/identity.rs, src/config.rs)
- Create: `ui/**` (Vite/Svelte/TS/Vitest-Gerüst, `src/App.svelte` mit Platzhalter „Mitgliederverwaltung")
- Test: `crates/app/tests/config.rs`, `ui/src/lib/smoke.test.ts`
- Modify: `.gitignore` (+ `/ui/node_modules`, `/ui/dist`, `/crates/app/gen`)

**Interfaces:**
- Produces:
  - `pub struct AppError(pub String)` mit `Serialize` (als String), `From<CoreError>`, `From<std::io::Error>`,
    `pub type AppResult<T> = Result<T, AppError>`.
  - `pub fn identity_from_env() -> Identity` (user: `USERNAME` oder `USER`, sonst `"unbekannt"`; host: `gethostname`).
  - `pub fn load_last_path(config_dir: &Path) -> Option<PathBuf>` und
    `pub fn save_last_path(config_dir: &Path, db: &Path) -> AppResult<()>` (Datei `config_dir/zuletzt.json`, `{"db": "<pfad>"}`).

- [ ] **Step 1: Write failing tests** `crates/app/tests/config.rs`:
  - `last_path_roundtrip`: Nach `save_last_path(tmp, "/x/verband.sqlite")` liefert `load_last_path(tmp)` diesen Pfad.
  - `missing_config_is_none`: Ein leeres Verzeichnis liefert `None`.
  - `corrupt_config_is_none`: Steht in `zuletzt.json` der Inhalt `"{kaputt"`, kommt `None` (kein Panic).
  - `save_creates_dir`: Existiert `config_dir` noch nicht, wird es angelegt.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test config`. Expected: Kompilierfehler/FAIL (Modul fehlt).
- [ ] **Step 3:** Gerüst anlegen. Frontend z.B. mit `pnpm create vite ui --template svelte-ts`, dann Vitest +
  jsdom + @testing-library/svelte ergänzen. Tauri-Crate von Hand gemäss File Structure anlegen.
  Fenster-Titel „Mitgliederverwaltung", Grösse 1280×800, min. 1024×680. Danach `config.rs`, `identity.rs`, `error.rs` umsetzen.
  Die `tauri.conf.json` verweist mit `build.devUrl` auf `http://localhost:5173` und mit
  `build.frontendDist` auf `../../ui/dist`. `beforeDevCommand`/`beforeBuildCommand` starten pnpm in `ui/`.
  Die Capabilities erlauben nur dialog (open/save), opener (open-url für `mailto:`) und clipboard (write-text).
  Tauri-CLI: `cargo install tauri-cli --version "^2" --locked`.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app` und `pnpm --dir ui test -- --run`.
  Expected: 4 Rust-Tests grün, 1 Smoke-Test grün. `pnpm --dir ui build` erzeugt `ui/dist`.
- [ ] **Step 5:** Run `cd crates/app && cargo tauri dev` (manuell, kurz). Expected: Fenster öffnet sich mit dem Platzhalter. Danach schliessen.
- [ ] **Step 6: Commit** `feat: scaffold tauri app and svelte frontend`

---

### Task 2: Session – öffnen, neu anlegen, übernehmen, Schreibschutz, schliessen

**Files:**
- Create: `crates/app/src/session.rs`, `crates/app/src/dto.rs` (Session-Teil)
- Test: `crates/app/tests/session.rs`, `crates/app/tests/common/mod.rs`

**Interfaces:**
- Consumes: `Workspace::{open, open_taking_over, check_lock, is_read_write, backup_now, close, path}`, `backup::status`, `AppError`.
- Produces:
  - `pub struct Session { ws: Option<Workspace> }` mit `Session::default()`.
  - `pub fn open(&mut self, path: &Path, who: &Identity) -> AppResult<SessionInfo>`.
    Eine bereits offene Sitzung wird vorher mit `close` beendet.
  - `pub fn create(&mut self, path: &Path, who: &Identity) -> AppResult<SessionInfo>`.
    Existiert die Datei schon, kommt der Fehler `Die Datei existiert bereits. Bitte „Öffnen" verwenden.`
  - `pub fn take_over(&mut self, who: &Identity) -> AppResult<SessionInfo>` (nur bei offener Sitzung, sonst `Keine Datei geöffnet.`).
  - `pub fn info(&mut self, now: DateTime<Utc>) -> AppResult<SessionInfo>` (ruft `check_lock`).
  - `pub fn read<T>(&self, f: impl FnOnce(&Connection) -> CoreResult<T>) -> AppResult<T>` (`CoreResult` = `verwaltung_core::error::Result`).
  - `pub fn write<T>(&mut self, f: impl FnOnce(&Connection) -> CoreResult<T>) -> AppResult<T>` (check_lock + ReadWrite, sonst Text aus Global Constraints).
  - `pub fn write_mut<T>(&mut self, f: impl FnOnce(&mut Connection) -> CoreResult<T>) -> AppResult<T>` (für Import).
  - `pub fn workspace_mut(&mut self) -> AppResult<&mut Workspace>` (für restore/backup).
  - `pub fn close(&mut self, now: DateTime<Local>) -> AppResult<Option<SecondaryResultDto>>`
    (`None`, wenn nichts offen war).
  - DTOs (`Serialize`, `#[serde(rename_all = "camelCase")]`):
    - `SessionInfo { path: String, readOnly: bool, heldBy: Option<LockDto>, backup: BackupStatusDto }`
    - `LockDto { user, host, since: String (RFC 3339) }`
    - `BackupStatusDto { secondaryDir: Option<String>, lastSecondary: Option<String>, reminderDue: bool }`
    - `SecondaryResultDto` (Tag-Enum `notConfigured | done{path} | unavailable{reason}`)

- [ ] **Step 1: Write failing tests** `tests/session.rs`. `common::fresh_session(tmp)` erzeugt eine neue Datei
  als Identität „anna"/„pc1".
  - `create_then_write_and_read`: Eine mit `write` angelegte Organisation liefert `read` wieder zurück.
  - `create_refuses_existing_file`: Der Fehlertext enthält „existiert bereits".
  - `second_session_is_read_only_and_refuses_writes`: Die zweite Session (Identität „ben") öffnet dieselbe Datei
    und bekommt `info.readOnly == true` mit `heldBy.user == "anna"`. Ihr `write` liefert genau
    den Nur-lesend-Text.
  - `lost_lock_refuses_next_write` (Review Focus 1): Anna öffnet, Ben ruft `take_over`, danach liefert
    Annas `write` einen Fehler und ihr `info` meldet `readOnly == true`.
  - `take_over_makes_writable`: Nach Bens `take_over` gelingt Bens `write`.
  - `calls_without_open_file_fail`: `read` auf `Session::default()` liefert `Keine Datei geöffnet.`
  - `close_without_secondary_reports_not_configured`.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test session`. Expected: FAIL (Modul fehlt).
- [ ] **Step 3:** `session.rs` + DTOs umsetzen.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app`. Expected: alle grün.
- [ ] **Step 5: Commit** `feat: add app session with lock-aware writes`

---

### Task 3: Adressen – Suche, Mail-Liste, mailto, Listen-Export, gespeicherte Gruppen, Filter-Optionen

**Files:**
- Create: `crates/app/src/commands/mod.rs`, `crates/app/src/commands/addresses.rs`
- Test: `crates/app/tests/addresses.rs`

**Interfaces:**
- Consumes: `query::filter::{search, Filter, ContactRow}`, `contact::{email_list, mailto_bcc}`,
  `export::{contact_table, write_csv, write_xlsx}`, `query::saved_groups::*`, `repo::{lookups, labels, teams}`.
- Produces (alle `pub fn …(session: &Session | &mut Session, …) -> AppResult<…>`):
  - `search_addresses(s, filter: &Filter, today: NaiveDate) -> SearchResult`, wobei
    `SearchResult { rows: Vec<ContactRow>, emailCount: usize, missingEmail: Vec<ContactRow>, joinedEmails: String, mailto: Option<String> }`.
  - `export_addresses(s, filter, today, path: &Path, format: ExportFormat)`. `ExportFormat` ist
    `Csv | Xlsx` (serde `"csv"|"xlsx"`), Blattname „Adressen".
  - `list_groups(s) -> Vec<GroupDto{id,name}>`, `save_group(s, name, filter) -> i64`, `load_group(s, id) -> Filter`, `delete_group(s, id)`.
  - `filter_options(s) -> FilterOptions { labelGroups: Vec<LabelGroupWithLabels{group: LabelGroup, labels: Vec<Label>}>, teams: Vec<Team>, functions, orgTypes, categories, cantonal: Vec<LookupValue> }`.

- [ ] **Step 1: Write failing tests** `tests/addresses.rs`:
  - `search_returns_rows_and_email_summary`: 2 Orgs mit Mail und 1 Person ohne Mail ergeben `rows.len()==3`,
    `emailCount==2`, `missingEmail.len()==1` und `joinedEmails` mit `"; "`.
  - `empty_result_has_no_mailto` (Review Focus 3): `rows` ist leer, `mailto == None`, `joinedEmails == ""`.
  - `too_many_addresses_no_mailto`: Bei 200 Orgs mit langen Adressen ist `mailto == None` und `emailCount == 200`.
  - `export_csv_and_xlsx_write_files`: Beide Dateien existieren, die CSV beginnt mit BOM und Kopfzeile `Typ;Name;`.
  - `groups_roundtrip`: Nach `save_group("Prüfungsexperten", f)` liefern `list_groups` und `load_group` den Filter `f`.
    `delete_group` entfernt ihn.
  - `save_group_is_refused_read_only`.
  - `filter_options_lists_labels_by_group`.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test addresses`. Expected: FAIL.
- [ ] **Step 3:** `commands/addresses.rs` umsetzen. Lesen über `Session::read`, Schreiben (Gruppen) über `Session::write`.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app`. Expected: alle grün.
- [ ] **Step 5: Commit** `feat: add address search, contact and group commands`

---

### Task 4: Betriebe und Personen – Detail, Bearbeiten, Verknüpfungen, Mitgliedschaft, Labels

**Files:**
- Create: `crates/app/src/commands/entities.rs`
- Test: `crates/app/tests/entities.rs`

**Interfaces:**
- Consumes: `repo::{organisations, persons, affiliations, memberships, assignments, teams, labels, lookups}`, `MemberRef`.
- Produces:
  - `organisation_detail(s, id, include_past: bool, today) -> OrganisationDetail { organisation, people: Vec<AffiliationView>, memberships: Vec<Membership>, labels: Vec<AssignedLabel> }`
  - `person_detail(s, id, include_past, today) -> PersonDetail { person, organisations: Vec<AffiliationView>, memberships, labels: Vec<AssignedLabel>, teams: Vec<TeamRole{team: Team, role: Option<String>}> }`
  - `AffiliationView { affiliation: Affiliation, personName: String, organisationName: String, functionName: String, current: bool }`
  - `AssignedLabel { assignment: Assignment, groupName: String, labelName: String }`
  - `create_organisation / update_organisation(s, [id,] &OrganisationInput) -> Organisation`, `delete_organisation(s, id)`; analog für Person mit `PersonInput`.
  - `create_affiliation / update_affiliation(s, [id,] &AffiliationInput) -> Affiliation`, `delete_affiliation(s, id)`.
  - `create_membership / update_membership(s, [id,] &MembershipInput) -> Membership`, `delete_membership(s, id)`.
  - `assign_label(s, &AssignmentInput) -> Assignment`, `unassign_label(s, id)`.
  - `list_people(s) -> Vec<PersonOption{id, name}>`, `list_organisations(s) -> Vec<OrgOption{id, name}>` (für Auswahlfelder, nach Name sortiert).

- [ ] **Step 1: Write failing tests** `tests/entities.rs`:
  - `organisation_detail_shows_people_with_function_names`.
  - `past_affiliations_hidden_unless_requested`: Ein Eintrag mit `valid_to` gestern fehlt bei `include_past=false`
    und ist bei `true` mit `current=false` dabei.
  - `person_detail_lists_teams_and_labels`.
  - `invalid_affiliation_dates_return_core_text`: `valid_to < valid_from` liefert genau „Bis-Datum liegt vor Von-Datum".
  - `label_for_wrong_target_is_refused`: Der Fehler ist „Label gilt nicht für diesen Eintragstyp".
  - `updates_refused_read_only`.
  - `option_lists_sorted_by_name`.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test entities`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen. Ist `entities.rs` über 400 Zeilen, aufteilen in `entities/{detail,write}.rs`.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app`. Expected: alle grün.
- [ ] **Step 5: Commit** `feat: add organisation and person commands`

---

### Task 5: Stammlisten, Labels und Teams verwalten

**Files:**
- Create: `crates/app/src/commands/catalog.rs`, `crates/app/src/commands/teams.rs`
- Test: `crates/app/tests/catalog.rs`, `crates/app/tests/teams.rs`

**Interfaces:**
- Consumes: `repo::{lookups, labels, teams}`, `LookupKind`, `AppliesTo`.
- Produces:
  - `list_lookups(s, kind: LookupKind) -> Vec<LookupUsage{value: LookupValue, usage: i64}>`, `create_lookup(s, kind, name)`, `rename_lookup(s, id, name)`, `delete_lookup(s, id, confirm: bool)`.
  - `list_label_tree(s) -> Vec<LabelGroupTree{group, labels: Vec<LabelUsage{label, usage}>}>`, `create_label_group(s, name, applies_to)`, `create_label(s, group_id, name)`, `rename_label(s, id, name)`, `delete_label(s, id, confirm)`, `delete_label_group(s, id, confirm)`.
  - `list_teams(s) -> Vec<TeamSummary{team, memberCount}>`, `team_members(s, id) -> Vec<TeamMemberView{personId, name, role}>`, `create_team(s, name, description)`, `update_team(s, id, name, description)`, `delete_team(s, id)`, `add_team_member(s, team_id, person_id, role)`, `remove_team_member(s, team_id, person_id)`.

- [ ] **Step 1: Write failing tests:**
  - `catalog.rs::delete_used_lookup_needs_confirm`: Ohne confirm kommt ein Fehler mit „noch 1-mal verwendet",
    mit confirm wird gelöscht.
  - `catalog.rs::label_tree_counts_usage`.
  - `catalog.rs::rename_to_empty_is_refused` (Core-Validierungstext).
  - `teams.rs::members_show_names_and_roles`; `teams.rs::add_member_twice_updates_role`; `teams.rs::summary_counts_members`.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test catalog --test teams`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app`. Expected: alle grün.
- [ ] **Step 5: Commit** `feat: add catalog and team commands`

---

### Task 6: Datensicherung, Wiederherstellen, Gesamtexport, Excel-Import

**Files:**
- Create: `crates/app/src/commands/backup.rs`, `crates/app/src/commands/import.rs`, DTOs in `dto.rs` (oder `dto/import.rs`)
- Test: `crates/app/tests/backup.rs`, `crates/app/tests/import.rs`

**Interfaces:**
- Consumes: `Workspace::{backup_now, restore}`, `backup::{list, local_backup_dir, db_stem, status}`, `repo::settings`, `export_full::full_export`, `import::{sheet::read_first_sheet, mapping::{Mapping, Field, plan}, apply::apply}`.
- Produces:
  - `backup_overview(s, now) -> BackupOverview { status: BackupStatusDto, localDir: String, files: Vec<BackupFileDto{path, created: String "TT.MM.JJJJ HH:MM"}> }`
  - `set_secondary_dir(s, dir: Option<&Path>)`. `None` entfernt den Ort. Ein Ordner, der nicht existiert, ergibt
    `Der Ordner existiert nicht.`
  - `backup_now(s, now) -> BackupNowDto { local: String, secondary: SecondaryResultDto }`
  - `restore_backup(s, backup: &Path, now)` (über `workspace_mut`, nur bei ReadWrite)
  - `export_all(s, path)`
  - `field_key(Field) -> &'static str` / `parse_field(&str) -> Option<Field>`. Die Schlüssel sind snake_case der
    Enum-Namen, z.B. `"legacy_id"`, `"fr_baeck"`, `"ignore"`.
  - `import_preview(s, path: &Path, mapping: Option<Vec<String>>) -> ImportPreview { headers: Vec<String>, mapping: Vec<String>, fields: Vec<String>, rows: usize, organisations: usize, persons: usize, sample: Vec<PreviewRow>, report: Vec<ReportEntryDto>, databaseEmpty: bool }`.
    Bei `mapping: None` gilt `Mapping::default_for(headers)`. Ein mitgegebenes Mapping muss so viele Einträge haben
    wie Spalten, sonst `Die Zuordnung passt nicht zur Datei.` `sample` enthält die ersten 20 Zeilen
    (Zeile, Betrieb, Person, Ort, Kategorie). `fields` listet alle gültigen Schlüssel.
  - `import_apply(s, path, mapping: Vec<String>) -> ImportSummaryDto { organisations, persons, report }` (plant neu und übernimmt dann, über `write_mut`).

- [ ] **Step 1: Write failing tests.** Testdateien werden mit `rust_xlsxwriter` im Test erzeugt; ein Helfer kommt in `tests/common`.
  - `backup.rs::backup_now_creates_local_file_and_lists_it`.
  - `backup.rs::secondary_dir_must_exist`.
  - `backup.rs::restore_brings_back_old_state`: Org A, sichern, Org B anlegen, wiederherstellen. Danach gibt es
    nur A, und `info.readOnly == false`.
  - `backup.rs::restore_refused_read_only` (Review Focus 5).
  - `backup.rs::export_all_writes_xlsx`.
  - `import.rs::preview_uses_default_mapping_for_known_headers`: Kopfzeile aus Spec §6.4 ergibt `mapping[4]=="name"`.
  - `import.rs::preview_reports_not_empty_database` (Review Focus 4): Existiert eine Org, ist `databaseEmpty == false`.
    `import_apply` liefert dann den Core-Text „Die Datenbank ist nicht leer…".
  - `import.rs::wrong_mapping_length_is_refused`; `import.rs::unknown_field_key_is_refused` (`Unbekanntes Feld: xyz`).
  - `import.rs::apply_imports_rows`: 3 Zeilen ergeben die erwarteten Zählerstände.
  - `import.rs::field_keys_roundtrip`: `parse_field(field_key(f)) == Some(f)` für alle Felder.
- [ ] **Step 2:** Run `cargo test -p verwaltung_app --test backup --test import`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen.
- [ ] **Step 4:** Run `cargo test -p verwaltung_app` und `cargo clippy -p verwaltung_app -- -D warnings`. Expected: alles grün, keine Warnungen.
- [ ] **Step 5: Commit** `feat: add backup, restore, full export and import commands`

---

### Task 7: Tauri-Verdrahtung und Frontend-Grundlagen (Typen, API, Datum, Formulare, Navigation, Hülle, Start)

**Files:**
- Create: `crates/app/src/tauri_commands.rs`; Modify: `crates/app/src/main.rs`
- Create: `ui/src/lib/{types.ts, api.ts, dates.ts, forms.ts, router.ts, session.ts}` + Tests
- Create: `ui/src/components/{Sidebar, NoticeBar, DateInput, ConfirmDialog}.svelte`, `ui/src/pages/Start.svelte`; Modify: `ui/src/App.svelte`, `ui/src/app.css`

**Interfaces:**
- Consumes: alle `commands::*` aus Tasks 2–6.
- Produces:
  - Tauri-State `AppState { session: Mutex<Session>, config_dir: PathBuf }`. Jede Command-Funktion aus Tasks 2–6
    bekommt eine Hülle mit gleichem Namen in snake_case. Für `today` gilt `Local::now().date_naive()`.
    Zusätzlich gibt es `last_path() -> Option<String>`. `open_file`/`create_file` speichern nach Erfolg den Pfad.
    Beim Fenster-Ereignis `CloseRequested` wird `session.close(Local::now())` aufgerufen, Fehler werden nur protokolliert.
  - `ui/src/lib/types.ts`: TS-Typen genau nach den Rust-DTOs/Core-Strukturen (Core-Structs in snake_case, DTOs camelCase).
  - `ui/src/lib/api.ts`: eine typisierte Funktion pro Befehl, z.B. `searchAddresses(filter: Filter): Promise<SearchResult>`.
    Fehler werden als `Error(message)` weitergereicht.
  - `dates.ts`: `formatDate(iso: string | null): string` ergibt `"TT.MM.JJJJ"` bzw. `""`.
    `parseDate(input: string): { ok: true; iso: string | null } | { ok: false; error: string }`
    akzeptiert `T.M.JJJJ` und `TT.MM.JJJJ`. Leere Eingabe ergibt `iso: null`. Sonst gilt der Fehler
    `Ungültiges Datum (TT.MM.JJJJ)`.
  - `forms.ts`: `emptyToNull(v: string): string | null` (trimmt) und `toIntOrNull(v: string): { ok, value | error }`
    (Fehler `Bitte eine ganze Zahl eingeben`).
    Dazu `organisationInputFromForm(form): { input: OrganisationInput } | { errors: Record<string,string> }` und
    analog `personInputFromForm`, `affiliationInputFromForm`, `membershipInputFromForm`.
  - `router.ts`: Svelte-Store `route` mit `Route = {page:'start'} | {page:'addresses'; filter?: Filter} | {page:'organisation'; id: number} | {page:'person'; id: number} | {page:'teams'} | {page:'groups'} | {page:'settings'; tab?: 'catalog'|'backup'|'import'|'export'}`, `navigate(r)`, `back()` (Verlauf, erhält den Adressen-Filter).
  - `session.ts`: Store `sessionInfo` und `dataVersion` (Zahl). `refreshSession()` ruft `sessionInfo()`,
    `bumpData()` erhöht die Version. Seiten laden neu, wenn sich `dataVersion` ändert.
  - `NoticeBar` zeigt „Nur lesen – geöffnet von {user} ({host}) seit {TT.MM.JJJJ HH:MM}" mit Button „Übernehmen".
    Vor dem Übernehmen kommt die Rückfrage: „Wer die Datei noch offen hat, kann danach nicht mehr speichern.
    Übernehmen?". Ausserdem zeigt sie „Letzte Sicherung am zweiten Ort ist älter als 30 Tage" bzw.
    „Kein zweiter Sicherungsort eingerichtet" mit Link zu Einstellungen › Datensicherung.
  - `Start.svelte`: Buttons „Datei öffnen…" und „Neue Datei anlegen…" (Dialog-Plugin, Filter `*.sqlite`).
    Ist ein Pfad gemerkt, wird er direkt angeboten („Zuletzt: …").
  - `App.svelte`: Seitenleiste (Adressen, Teams, Gruppen, Einstellungen) und Inhaltsbereich nach `route`.
    Nach jedem Fehler eines Schreibbefehls läuft `refreshSession()`, damit die Leiste einen Sperrverlust sofort zeigt.

- [ ] **Step 1: Write failing tests:**
  - `dates.test.ts`:
    - `formatDate("2024-03-05")=="05.03.2024"`, `formatDate(null)==""`
    - `parseDate("5.3.2024")` → iso `"2024-03-05"`
    - `parseDate("")` → `iso null`
    - `parseDate("31.02.2024")`, `parseDate("1.2.24")`, `parseDate("2024")`, `parseDate("abc")` → `ok:false`
  - `forms.test.ts`:
    - `emptyToNull("  ")===null`
    - ein Org-Formular mit Name `" Beck AG "` und leerer Mail ergibt `input.name=="Beck AG"`, `email===null`
    - leerer Name ergibt `errors.name`
    - `branches:"2,5"` ergibt `errors.branches`
    - Mitgliedschaft mit Austritt `"31.02.2024"` ergibt `errors.left_on`
  - `router.test.ts`: Nach `navigate(addresses mit Filter) → navigate(organisation 3) → back()` ist die Route wieder Adressen mit gleichem Filter.
  - `session.test.ts` (mit gemocktem `api`): `refreshSession` setzt den Store, `bumpData` erhöht die Version.
- [ ] **Step 2:** Run `pnpm --dir ui test -- --run`. Expected: FAIL (Module fehlen).
- [ ] **Step 3:** Umsetzen. Styles schlicht, hell, Systemschrift, gut lesbare Tabellen. Keine externen Ressourcen.
- [ ] **Step 4:** Run `pnpm --dir ui test -- --run`, `pnpm --dir ui check` (svelte-check), `cargo build -p verwaltung_app`. Expected: grün, keine Typfehler.
- [ ] **Step 5:** Manuell `cargo tauri dev`: neue Datei anlegen, Programm schliessen, wieder öffnen. Expected: „Zuletzt: …" wird angeboten, `backups/` neben der Datei enthält eine Sicherung.
- [ ] **Step 6: Commit** `feat: wire tauri commands and add app shell`

---

### Task 8: Seite „Adressen" – Suche, Filter, Tabelle, Aktionen

**Files:**
- Create: `ui/src/lib/filters.ts`, `ui/src/lib/contact.ts` + Tests, `ui/src/components/LabelPicker.svelte`, `ui/src/pages/Addresses.svelte`

**Interfaces:**
- Consumes: `api.searchAddresses, exportAddresses, saveGroup, filterOptions`, `router`, Clipboard- und Opener-Plugin.
- Produces:
  - `filters.ts`: `emptyFilter(): Filter`, `withText(f, text)`, `toggleId(f, key, id)`, `setKinds(f, kinds)`,
    `setMembership(f, status)`, `activeFilterCount(f): number`. Alle Funktionen liefern **neue** Objekte.
  - `contact.ts`: `contactActions(result: SearchResult): { canCopy: boolean; canMailto: boolean; hint: string | null }`.
    Hinweise: kein Eintrag mit Mail ergibt „Keine Mail-Adressen in dieser Liste",
    `mailto == null` bei Adressen ergibt „Zu viele Adressen für das Mailprogramm – bitte kopieren".
  - Die Seite sucht verzögert (300 ms) bei Texteingabe und sofort bei Filteränderung. Sie zeigt
    „N Einträge · M ohne Mail" (M aufklappbar mit Namen) und die Buttons Mails kopieren, Mail öffnen,
    Excel, CSV, Als Gruppe speichern (Namensabfrage). Ein Zeilenklick öffnet das Detail.
    Ausserdem gibt es die Buttons „Neuer Betrieb" und „Neue Person".

- [ ] **Step 1: Write failing tests:**
  - `filters.test.ts`: `toggleId` zweimal ergibt die Ausgangsliste. Das Original-Objekt bleibt unverändert
    (`Object.freeze` im Test). `activeFilterCount` zählt Text, Arten, IDs und Status.
  - `contact.test.ts` (Review Focus 3): leeres Ergebnis ergibt `canCopy=false, canMailto=false`. Gibt es Adressen,
    aber `mailto=null`, ist `canCopy=true, canMailto=false` mit dem „zu viele"-Hinweis. Normalfall: beides true, `hint=null`.
- [ ] **Step 2:** Run `pnpm --dir ui test -- --run`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen.
- [ ] **Step 4:** Run `pnpm --dir ui test -- --run && pnpm --dir ui check`. Expected: grün.
- [ ] **Step 5: Commit** `feat: add address list page`

---

### Task 9: Detailseiten Betrieb und Person

**Files:**
- Create: `ui/src/pages/OrganisationDetail.svelte`, `ui/src/pages/PersonDetail.svelte`, ggf. `ui/src/components/{AffiliationEditor, MembershipEditor, LabelAssignments}.svelte`
- Test: `ui/src/components/*.test.ts` für die Editoren (Testing Library)

**Interfaces:**
- Consumes: `api.organisationDetail/personDetail/create*/update*/delete*/assignLabel/unassignLabel/listPeople/listOrganisations/filterOptions`, `forms.ts`, `DateInput`, `ConfirmDialog`.
- Produces:
  - Betrieb mit Stammdaten. „Personen" als Liste mit Funktion und Von/Bis, Schalter „Frühere anzeigen",
    Buttons „Person verknüpfen" (bestehende auswählen oder neu anlegen) und Bearbeiten/Beenden (Bis-Datum setzen).
    Dazu Mitgliedschaften (Kategorie, Eintritt, Austritt), Labels (nur Gruppen mit passendem `applies_to`)
    und „Löschen" mit Rückfrage.
  - Person mit Stammdaten inkl. „Honorar/Lohn", Betrieben, Mitgliedschaft, Labels und Teams (Anzeige; Link zu Teams).
  - Neuanlage über dieselbe Seite mit `id: null`. Nach dem Speichern führt `navigate` zur ID.
  - Feldfehler erscheinen am Feld, Core-Fehler oben im Formular.

- [ ] **Step 1: Write failing tests:**
  - `AffiliationEditor.test.ts`: Ein ungültiges Von-Datum zeigt „Ungültiges Datum (TT.MM.JJJJ)" und ruft das gemockte
    `api.createAffiliation` nicht auf. Ein gültiges ruft es mit ISO-Datum auf.
  - `MembershipEditor.test.ts`: Kommt aus der API der Fehler „Austritt liegt vor Eintritt", wird er im Formular angezeigt.
- [ ] **Step 2:** Run `pnpm --dir ui test -- --run`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen. Nach jedem Speichern `bumpData()`, nach Fehlern `refreshSession()`.
- [ ] **Step 4:** Run `pnpm --dir ui test -- --run && pnpm --dir ui check`. Expected: grün.
- [ ] **Step 5: Commit** `feat: add organisation and person detail pages`

---

### Task 10: Seiten „Teams" und „Gruppen"

**Files:**
- Create: `ui/src/pages/Teams.svelte`, `ui/src/pages/Groups.svelte`
- Test: `ui/src/pages/Groups.test.ts`

**Interfaces:**
- Consumes: `api.listTeams/teamMembers/createTeam/updateTeam/deleteTeam/addTeamMember/removeTeamMember/listPeople/listGroups/loadGroup/deleteGroup`, `router.navigate`.
- Produces:
  - Teams: links die Liste mit Mitgliederzahl, rechts die Mitglieder (Name → Person, Rolle editierbar),
    „Person hinzufügen", Team anlegen/umbenennen/löschen. Dazu der Button „Team anschreiben", der zu „Adressen"
    mit `team_ids=[id]` führt.
  - Gruppen: eine Liste. Klick lädt den Filter und öffnet „Adressen" damit. Ausserdem Löschen mit Rückfrage.

- [ ] **Step 1: Write failing test** `Groups.test.ts`: Ein Klick auf die Gruppe „Vorstand" (gemockt `loadGroup` → Filter mit `label_ids:[4]`) setzt `route` auf Adressen mit diesem Filter.
- [ ] **Step 2:** Run `pnpm --dir ui test -- --run`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen.
- [ ] **Step 4:** Run `pnpm --dir ui test -- --run && pnpm --dir ui check`. Expected: grün.
- [ ] **Step 5: Commit** `feat: add teams and groups pages`

---

### Task 11: Einstellungen – Listen & Labels, Datensicherung, Import-Assistent, Gesamtexport

**Files:**
- Create: `ui/src/pages/Settings.svelte`, `ui/src/pages/settings/{Catalog, Backup, ImportWizard, FullExport}.svelte`, `ui/src/lib/importMapping.ts` + Test
- Test: `ui/src/pages/settings/Backup.test.ts`

**Interfaces:**
- Consumes: Katalog-, Backup- und Import-Befehle aus Tasks 5/6, Dialog-Plugin, `session.bumpData/refreshSession`.
- Produces:
  - Catalog: Tabs für Funktionen, Betriebsarten, Mitgliederkategorien, Kantonalverbände und Labels
    (Gruppen mit „gilt für"). Jeder Eintrag zeigt die Nutzungszahl und hat Umbenennen und Löschen.
    Bei Nutzung > 0 kommt die Rückfrage „Wird N-mal verwendet. Trotzdem löschen?" und dann `confirm=true`.
  - Backup: Status, Ordner, Liste der Sicherungen (neueste zuerst), „Jetzt sichern", „Zweiten Ort wählen…/entfernen".
    „Wiederherstellen" fragt vorher: „Der aktuelle Stand wird vorher gesichert. Wiederherstellen?".
    Danach folgen `bumpData()` und `refreshSession()`.
  - ImportWizard, 3 Schritte:
    1. Datei wählen.
    2. Zuordnung: Tabelle Spalte → Feld-Auswahl mit deutschen Namen aus `importMapping.ts`, darunter eine
       Vorschau mit Zählern und Bericht.
    3. Übernehmen.
    Bei `databaseEmpty=false` ist „Übernehmen" gesperrt, mit dem Hinweis „Import nur in eine leere Datei möglich.
    Bitte neue Datei anlegen." Danach zeigt er die Zusammenfassung und den Bericht (Zeile, Spalte, Meldung).
    Der Bericht lässt sich als CSV speichern; dafür bekommt `write_csv` einen eigenen Befehl `export_report`
    in Task 6-Stil (Test in `tests/import.rs::report_export_writes_csv`).
  - FullExport: „Alle Daten als Excel exportieren…" mit Speichern-Dialog, Vorschlag `Mitgliederverwaltung-JJJJ-MM-TT.xlsx`.
  - `importMapping.ts`: `FIELD_LABELS: Record<string,string>` (deutsche Namen für jeden Feldschlüssel, z.B.
    `name: "Name/Firma"`, `fr_baeck: "Fr Bäck (Label)"`, `ignore: "– nicht übernehmen –"`) und
    `setColumn(mapping: string[], index: number, key: string): string[]` (neues Array).

- [ ] **Step 1: Write failing tests:**
  - `importMapping.test.ts`: `setColumn` liefert ein neues Array, das Original bleibt gleich. Jeder Schlüssel aus einer
    Beispiel-`fields`-Liste mit allen 24 Feldern hat ein Label.
  - `Backup.test.ts` (Review Focus 5): Nach bestätigtem „Wiederherstellen" sind `api.restoreBackup`, `bumpData` und
    `refreshSession` aufgerufen. Ist `sessionInfo.readOnly`, sind die Buttons „Wiederherstellen" und „Jetzt sichern" deaktiviert.
  - Rust: `report_export_writes_csv`.
- [ ] **Step 2:** Run `pnpm --dir ui test -- --run` und `cargo test -p verwaltung_app --test import`. Expected: FAIL.
- [ ] **Step 3:** Umsetzen.
- [ ] **Step 4:** Run `pnpm --dir ui test -- --run && pnpm --dir ui check && cargo test -p verwaltung_app`. Expected: grün.
- [ ] **Step 5: Commit** `feat: add settings with catalog, backup, import and export`

---

### Task 12: Abnahme – Coverage, Durchklicken, Anleitung

**Files:**
- Create: `README.md` (Deutsch: was das Programm ist, Starten für Entwicklung, Tests, Datenablage, Backup-Konzept kurz)
- Modify: `ui/vite.config.ts` (Coverage-Schwelle 80 % für `src/lib`)

- [ ] **Step 1:** Run `cargo llvm-cov -p verwaltung_app --ignore-filename-regex 'tauri_commands|main\.rs' --summary-only`
  (ggf. vorher `cargo install cargo-llvm-cov`) und `pnpm --dir ui test -- --run --coverage`.
  Expected: beide ≥ 80 % Zeilen. Fehlt etwas, Tests für die Lücken ergänzen (Test zuerst).
- [ ] **Step 2:** Run `cargo clippy --workspace -- -D warnings` und `cargo test --workspace`. Expected: keine Warnungen, alles grün.
- [ ] **Step 3:** Manuell mit `cargo tauri dev` durchklicken:
  1. Neue Datei anlegen.
  2. Betrieb mit Person (Funktion Inhaber) anlegen.
  3. Ein Label vergeben.
  4. Filter nach dem Label und als Gruppe speichern.
  5. Mails kopieren.
  6. Excel exportieren.
  7. Zweiten Sicherungsort setzen und „Jetzt sichern".
  8. Wiederherstellen.
  9. Ein zweites Programmfenster mit derselben Datei zeigt „Nur lesen".

  Expected: Jeder Schritt funktioniert, alle Texte sind Deutsch, Daten erscheinen als TT.MM.JJJJ.
  Abweichungen als Ruling oder Fix (Test zuerst) behandeln.
- [ ] **Step 4:** README schreiben.
- [ ] **Step 5: Commit** `docs: add README and coverage gate for app`

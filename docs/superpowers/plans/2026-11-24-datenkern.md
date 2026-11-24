# Datenkern (Rust Core) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the UI-free Rust library `verwaltung_core` that stores, queries, imports and exports all member data of the Bäckervereinigung in a single SQLite file, with file locking and automatic backups.

**Architecture:** Cargo workspace with one library crate `crates/core`. Free functions per domain module take `&rusqlite::Connection` and return plain data structs. A `Workspace` type owns lock, backup and connection. Later plans (Tauri UI, CI/Release) depend only on this crate's public API.

**Tech Stack:** Rust (stable, edition 2021), rusqlite 0.32 (`bundled`), chrono 0.4, serde/serde_json 1, thiserror 1, csv 1, rust_xlsxwriter 0.79, calamine 0.26, tempfile 3 (dev). Coverage: cargo-llvm-cov.

**Spec:** `docs/superpowers/specs/2026-11-24-mitgliederverwaltung-design.md`

**Follow-up plans (not part of this one):** 2) Tauri 2 + Svelte UI, 3) GitHub Actions build/release + E2E.

## Global Constraints

- Prerequisite: Rust is not installed on the dev machine. Install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, then `cargo install cargo-llvm-cov`.
- SQLite journal mode `DELETE`, never WAL. `PRAGMA foreign_keys = ON` on every connection.
- No running costs, no network access from the core.
- Dates displayed and exported as `TT.MM.JJJJ` (`%d.%m.%Y`). Swiss spelling: "ss", never "ß", in all user-facing strings.
- All user-facing error messages are German.
- Backups: keep the newest **20**.
- Test coverage ≥ 80 % (`cargo llvm-cov --workspace --fail-under-lines 80`).
- Immutability: repository functions take input structs by reference and return new structs; no in-place mutation of caller data.
- Files ≤ 400 lines; functions < 50 lines.

## Review Focus

1. **Umlauts in search:** SQLite `LIKE` only folds ASCII case, so "müller" would not find "Müller". Search uses a Rust-normalized `search_text` column (lowercase). Test in Task 5.
2. **Stale lock after a crash or a network drive dropout:** the second user must get read-only mode with the holder shown, and a deliberate takeover. Tests in Tasks 2 and 3.
3. **Deleting a label or list value that is in use:** must report the usage count and only delete with `confirm = true`. Deleting an organisation removes its affiliations but keeps the persons. Tests in Tasks 4 and 6.
4. **Messy Excel cells:** trailing spaces, PLZ stored as float (`8000.0`), uppercase `X`, empty rows, year-only dates. Tests in Task 11.
5. **Bad e-mail data:** duplicates in different case, surrounding whitespace, values without `@`. These are deduplicated or reported as missing, never passed on silently. Test in Task 9.

---

## File Structure

```
Cargo.toml                         workspace
crates/core/Cargo.toml
crates/core/src/lib.rs             module declarations + re-exports
crates/core/src/error.rs           CoreError, Result
crates/core/src/db/mod.rs          open_connection, migrate
crates/core/src/db/0001_init.sql   schema
crates/core/src/storage/lock.rs    LockFile
crates/core/src/storage/backup.rs  backups
crates/core/src/storage/workspace.rs
crates/core/src/model.rs           shared plain data types
crates/core/src/text.rs            normalize_search, format_date
crates/core/src/repo/{lookups,labels,organisations,persons,affiliations,memberships,assignments,teams}.rs
crates/core/src/query/{filter,saved_groups}.rs
crates/core/src/contact.rs
crates/core/src/export.rs
crates/core/src/import/{sheet,values,mapping,apply}.rs
crates/core/tests/<module>_test.rs one integration test file per module
crates/core/tests/common/mod.rs    fn test_conn() -> Connection (in-memory, migrated)
```

---

### Task 1: Workspace scaffold, errors, schema and migrations

**Files:**
- Create: `Cargo.toml`, `crates/core/Cargo.toml`, `crates/core/src/lib.rs`, `crates/core/src/error.rs`, `crates/core/src/db/mod.rs`, `crates/core/src/db/0001_init.sql`, `crates/core/tests/common/mod.rs`, `.gitignore` (`/target`)
- Test: `crates/core/tests/db_test.rs`

**Interfaces:**
- Produces:
  - `pub enum CoreError { Db(rusqlite::Error), Io(std::io::Error), ReadOnly, LockHeld(LockInfo), NotFound { entity: &'static str, id: i64 }, InUse { count: i64 }, DatabaseNotEmpty, Validation(String), Import(String) }` with `thiserror` German `Display` texts (e.g. `ReadOnly` → "Die Datei ist schreibgeschützt geöffnet."). `pub type Result<T> = std::result::Result<T, CoreError>;`. Map rusqlite `SQLITE_READONLY` to `CoreError::ReadOnly` in `From<rusqlite::Error>`.
  - `db::open_connection(path: &Path, read_only: bool) -> Result<Connection>`: sets `journal_mode=DELETE`, `foreign_keys=ON`, then calls `migrate` unless read-only.
  - `db::open_in_memory() -> Result<Connection>`: migrated in-memory connection (used by tests).
  - `db::migrate(conn: &Connection) -> Result<()>`: applies embedded migrations (`include_str!`) whose index > `PRAGMA user_version`, each in a transaction, then sets `user_version`.
  - `db::SCHEMA_VERSION: i64 = 1`.

Schema `0001_init.sql` (exact tables; all `id INTEGER PRIMARY KEY`, dates `TEXT` ISO `YYYY-MM-DD`):
- `lookup_values(kind TEXT NOT NULL CHECK(kind IN ('org_type','function','membership_category','cantonal_association')), name TEXT NOT NULL, sort INTEGER NOT NULL DEFAULT 0, UNIQUE(kind,name))`
- `label_groups(name TEXT NOT NULL UNIQUE, applies_to TEXT NOT NULL CHECK(applies_to IN ('person','organisation','both')))`
- `labels(group_id INTEGER NOT NULL REFERENCES label_groups ON DELETE CASCADE, name TEXT NOT NULL, UNIQUE(group_id,name))`
- `organisations(name, addition, street, po_box, zip, city, phone, email, website, branches INTEGER, apprentice_count INTEGER, notes, legacy_id TEXT, org_type_id REFERENCES lookup_values, cantonal_association_id REFERENCES lookup_values, search_text TEXT NOT NULL)`, where `name TEXT NOT NULL` and the other text fields are nullable.
- `persons(salutation, first_name, last_name NOT NULL, street, zip, city, phone, mobile, email, compensation, notes, search_text NOT NULL)`
- `affiliations(person_id NOT NULL REFERENCES persons ON DELETE CASCADE, organisation_id NOT NULL REFERENCES organisations ON DELETE CASCADE, function_id NOT NULL REFERENCES lookup_values, valid_from, valid_to)`
- `memberships(organisation_id REFERENCES organisations ON DELETE CASCADE, person_id REFERENCES persons ON DELETE CASCADE, category_id REFERENCES lookup_values, joined_on, left_on, CHECK((organisation_id IS NULL) <> (person_id IS NULL)))`
- `label_assignments(label_id NOT NULL REFERENCES labels ON DELETE CASCADE, organisation_id REFERENCES organisations ON DELETE CASCADE, person_id REFERENCES persons ON DELETE CASCADE, assigned_on, note, CHECK((organisation_id IS NULL) <> (person_id IS NULL)))`
- `teams(name NOT NULL UNIQUE, description)`, `team_members(team_id NOT NULL REFERENCES teams ON DELETE CASCADE, person_id NOT NULL REFERENCES persons ON DELETE CASCADE, role, UNIQUE(team_id, person_id))`
- `saved_groups(name NOT NULL UNIQUE, filter_json NOT NULL)`

- [ ] **Step 1: Write failing tests** in `db_test.rs`:
  - `creates_file_with_schema_version_1`: `open_connection(tmp/"d.sqlite", false)`, then assert the file exists and `PRAGMA user_version` == 1.
  - `uses_delete_journal_and_foreign_keys`: `PRAGMA journal_mode` == `"delete"` and `PRAGMA foreign_keys` == 1.
  - `migrate_is_idempotent`: open, close, open again: no error, version still 1.
  - `read_only_connection_rejects_writes`: create the db, reopen with `read_only = true`, insert into `teams`, assert `Err(CoreError::ReadOnly)`.
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test db_test`. Expected: FAIL (crate does not compile).
- [ ] **Step 3: Implement** the files above.
- [ ] **Step 4: Run** `cargo test -p verwaltung_core --test db_test`. Expected: 4 passed.
- [ ] **Step 5: Commit** `feat: scaffold core crate with schema and migrations`

---

### Task 2: Lock file

**Files:**
- Create: `crates/core/src/storage/lock.rs` (+ `storage/mod.rs`)
- Test: `crates/core/tests/lock_test.rs`

**Interfaces:**
- Produces:
  - `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)] pub struct LockInfo { pub user: String, pub host: String, pub created_at: chrono::DateTime<Utc> }`
  - `pub struct Identity { pub user: String, pub host: String }`
  - `pub fn lock_path(db_path: &Path) -> PathBuf`: returns `<db_path>.lock`.
  - `pub struct LockGuard`. `LockGuard::acquire(db_path: &Path, who: &Identity) -> Result<LockGuard>` creates the file atomically (`OpenOptions::create_new`) with JSON `LockInfo`. If the file exists, returns `Err(CoreError::LockHeld(info))`. An unreadable or corrupt lock file yields `LockInfo { user: "unbekannt", host: "unbekannt", created_at: file mtime }`.
  - `LockGuard::take_over(db_path: &Path, who: &Identity) -> Result<LockGuard>` overwrites the existing lock.
  - `impl Drop for LockGuard`: deletes the file **only if** its content still equals our `LockInfo` (so a taken-over lock is never removed by the old holder).
  - `pub fn is_stale(info: &LockInfo, now: DateTime<Utc>) -> bool`: true when older than 24 h (UI hint only).

- [ ] **Step 1: Write failing tests:** `acquire_creates_lock_file`, `second_acquire_returns_lock_held_with_holder` (assert `info.user == "anna"`), `drop_removes_own_lock`, `take_over_replaces_holder_and_old_drop_keeps_new_lock`, `corrupt_lock_file_reports_unbekannt`, `is_stale_after_24h` (23 h → false, 25 h → true).
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test lock_test`. Expected: FAIL.
- [ ] **Step 3: Implement** `lock.rs`.
- [ ] **Step 4: Run** the same. Expected: 6 passed.
- [ ] **Step 5: Commit** `feat: add lock file with takeover`

---

### Task 3: Backups and `Workspace::open`

**Files:**
- Create: `crates/core/src/storage/backup.rs`, `crates/core/src/storage/workspace.rs`
- Test: `crates/core/tests/workspace_test.rs`

**Interfaces:**
- Consumes: `LockGuard`, `Identity`, `LockInfo` (Task 2); `db::open_connection` (Task 1).
- Produces:
  - `backup::create_backup(db_path: &Path, now: DateTime<Local>) -> Result<Option<PathBuf>>`: copies to `<dir>/backups/<stem>-YYYYMMDD-HHMMSS.sqlite`. Returns `None` if the db file does not exist yet.
  - `backup::prune(db_path: &Path, keep: usize) -> Result<()>`: deletes the oldest backups (by file name) beyond `keep`.
  - `pub const BACKUP_KEEP: usize = 20;`
  - `pub enum AccessMode { ReadWrite, ReadOnly { held_by: LockInfo } }`
  - `pub struct Workspace { pub conn: Connection, pub mode: AccessMode, /* private: path, lock */ }`
  - `Workspace::open(db_path: &Path, who: &Identity) -> Result<Workspace>`: acquire lock → backup + prune → `open_connection(rw)`. On `LockHeld(info)`: if the db exists, open read-only with `AccessMode::ReadOnly { held_by: info }`, otherwise return `Err(LockHeld)`.
  - `Workspace::open_taking_over(db_path: &Path, who: &Identity) -> Result<Workspace>`: same, using `LockGuard::take_over`.

- [ ] **Step 1: Write failing tests:** `first_open_creates_db_without_backup`, `second_open_creates_backup`, `prune_keeps_newest_20` (create 22 dummy backups with increasing names, assert 20 remain and the 2 oldest are gone), `open_while_locked_is_read_only_with_holder`, `read_only_workspace_rejects_writes` (`CoreError::ReadOnly`), `take_over_gives_read_write`, `locked_and_missing_db_returns_lock_held`.
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test workspace_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: 7 passed.
- [ ] **Step 5: Commit** `feat: add workspace with backups and read-only fallback`

---

### Task 4: Lookup values, label groups and labels

**Files:**
- Create: `crates/core/src/model.rs` (start), `crates/core/src/repo/lookups.rs`, `crates/core/src/repo/labels.rs`
- Test: `crates/core/tests/lookups_test.rs`, `crates/core/tests/labels_test.rs`

**Interfaces:**
- Produces (`model.rs`):
  - `pub enum LookupKind { OrgType, Function, MembershipCategory, CantonalAssociation }` with `as_str()` matching the SQL `CHECK` values.
  - `pub struct LookupValue { pub id: i64, pub kind: LookupKind, pub name: String, pub sort: i64 }`
  - `pub enum AppliesTo { Person, Organisation, Both }`
  - `pub struct LabelGroup { pub id: i64, pub name: String, pub applies_to: AppliesTo }`, `pub struct Label { pub id: i64, pub group_id: i64, pub name: String }`
- Produces (repo):
  - `lookups::{create(conn, kind, name: &str) -> Result<LookupValue>, list(conn, kind) -> Result<Vec<LookupValue>>, rename(conn, id, name) -> Result<LookupValue>, find_or_create(conn, kind, name) -> Result<LookupValue>, usage_count(conn, id) -> Result<i64>, delete(conn, id, confirm: bool) -> Result<()>}`
  - `labels::{create_group(conn, name, applies_to) -> Result<LabelGroup>, list_groups(conn) -> Result<Vec<LabelGroup>>, create_label(conn, group_id, name) -> Result<Label>, list_labels(conn, group_id) -> Result<Vec<Label>>, rename_label(conn, id, name) -> Result<Label>, find_or_create_label(conn, group_name, applies_to, label_name) -> Result<Label>, label_usage_count(conn, id) -> Result<i64>, delete_label(conn, id, confirm: bool) -> Result<()>, delete_group(conn, id, confirm: bool) -> Result<()>}`
  - Delete rule: if usage > 0 and `!confirm` → `Err(CoreError::InUse { count })`. With `confirm`: lookup values in use as `org_type_id`, `cantonal_association_id` or `category_id` are set to NULL. A `function` value in use is never deleted → `Err(Validation("Funktion wird noch verwendet"))`, because affiliations require a function.
  - Names are trimmed. An empty name → `Err(Validation("Name darf nicht leer sein"))`.

- [ ] **Step 1: Write failing tests:** `create_and_list_sorted_by_sort_then_name`, `rename_updates_value`, `find_or_create_is_idempotent`, `empty_name_is_rejected`, `delete_in_use_without_confirm_returns_in_use_count` (count == 2), `delete_confirmed_nulls_references`, `function_in_use_cannot_be_deleted`, `label_delete_confirmed_removes_assignments`, `group_delete_cascades_labels`.
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test lookups_test --test labels_test`. Expected: FAIL.
- [ ] **Step 3: Implement.** Tests needing organisations/assignments insert rows with raw SQL until Tasks 5 and 7 exist.
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add lookup values and labels`

---

### Task 5: Organisations and persons

**Files:**
- Create: `crates/core/src/text.rs`, `crates/core/src/repo/organisations.rs`, `crates/core/src/repo/persons.rs`
- Test: `crates/core/tests/text_test.rs`, `crates/core/tests/organisations_test.rs`, `crates/core/tests/persons_test.rs`

**Interfaces:**
- Produces:
  - `text::normalize_search(parts: &[Option<&str>]) -> String`: joins non-empty parts with a space, trimmed, `to_lowercase()` (Unicode-aware).
  - `text::format_date(d: NaiveDate) -> String` → `"%d.%m.%Y"`.
  - `pub struct OrganisationInput { name: String, addition, street, po_box, zip, city, phone, email, website: Option<String>, branches, apprentice_count: Option<i64>, notes, legacy_id: Option<String>, org_type_id, cantonal_association_id: Option<i64> }` (all `pub`, `Default`). `pub struct Organisation { pub id: i64, /* same fields */ }`.
  - `pub struct PersonInput { salutation, first_name: Option<String>, last_name: String, street, zip, city, phone, mobile, email, compensation, notes: Option<String> }`. `pub struct Person { pub id: i64, ... }` with `fn display_name(&self) -> String` ("Vorname Nachname").
  - `organisations::{create(conn, &OrganisationInput) -> Result<Organisation>, update(conn, id, &OrganisationInput) -> Result<Organisation>, get(conn, id) -> Result<Organisation>, delete(conn, id) -> Result<()>, list(conn) -> Result<Vec<Organisation>>}`. Same set in `persons`.
  - Writes trim all text fields, store empty strings as NULL and recompute `search_text` from name, addition, city, email (org) or first_name, last_name, city, email (person). Validation: empty name/last_name → `Validation`. `get` on a missing id → `NotFound`.

- [ ] **Step 1: Write failing tests:** `normalize_search_lowercases_umlauts` (`["Bäckerei MÜLLER", "Zürich"]` → `"bäckerei müller zürich"`), `format_date_swiss` (2024-03-05 → `"05.03.2024"`), `create_trims_and_blanks_become_none`, `update_recomputes_search_text`, `get_missing_returns_not_found`, `empty_name_rejected`, `person_display_name`, `delete_organisation_keeps_persons` (affiliation via raw SQL until Task 6).
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test text_test --test organisations_test --test persons_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add organisations and persons`

---

### Task 6: Affiliations and memberships

**Files:**
- Create: `crates/core/src/repo/affiliations.rs`, `crates/core/src/repo/memberships.rs`
- Test: `crates/core/tests/affiliations_test.rs`, `crates/core/tests/memberships_test.rs`

**Interfaces:**
- Produces:
  - `pub struct AffiliationInput { pub person_id: i64, pub organisation_id: i64, pub function_id: i64, pub valid_from: Option<NaiveDate>, pub valid_to: Option<NaiveDate> }`, `pub struct Affiliation { pub id: i64, ...same }` with `fn is_current(&self, today: NaiveDate) -> bool` (`valid_to` is None or ≥ today).
  - `affiliations::{create, update(conn, id, &AffiliationInput), delete(conn, id), for_person(conn, person_id, include_past: bool, today) -> Result<Vec<Affiliation>>, for_organisation(conn, org_id, include_past: bool, today) -> Result<Vec<Affiliation>>}`. Validation: `valid_to < valid_from` → `Validation("Bis-Datum liegt vor Von-Datum")`.
  - `pub enum MemberRef { Organisation(i64), Person(i64) }`. `pub struct MembershipInput { pub member: MemberRef, pub category_id: Option<i64>, pub joined_on: Option<NaiveDate>, pub left_on: Option<NaiveDate> }`. `pub struct Membership { pub id: i64, ...same }` with `fn is_active(&self, today) -> bool` (`left_on` is None or > today).
  - `memberships::{create, update, delete, for_member(conn, &MemberRef) -> Result<Vec<Membership>>}`.

- [ ] **Step 1: Write failing tests:** `person_can_have_two_affiliations`, `past_affiliation_hidden_by_default_and_shown_with_include_past`, `valid_to_before_from_rejected`, `deleting_person_removes_affiliations`, `membership_for_person_and_for_organisation`, `membership_active_rules` (no left_on → active; left_on tomorrow → active; left_on today → inactive).
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test affiliations_test --test memberships_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add affiliations and memberships`

---

### Task 7: Label assignments and teams

**Files:**
- Create: `crates/core/src/repo/assignments.rs`, `crates/core/src/repo/teams.rs`
- Test: `crates/core/tests/assignments_test.rs`, `crates/core/tests/teams_test.rs`

**Interfaces:**
- Consumes: `MemberRef` (Task 6), `AppliesTo` (Task 4).
- Produces:
  - `pub struct AssignmentInput { pub label_id: i64, pub target: MemberRef, pub assigned_on: Option<NaiveDate>, pub note: Option<String> }`. `pub struct Assignment { pub id: i64, ...same }`.
  - `assignments::{assign(conn, &AssignmentInput) -> Result<Assignment>, unassign(conn, id) -> Result<()>, for_target(conn, &MemberRef) -> Result<Vec<Assignment>>}`. If the label's group `applies_to` excludes the target kind → `Validation("Label gilt nicht für diesen Eintragstyp")`. Assigning the same label twice to the same target is allowed (e.g. a course attended twice with different dates).
  - `pub struct Team { pub id: i64, pub name: String, pub description: Option<String> }`, `pub struct TeamMember { pub person_id: i64, pub role: Option<String> }`.
  - `teams::{create(conn, name, description: Option<&str>) -> Result<Team>, update(conn, id, name, description) -> Result<Team>, delete(conn, id) -> Result<()>, list(conn) -> Result<Vec<Team>>, add_member(conn, team_id, person_id, role: Option<&str>) -> Result<()>, remove_member(conn, team_id, person_id) -> Result<()>, members(conn, team_id) -> Result<Vec<TeamMember>>}`. Adding an existing member updates the role.

- [ ] **Step 1: Write failing tests:** `assign_with_date_and_note`, `person_only_label_rejected_for_organisation`, `same_label_twice_allowed`, `team_add_member_twice_updates_role`, `deleting_team_removes_members_not_persons`.
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test assignments_test --test teams_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add label assignments and teams`

---

### Task 8: Search, filters and saved groups

**Files:**
- Create: `crates/core/src/query/filter.rs`, `crates/core/src/query/saved_groups.rs`
- Test: `crates/core/tests/filter_test.rs`, `crates/core/tests/saved_groups_test.rs`

**Interfaces:**
- Consumes: all repos above, `text::normalize_search`.
- Produces:
  - `#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)] pub struct Filter { pub text: Option<String>, pub kinds: Vec<EntryKind>, pub label_ids: Vec<i64>, pub team_ids: Vec<i64>, pub function_ids: Vec<i64>, pub org_type_ids: Vec<i64>, pub category_ids: Vec<i64>, pub cantonal_ids: Vec<i64>, pub membership: MembershipStatus }`
  - `pub enum EntryKind { Person, Organisation }` (empty `kinds` = both). `pub enum MembershipStatus { #[default] Any, Active, Exited }`.
  - `pub struct ContactRow { pub kind: EntryKind, pub id: i64, pub display_name: String, pub organisation: Option<String>, pub street, zip, city, po_box, email, phone, mobile: Option<String> }`. For persons, `organisation` = names of current affiliations joined with ", ".
  - `filter::search(conn, &Filter, today: NaiveDate) -> Result<Vec<ContactRow>>`, sorted by `display_name` (case-insensitive).
  - Semantics (pin in tests):
    - `text`: each whitespace-separated, normalized word must occur in `search_text` (`instr`, AND).
    - `label_ids`: labels of the **same group are OR-ed, different groups AND-ed**.
    - `team_ids`, `function_ids`: OR within the list. Function filter matches persons with a current affiliation having that function. Organisation-scoped filters (`org_type_ids`, `cantonal_ids`) match organisations directly and persons via a current affiliation.
    - `category_ids` and `membership` match a membership of the entry itself; for persons, a membership of a currently affiliated organisation also counts.
    - Filters that cannot apply to a kind (e.g. `team_ids` for organisations) exclude that kind.
  - `saved_groups::{save(conn, name, &Filter) -> Result<i64>, list(conn) -> Result<Vec<(i64, String)>>, load(conn, id) -> Result<Filter>, delete(conn, id) -> Result<()>}`. Saving an existing name overwrites it.

- [ ] **Step 1: Write failing tests** (fixture: 2 orgs, 4 persons, labels Vorstand + Prüfungsexperte in group "Verbandsfunktion", label "Fr Conf" in "Ausbildung Fachrichtung", one team):
  - `text_search_finds_umlaut_case_insensitive` ("müller" finds "Bäckerei Müller")
  - `multiple_words_are_anded`
  - `labels_same_group_or` (Vorstand OR Prüfungsexperte → 3 persons)
  - `labels_different_groups_and`
  - `team_filter_excludes_organisations`
  - `function_filter_ignores_past_affiliations`
  - `active_membership_via_organisation_counts_for_person`
  - `exited_members_only`
  - `saved_group_roundtrip_and_overwrite`
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test filter_test --test saved_groups_test`. Expected: FAIL.
- [ ] **Step 3: Implement.** Build SQL with `rusqlite::params_from_iter` placeholders only; never interpolate values into SQL.
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add search filters and saved groups`

---

### Task 9: Contact helpers

**Files:**
- Create: `crates/core/src/contact.rs`
- Test: `crates/core/tests/contact_test.rs`

**Interfaces:**
- Consumes: `ContactRow` (Task 8).
- Produces:
  - `pub struct EmailList { pub addresses: Vec<String>, pub joined: String, pub missing: Vec<ContactRow> }`. `joined` = addresses joined with `"; "` (Outlook BCC).
  - `contact::email_list(rows: &[ContactRow]) -> EmailList`: trims, dedupes case-insensitively (keeps the first spelling) and preserves order. Rows with no e-mail or one without `@` go to `missing`.
  - `contact::mailto_bcc(addresses: &[String]) -> Option<String>`: `mailto:?bcc=a,b` (percent-encoded). Returns `None` if longer than `MAILTO_MAX_LEN = 2000`.

- [ ] **Step 1: Write failing tests:** `dedupes_case_insensitive_and_trims` (`" A@x.ch"`, `"a@X.ch"` → one), `invalid_and_empty_go_to_missing`, `joined_uses_semicolon_space`, `mailto_none_when_too_long`, `mailto_encodes_plus_and_umlaut`.
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test contact_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add e-mail list and mailto helpers`

---

### Task 10: Export (CSV and XLSX)

**Files:**
- Create: `crates/core/src/export.rs`
- Test: `crates/core/tests/export_test.rs`

**Interfaces:**
- Consumes: `ContactRow`.
- Produces:
  - `pub struct Table { pub headers: Vec<String>, pub rows: Vec<Vec<String>> }`
  - `export::contact_table(rows: &[ContactRow]) -> Table`. Headers exactly: `Typ, Name, Betrieb, Adresse, Postfach, PLZ, Ort, Mail, Telefon, Mobiltelefon`. `Typ` is `Person` or `Betrieb`. This table doubles as the Word mail-merge source.
  - `export::write_csv(table: &Table, path: &Path) -> Result<()>`: UTF-8 **with BOM**, delimiter `;`, quote when needed (Swiss Excel opens it correctly).
  - `export::write_xlsx(table: &Table, path: &Path, sheet_name: &str) -> Result<()>`: bold header row, all cells written as strings (keeps leading zeros), autofilter on the header.

- [ ] **Step 1: Write failing tests:** `contact_table_headers_exact`, `csv_has_bom_and_semicolons_and_umlauts` (read back bytes: starts with `EF BB BF`, contains `Bäckerei Müller;`), `csv_quotes_field_with_semicolon`, `xlsx_roundtrip_with_calamine` (read back, header and one row equal, the PLZ `"0800"` stays a string).
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test export_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add csv and xlsx export`

---

### Task 11: Import – reading the sheet and parsing values

**Files:**
- Create: `crates/core/src/import/mod.rs`, `crates/core/src/import/sheet.rs`, `crates/core/src/import/values.rs`
- Test: `crates/core/tests/import_values_test.rs`, `crates/core/tests/import_sheet_test.rs` (fixtures are built in the test with `rust_xlsxwriter` into a tempdir; no binary fixtures in git)

**Interfaces:**
- Produces:
  - `pub struct Sheet { pub headers: Vec<String>, pub rows: Vec<SheetRow> }`, `pub struct SheetRow { pub line: usize /* 1-based Excel row */, pub cells: Vec<Cell> }`, `pub enum Cell { Empty, Text(String), Number(f64), Bool(bool), Date(NaiveDate) }`.
  - `sheet::read_first_sheet(path: &Path) -> Result<Sheet>` (calamine, `.xlsx`/`.xls`): the first row is the header (trimmed), fully empty rows are skipped, Excel date cells become `Cell::Date`.
  - `values::as_text(&Cell) -> Option<String>`: trimmed, empty → None. A whole-number `Number` renders without decimals (`8000.0` → `"8000"`).
  - `values::as_flag(&Cell) -> FlagResult` where `pub enum FlagResult { Yes, No, Unknown(String) }`: Yes for `x`, `ja`, `j`, `1`, `true`, `wahr` (case-insensitive) and numbers ≠ 0; No for empty, `nein`, `n`, `0`, `false`, `falsch`.
  - `values::as_date(&Cell) -> DateResult` where `pub enum DateResult { None, Date(NaiveDate), YearOnly(NaiveDate) /* 01.01. */, Invalid(String) }`. Accepts `Cell::Date`, Excel serials (`Number`), `TT.MM.JJJJ`, `T.M.JJJJ`, `JJJJ`.
  - `values::as_int(&Cell) -> Result<Option<i64>, String>`.

- [ ] **Step 1: Write failing tests:** `text_trims_and_renders_whole_floats`, `flag_variants` (table-driven, includes `"X"`, `" ja "`, `"vielleicht"` → Unknown), `date_variants` (`"5.3.2019"`, `"2019"` → YearOnly, serial 43529.0 → 2019-03-05, `"31.02.2019"` → Invalid), `int_rejects_text`, `reads_headers_and_skips_empty_rows` (row line numbers preserved).
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test import_values_test --test import_sheet_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run.** Expected: all pass.
- [ ] **Step 5: Commit** `feat: add excel reading and value parsing for import`

---

### Task 12: Import – mapping, dry-run plan and apply

**Files:**
- Create: `crates/core/src/import/mapping.rs`, `crates/core/src/import/apply.rs`
- Test: `crates/core/tests/import_test.rs`

**Interfaces:**
- Consumes: Task 11 types; repos from Tasks 4–7.
- Produces:
  - `pub enum Field { LegacyId, ApprenticeCount, CantonalAssociation, Name, FirstName, Addition, Street, Zip, City, Category, Email, Website, Apprenticeship, FrBaeck, FrConf, Dh, Notes, Joined, Left, Branches, Phone, Mobile, PoBox, Ignore }`
  - `pub struct Mapping { pub columns: Vec<(String /* header */, Field)> }` with `Mapping::default_for(headers: &[String]) -> Mapping`, matching the spec headers exactly (after trim, case-insensitive): `ID1→LegacyId, Anzahl→ApprenticeCount, Kt Verband→CantonalAssociation, Name/Firma→Name, Vorname→FirstName, Zusatz→Addition, Adresse→Street, PLZ→Zip, Ort→City, Kategorie→Category, Mail→Email, Home→Website, Lehrbetrieb→Apprenticeship, Fr Bäck→FrBaeck, Fr Conf→FrConf, DH→Dh, Besonderes→Notes, Eintritt→Joined, Austritt→Left, Filialen→Branches, Telefon→Phone, Mobiltelefon→Mobile, Postfach→PoBox`. Unknown headers → `Ignore`.
  - `pub struct ReportEntry { pub line: usize, pub column: Option<String>, pub message: String }` (German message).
  - `pub struct PlannedRow { pub line: usize, pub organisation: OrganisationInput, pub person: Option<PersonInput>, pub category: Option<String>, pub cantonal: Option<String>, pub flags: Vec<&'static str>, pub joined: Option<NaiveDate>, pub left: Option<NaiveDate> }`
  - `pub struct ImportPlan { pub rows: Vec<PlannedRow>, pub report: Vec<ReportEntry> }`
  - `mapping::plan(sheet: &Sheet, mapping: &Mapping) -> ImportPlan`: pure, no DB (= dry run / preview). Rules exactly as in spec §6.4:
    - Empty `Vorname` → organisation only.
    - `Vorname` set → person (last_name = Name/Firma, first_name = Vorname) plus organisation (name = Zusatz if set, else "Vorname Name"; addition = None in that case).
    - Mobile goes to the person if one exists, else to the organisation.
    - Flags: `Lehrbetrieb` alone adds no label; `FrBaeck/FrConf/Dh = Yes` add labels `"Fr Bäck"`, `"Fr Conf"`, `"DH"` in group `"Ausbildung Fachrichtung"` (applies_to Organisation). `Lehrbetrieb = Yes` with no Fachrichtung → report "Lehrbetrieb ohne Fachrichtung".
    - Unknown flag, invalid date, non-integer Anzahl/Filialen → report entry, value skipped.
    - YearOnly date → report entry "Nur Jahr angegeben, 01.01. angenommen".
    - Rows without Name/Firma → report entry, row skipped.
    - Duplicate organisation names (case-insensitive) → report entry "Doppelter Firmenname", both kept.
  - `apply::apply(conn: &mut Connection, plan: &ImportPlan) -> Result<ImportSummary>` where `pub struct ImportSummary { pub organisations: usize, pub persons: usize, pub report: Vec<ReportEntry> }`. Returns `Err(DatabaseNotEmpty)` if `organisations` or `persons` has any row. Runs in **one transaction** (all or nothing). Creates missing lookups/labels via `find_or_create`. Person rows get an affiliation with function `"Inhaber"` (find_or_create). Memberships are attached to the organisation.

- [ ] **Step 1: Write failing tests:**
  - `default_mapping_matches_spec_headers` (all 23 headers mapped, none `Ignore`)
  - `row_without_first_name_is_organisation_only`
  - `row_with_first_name_creates_person_org_and_inhaber` (Zusatz "Bäckerei Muster" → org name; without Zusatz → "Hans Muster")
  - `fachrichtung_flags_become_labels`
  - `messy_row_produces_report_not_failure` (`"vielleicht"` flag, `"31.02.2019"`, Anzahl `"viele"`, PLZ `8000.0` → zip `"8000"`)
  - `duplicate_firm_names_reported`
  - `apply_into_non_empty_db_is_rejected`
  - `apply_is_atomic` (force a failure mid-way, e.g. a lookup name with only whitespace that passed planning, then assert 0 rows)
  - `end_to_end_import_then_filter` (import an xlsx built in the test, then `filter::search` with label "Fr Conf" finds the expected org)
- [ ] **Step 2: Run** `cargo test -p verwaltung_core --test import_test`. Expected: FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test --workspace`. Expected: all pass.
- [ ] **Step 5: Coverage gate** `cargo llvm-cov --workspace --fail-under-lines 80`. Expected: exit 0.
- [ ] **Step 6: Commit** `feat: add excel import with dry run and report`

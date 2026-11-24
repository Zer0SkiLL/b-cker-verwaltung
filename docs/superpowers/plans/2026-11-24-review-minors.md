# Review-Minors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Die acht freigegebenen Minors aus dem Review von Plan 2 beheben.

**Architecture:** Umbenennen im Core (`saved_groups::rename`, `labels::rename_group`), Sperr-Hinweis und
Übernehmen-Rückfall in `crates/app/src/session.rs`, Rest in `ui/`.

**Tech Stack:** wie Plan 2.

**Spec:** `docs/superpowers/specs/2026-11-24-review-minors-design.md`

## Global Constraints
Wie Plan 2 (Deutsch, TDD, Dateien < 400 Zeilen, Funktionen < 50 Zeilen, unveränderliche Daten im TS,
Commits `<type>: <description>` + `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`).

## Review Focus
1. Umbenennen auf einen schon vergebenen Namen → deutscher Fehler, nicht die englische UNIQUE-Meldung.
2. Übernehmen schlägt fehl → Sitzung bleibt offen (nur lesen), alle Ansichten funktionieren weiter.
3. Eigene Sperre nach Absturz → Hinweis erscheint; fremde, frische Sperre → kein Hinweis.
4. Neue Person + Verknüpfung: schlägt die Verknüpfung fehl, ist die Person trotzdem angelegt → Meldung sagt das.
5. Sitzungszustand noch unbekannt → Schreib-Buttons gesperrt.

---

### Task 1: Core – Gruppe und Label-Gruppe umbenennen
**Files:** Modify `crates/core/src/query/saved_groups.rs`, `crates/core/src/repo/labels.rs`; Test `crates/core/tests/saved_groups_test.rs`, `crates/core/tests/labels_test.rs`
**Produces:** `saved_groups::rename(conn, id, name: &str) -> Result<()>`, `labels::rename_group(conn, id, name: &str) -> Result<LabelGroup>`.
Doppelter Name → `CoreError::Validation("Name ist bereits vergeben")`; unbekannte ID → `NotFound`; leerer Name → `require_name`-Fehler.
- [ ] Tests: `rename_group_changes_name`, `rename_to_existing_name_is_refused` (Text exakt), `rename_unknown_id_not_found` – je für beide.
- [ ] RED, umsetzen, GREEN (`cargo test -p verwaltung_core`). Commit `feat: rename saved groups and label groups`.

### Task 2: App – Umbenennen-Befehle, Absturz-Hinweis, Übernehmen-Rückfall, Pfad merken
**Files:** Modify `crates/app/src/{session.rs,dto.rs,commands/addresses.rs,commands/catalog.rs,tauri_commands/*}`; Create `crates/app/src/commands/session.rs`; Tests in `crates/app/tests/{session.rs,addresses.rs,catalog.rs,remember.rs}`
**Produces:**
- `addresses::rename_group(s, id, name)`, `catalog::rename_label_group(s, id, name) -> LabelGroup` (+ Tauri-Hüllen `rename_group`, `rename_label_group`).
- `LockDto.likelyCrash: bool` = `is_stale(info, now)` oder (user, host) gleich der eigenen Identität. `Session` merkt sich die Identität beim Öffnen.
- `Session::take_over`: schlägt `open_taking_over` fehl → `Workspace::open` (nur lesend) wiederherstellen, Fehler zurückgeben.
- `commands::session::open_and_remember(s, config_dir, path, who, create: bool) -> AppResult<OpenResult { info: SessionInfo, warning: Option<String> }>`; Speicherfehler → `warning = "Die Datei ist geöffnet, wird beim nächsten Start aber nicht vorgeschlagen."`.
- [ ] Tests: `rename_group_command`, `rename_label_group_command`, `own_lock_is_likely_crash`, `foreign_fresh_lock_is_not_likely_crash`, `failed_take_over_keeps_file_open_read_only` (Sperrdatei durch Verzeichnis ersetzen → Schreiben schlägt fehl), `remember_failure_is_only_a_warning` (config_dir ist eine Datei).
- [ ] RED, umsetzen, GREEN (`cargo test -p verwaltung_app`), clippy. Commit `fix: crash hint, safe take-over and remembered path`.

### Task 3: UI – Anzeige und Bedienung
**Files:** Modify `ui/src/lib/{types.ts,api.ts,session.ts}`, `ui/src/pages/{Start,Addresses,Groups,Teams,OrganisationDetail,PersonDetail}.svelte`, `ui/src/pages/settings/LabelEditor.svelte`, `ui/src/components/{NoticeBar,AffiliationEditor}.svelte`
**Produces:** `readOnly` (derived store: `sessionInfo?.readOnly ?? true`) in `session.ts`, von allen Seiten genutzt; `noticesFor` hängt bei `heldBy.likelyCrash` „ – wahrscheinlich nach einem Absturz, bitte „Übernehmen““ an.
- [ ] Tests: `session.test.ts` (readOnly unbekannt = true; Absturz-Hinweis), `Groups.test.ts` (umbenennen ruft `renameGroup`), `AffiliationEditor.test.ts` (Neue Person: `createPerson` dann `createAffiliation` mit neuer ID; scheitert die Verknüpfung → Meldung „Person wurde angelegt, die Verknüpfung aber nicht: …“), `Teams.test.ts` (fehlgeschlagenes Anlegen behält Eingabe), `OrganisationDetail.test.ts` (Ladefehler verschwindet nach erfolgreichem Neuladen).
- [ ] RED, umsetzen, GREEN (`pnpm --dir ui exec vitest run && pnpm --dir ui check`). Commit `fix: read-only consistency, renames and inline person creation`.

# Problembericht 2.0 und Frühwarnung – Umsetzungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Problembericht im Dashboard-Layout, als Datei speicherbar, mit Systemwerten, Laufzeiten, Sperre, Datei-Angaben und lesbarem Protokoll; dazu Diagnose-Ordner, Absturz-Hinweis, Selbsttest, Warnung bei langsamen Aktionen und befristetes Senden an einen Discord-Webhook.

**Architecture:** Rust sammelt Fakten (`diagnostics_snapshot`), bereitet Protokollzeilen auf und versendet; TypeScript (`buildReport`) gestaltet den Text. Automatische Berichte: Rust meldet WARN/ERROR als Tauri-Ereignis, die Oberfläche baut den Bericht und übergibt ihn `diagnostics_publish`, das ihn in den Diagnose-Ordner und den Ausgang schreibt und an Discord schickt.

**Tech Stack:** Rust (sysinfo, windows-sys, reqwest 0.13 aus dem Baum, chrono, tauri-plugin-log/fern), Svelte 5, TypeScript, Vitest.

**Spec:** `docs/superpowers/specs/2026-11-27-problembericht-design.md`

**Abweichung von der Spec (bewusst):** Die Datei im Diagnose-Ordner wird beim Öffnen und bei jedem Ereignis geschrieben, nicht beim Beenden: Beim Beenden läuft die Oberfläche, die den Bericht baut, nicht mehr zuverlässig. Das Programmende steht im Protokoll und damit in der nächsten Fassung der Datei.

## Global Constraints

- Alle Texte deutsch, Schweizer Schreibweise („ss“), keine Gendersprache.
- Keine Vereinsdaten, kein Dateipfad, kein Windows-Benutzername in Bericht, Protokoll-Auszug oder Nachricht; PC-Name erlaubt.
- Webhook-URL nur aus `option_env!("MV_DIAGNOSE_WEBHOOK")`; nie in Code, Protokoll, Fehlermeldungen, Oberfläche. Lokal liegt sie in `~/.tauri/mitgliederverwaltung.diagnose-webhook` und wird nie ausgegeben.
- `DIAGNOSE_UNTIL = 2027-05-31`, `DIAGNOSE_DEFAULT = true`, Lebenszeichen alle 7 Tage, höchstens 10 Nachrichten/Tag/PC, Ausgang höchstens 20 Dateien, Senden-Timeout 10 s, Nachsenden alle 15 min, Warnungsfenster 60 s.
- Grenzen: langsam > 2000 ms, Öffnen > 3 s, freier Platz < 500 MB, Sicherung älter 7 Tage, Programm-RAM ≥ 1 GB.
- Kein Teil darf die Arbeit blockieren: alle Diagnose-Fehler werden geschluckt und höchstens als INFO protokolliert.
- Commits `<type>: <Beschreibung>`; `feat`/`fix` deutsch aus Nutzersicht; Ende `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Prüfungen: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir ui exec vitest run`, `pnpm --dir ui check`, `node --test 'scripts/*.test.mjs'`.

## Review Focus

1. Webhook-URL taucht in einer `reqwest`-Fehlermeldung oder Protokollzeile auf (ohne `without_url()`) → nie.
2. Rückkopplung: Senden scheitert → WARN → neues Ereignis → Senden … → Protokollzeilen des Versands lösen kein Ereignis aus (INFO, oder Ziel filtert `diagnose`-Modul).
3. Netzlaufwerk weg: Diagnose-Ordner nicht schreibbar, Grösse nicht lesbar → Bericht trotzdem vollständig, Status „nicht erreichbar“, kein Absturz, kein Hängen.
4. Protokollzeile mit Benutzerordner (`C:\Users\Hans\…`, `/Users/hans/…`) → maskiert.
5. Erster Start überhaupt (kein Protokoll, keine Konfiguration, kein Ausgang) → kein Absturz-Hinweis, kein Fehler; ein erstes Lebenszeichen wird gesendet (noch nie gesendet = fällig).

---

### Task 1: Protokoll aufbereiten, maskieren, Starts erkennen

**Files:**
- Create: `crates/app/src/commands/logfile.rs` (in `commands/mod.rs` eintragen)
- Modify: `crates/app/src/commands/diagnostics.rs` (`log_tail` nutzt Aufbereitung)
- Test: `crates/app/tests/logfile.rs`

**Interfaces:**
- Produces:
  - `pub fn read_log_lines(dir: &Path) -> AppResult<Vec<String>>` (alle Zeilen, rotierte zuerst; aus bisherigem `log_tail` herausgelöst)
  - `pub fn mask_user_dirs(line: &str) -> String`
  - `pub fn pretty_line(line: &str) -> String` → `26.11. 21:33:53  WARNUNG  Text` (Stufen: `INFO    `, `WARNUNG `, `FEHLER  `, sonst `DEBUG`/`TRACE` wie `INFO`; Stufe auf 8 Zeichen aufgefüllt); unlesbare Zeile → nur maskiert
  - `pub fn log_excerpt(lines: &[String]) -> Vec<String>` (Auswahl wie bisher, aufbereitet, Lücke → `…`)
  - `#[derive(Serialize)] #[serde(rename_all="camelCase")] pub struct StartDto { pub at: String /* RFC 3339 lokal */, pub clean_before: Option<bool> }`
  - `pub fn start_history(lines: &[String], limit: usize) -> Vec<StartDto>` (neueste zuerst; `clean_before: None` beim ältesten Start im Protokoll)
  - `pub fn problems_today(lines: &[String], today: NaiveDate) -> usize`
  - Konstanten `START_MARKER = "Programmstart"`, `END_MARKER = "Programmende"`

- [ ] **Step 1: Failing tests** in `crates/app/tests/logfile.rs`:
  - `pretty_line("[2026-11-26][21:33:53][verwaltung_app::tauri_commands][WARN] Fehler an die Oberfläche: X") == "26.11. 21:33:53  WARNUNG  Fehler an die Oberfläche: X"`; ebenso INFO/ERROR.
  - `mask_user_dirs(r"Dateifehler C:\Users\Hans\Desktop\a.sqlite") == r"Dateifehler C:\Users\…\Desktop\a.sqlite"`; `/Users/hans/x` → `/Users/…/x`; `/home/anna/x` → `/home/…/x`; Zeile ohne Pfad unverändert.
  - `pretty_line("kaputt")` == `"kaputt"`.
  - `log_excerpt` mit 60 INFO-Zeilen und einer WARN in Zeile 5: enthält die WARN, dann `"…"`, dann die letzten 10; Reihenfolge alt → neu.
  - `start_history`: Zeilen Start A, Ende, Start B, Start C → `[C(clean_before=Some(false)), B(Some(true)), A(None)]`.
  - `problems_today` zählt nur WARN/ERROR mit heutigem Datum.
- [ ] **Step 2:** `cargo test -p verwaltung_app --test logfile` → FAIL (Modul fehlt).
- [ ] **Step 3:** Implementieren; `log_tail` = `log_excerpt(&read_log_lines(dir)?)`. Masken ohne Regex-Crate (Suche nach `\Users\`, `/Users/`, `/home/`, Segment bis zum nächsten Trenner ersetzen).
- [ ] **Step 4:** Test + bestehende `crates/app/tests/diagnostics.rs` → PASS.
- [ ] **Step 5:** Commit `chore: Protokoll lesbar aufbereiten und Starts erkennen`.

### Task 2: Momentaufnahme (System, Programm, Datei)

**Files:**
- Create: `crates/app/src/commands/resources.rs`, `crates/app/src/commands/location.rs`
- Modify: `crates/app/src/commands/diagnostics.rs`, `crates/app/Cargo.toml` (`sysinfo = "0.40"`, `[target.'cfg(windows)'.dependencies] windows-sys = { version = "0.59", features = ["Win32_Storage_FileSystem"] }`), `crates/app/src/tauri_commands/session.rs`, `crates/app/src/main.rs` (Befehl registrieren, Höchstwert-Thread starten), `ui/src/lib/api.ts`, `ui/src/lib/types.ts`
- Test: `crates/app/tests/snapshot.rs`

**Interfaces:**
- Consumes: Task 1 (`read_log_lines`, `log_excerpt`, `start_history`, `problems_today`).
- Produces (alle DTOs `camelCase`):
  - `ResourcesDto { cpu_total: Option<f32>, cpu_program: Option<f32>, mem_used: Option<u64>, mem_total: Option<u64>, mem_program: Option<u64>, mem_program_peak: Option<u64>, program_since: Option<String>, computer_since: Option<String> }` (Bytes, Prozent 0–100, RFC 3339)
  - `pub fn tree_sum(own: u32, procs: &[(u32, Option<u32>, u64, f32)]) -> (u64, f32)` (pid, parent, mem, cpu; Summe über eigenen Prozess und Nachkommen)
  - `pub fn measure() -> ResourcesDto`; `pub fn record_peak(bytes: u64)`; `pub fn peak() -> u64` (static `AtomicU64`); `pub fn start_peak_sampler()` (Thread, alle 60 s)
  - `pub enum LocationKind { NetworkDrive(char), NetworkPath, Removable, Local, Unknown }` mit `label() -> String` („Netzlaufwerk (S:)“, „Netzpfad“, „externes Laufwerk oder Netz“, „lokal“, „unbekannt“)
  - `pub fn classify(path: &str, drive_is_remote: Option<bool>) -> LocationKind` (rein, testbar) und `pub fn location_of(path: &Path) -> LocationKind` (Windows: `GetDriveTypeW` → `DRIVE_REMOTE`)
  - `FileFactsDto { location: String, size: Option<u64>, schema_version: Option<i64>, organisations: Option<i64>, persons: Option<i64>, active_members: Option<i64>, last_local_backup: Option<String>, lock_host: Option<String>, lock_since: Option<String>, lock_likely_crash: bool, reachable: bool }`
  - `pub fn file_facts(s: &Session, today: NaiveDate) -> Option<FileFactsDto>` (None ohne Datei; jeder Einzelwert darf fehlen)
  - `SnapshotDto { system: SystemInfoDto, host: String, resources: ResourcesDto, file: Option<FileFactsDto>, starts: Vec<StartDto>, problems_today: usize, log: Vec<String> }`
  - Tauri: `diagnostics_snapshot(app, state) -> AppResult<SnapshotDto>`; TS `api.diagnosticsSnapshot(): Promise<T.SnapshotDto>` (ersetzt `diagnosticsSystem`/`diagnosticsLogTail` in der Oberfläche)

- [ ] **Step 1: Failing tests:** `tree_sum(10, [(10,None,100,1.0),(11,Some(10),50,2.0),(12,Some(11),25,0.5),(99,Some(1),999,9.0)]) == (175, 3.5)`; `classify(r"\\server\share\a.sqlite", None)` → `NetworkPath`; `classify("S:\\a.sqlite", Some(true))` → `NetworkDrive('S')`, label „Netzlaufwerk (S:)“; `classify("C:\\a", Some(false))` → `Local`; `classify("/Volumes/X/a", None)` → `Removable`; `classify("/home/x/a", None)` → `Local`. `file_facts` auf `fresh_session` mit einem Betrieb: `organisations == Some(1)`, `schema_version == Some(SCHEMA_VERSION)`, `size > 0`, `reachable`, kein Feld enthält den Ordnernamen der Testdatei. `measure()` liefert `mem_total.is_some()`.
- [ ] **Step 2:** `cargo test -p verwaltung_app --test snapshot` → FAIL.
- [ ] **Step 3:** Implementieren. CPU: zwei Refreshs mit `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL` Abstand, Prozess-CPU / Kernzahl. Sperre aus der bestehenden Sitzungs-Info (`held_by`), nur `host`/`since`/`likely_crash`. Letzte Sicherung: `backup::list(local_backup_dir(p), &db_stem(p))` erstes Element.
- [ ] **Step 4:** Tests PASS, clippy sauber.
- [ ] **Step 5:** Commit `chore: Momentaufnahme für den Problembericht`.

### Task 3: Absturz, Programmende, Protokoll aus der Oberfläche, Export-Protokoll, Speichern

**Files:**
- Modify: `crates/app/src/main.rs` (Panic-Hook in `setup` nach dem Logger; `log::info!("Programmende")` in `RunEvent::Exit`), `crates/app/src/commands/diagnostics.rs`, `crates/app/src/commands/addresses.rs`, `crates/app/src/commands/backup.rs` (`export_all`), `crates/app/src/commands/import.rs` (`export_report`), `crates/app/src/tauri_commands/session.rs`, `ui/src/lib/api.ts`, `ui/src/main.ts`, `ui/src/lib/updates.ts`
- Test: `crates/app/tests/diagnostics.rs`, `ui/src/lib/updates.test.ts`

**Interfaces:**
- Produces:
  - `pub fn panic_message(info: &std::panic::PanicHookInfo) -> String` → `Absturz: <Meldung> (<Datei>:<Zeile>)`; `pub fn install_panic_hook()` (loggt `log::error!`, ruft danach den vorherigen Hook)
  - `pub fn log_from_ui(level: &str, message: &str)` (`warn`/`error`/sonst info; Nachricht auf 500 Zeichen, maskiert)
  - `pub fn save_report(path: &Path, text: &str) -> AppResult<()>`
  - Tauri `diagnostics_log(level, message)`, `save_report(path, text)`; TS `logToFile(level: 'info'|'warn'|'error', message: string): void` (direkt `invoke`, Fehler geschluckt, nicht in der Aktionsliste), `api.saveReport(path, text)`
  - Export-Protokoll: `Export Adressen: N Zeilen, X ms`, `Gesamtexport: X ms`, `Import-Bericht gespeichert: N Zeilen`
  - Update-Prüfung: `logToFile('info', 'Update-Prüfung: keine neue Version' | 'Update-Prüfung: Version X verfügbar')`, Fehler `logToFile('warn', 'Update-Prüfung fehlgeschlagen')`
  - `recordError` ruft zusätzlich `logToFile('error', 'Oberfläche: ' + message)`

- [ ] **Step 1: Failing tests:** `logs_contain_no_business_data` erweitert um `export_addresses` und `export_all` (Log enthält „Export Adressen“, aber nicht „Geheim“); `panic_message` über `std::panic::catch_unwind` mit gesetztem Hook, der in einen Puffer schreibt → beginnt mit `Absturz: boom (`; `save_report` schreibt UTF-8 inkl. „ü“; `log_from_ui("warn", 600 Zeichen)` protokolliert ≤ 500 Zeichen. Vitest: `checkForUpdates` ruft die injizierte Log-Funktion mit dem richtigen Text (deps um `log` erweitern).
- [ ] **Step 2:** Tests → FAIL.
- [ ] **Step 3:** Implementieren.
- [ ] **Step 4:** Tests PASS.
- [ ] **Step 5:** Commit `chore: Abstürze und Exporte im Protokoll`.

### Task 4: Bericht im Dashboard-Layout, Speichern als Datei

**Files:**
- Create: `ui/src/lib/reportLayout.ts` (Balken, Dauer, Rahmen, Statuszeilen)
- Modify: `ui/src/lib/diagnostics.ts` (`buildReport` neu), `ui/src/components/ReportDialog.svelte`, Tests `ui/src/lib/diagnostics.test.ts`, `ui/src/lib/reportLayout.test.ts`, `ui/src/components/ReportDialog.test.ts`

**Interfaces:**
- Consumes: `T.SnapshotDto` (Task 2), `api.saveReport` (Task 3).
- Produces:
  - `bar(ratio: number | null, width = 20): string` (`█`/`▌`/`░`; null → 20× `░`)
  - `duration(seconds: number): string` → `45 s`, `6 h 31 min`, `2 Tage 6 h`, `1 Tag 3 h`
  - `frame(lines: string[]): string[]` (╔═╗ ║ ╚═╝, Breite = längste + 4)
  - `type Status = 'ok' | 'warn' | 'none'`; `statusLines(input): { status: Status; label: string; text: string }[]` nach Spec-Tabelle
  - `buildReport(input: { description: string; now: Date; route: Route; session: SessionInfo | null; retention: RetentionPolicy | null; actions: ActionEntry[]; snapshot: SnapshotDto | null }): string`
  - ReportDialog: Knopf **Als Datei speichern…** → `pickSavePath(\`Problembericht-${todayStamp()}.txt\`, 'txt')` → `api.saveReport` → „Gespeichert: <Pfad>“; Fehler sichtbar.

- [ ] **Step 1: Failing tests:** `bar(0.12)` = `██▌` + 17 `░` (0.12×20 = 2.4, auf halbe Zeichen gerundet 2.5); `bar(0.0001)` beginnt mit `▌`; `bar(0)` = 20 `░`; `duration(23460) == '6 h 31 min'`, `duration(194400) == '2 Tage 6 h'`; `frame` gleich lange Zeilen; Bericht mit nur-lesend-Sitzung enthält `[ !! ] Schreibzugriff     nur lesend – gesperrt von PC-TEST seit` und nie den Benutzernamen der Sperre; ohne Sitzung `[ -- ] Datei`, kein Abschnitt `DATEI`; Abschnitte in Reihenfolge Kopf, `» `, STATUS, COMPUTER, LAUFZEITEN, DATEI, SEITE, LETZTE AKTIONEN, PROTOKOLL; `snapshot: null` → Bericht mit `unbekannt`, kein Fehler; Starts mit `clean_before=false` → `(!)`. Dialog-Test: Knopf vorhanden, Klick ruft gemockte Speicher-Funktion mit Pfad und Berichtstext, zeigt „Gespeichert:“.
- [ ] **Step 2:** `pnpm --dir ui exec vitest run src/lib src/components/ReportDialog.test.ts` → FAIL.
- [ ] **Step 3:** Implementieren; `ReportDialog` lädt `api.diagnosticsSnapshot()` beim Öffnen (Fehler → `null`).
- [ ] **Step 4:** Tests + `pnpm --dir ui check` PASS.
- [ ] **Step 5:** Commit `feat: Problembericht übersichtlicher und als Datei speicherbar`.

### Task 5: Selbsttest, langsame Aktionen, Hinweis nach Absturz

**Files:**
- Create: `crates/app/src/commands/selftest.rs`, `ui/src/lib/earlyWarning.ts`
- Modify: `crates/app/Cargo.toml` (`fs4 = "0.13"` für freien Platz, oder `sysinfo::Disks` nach Einhängepunkt – Umsetzer wählt, Netzlaufwerke müssen funktionieren), `crates/app/src/tauri_commands/session.rs` (`self_test`), `ui/src/lib/api.ts` (Langsam-Warnung in `call`), `ui/src/components/NoticeBar.svelte`, `ui/src/App.svelte`
- Test: `crates/app/tests/selftest.rs`, `ui/src/lib/earlyWarning.test.ts`, `ui/src/lib/api.test.ts` (neu, falls nicht vorhanden), `ui/src/components/NoticeBar.test.ts`

**Interfaces:**
- Produces:
  - `SelfTestDto { integrity_ok: bool, low_space: bool, backup_stale: bool, slow_open: bool }`; `pub fn self_test(s: &Session, open_ms: u128, now: DateTime<Local>) -> AppResult<SelfTestDto>` (loggt `Selbsttest: in Ordnung` bzw. je Befund WARN `Selbsttest: …`)
  - Öffnen misst Dauer: `open_file` gibt `openMs` im `OpenResult` zurück; Oberfläche ruft danach `api.selfTest(openMs)` im Hintergrund
  - TS: `SLOW_MS = 2000`; in `call`: bei Dauer > `SLOW_MS` → `logToFile('warn', \`Langsam: ${command} ${ms} ms\`)`
  - TS `earlyWarning.ts`: `crashNotice(starts: StartDto[], dismissed: string | null): StartDto | null` (neuester Start mit `cleanBefore === false` und `at !== dismissed`); `dismissCrash(at)` (localStorage `diagnose-absturz-gesehen`, try/catch); Store `selfTestNotices: string[]`
  - NoticeBar zeigt: „Das Programm wurde letztes Mal unerwartet beendet.“ + **Problem melden** + **Ausblenden**; „Die Datei hat einen Fehler. Bitte melden Sie das Problem.“; „Auf dem Laufwerk der Datei ist nur noch wenig Platz frei.“

- [ ] **Step 1: Failing tests:** `self_test` auf frischer Sitzung → alles `false` ausser `backup_stale` (keine Sicherung → `true`), Protokoll enthält `Selbsttest:`; `open_ms = 3500` → `slow_open`. `crashNotice` mit/ohne dismissed; `call` mit Fake-`invoke` (vi.mock) und Fake-`performance.now` 2500 ms → Log-Aufruf `Langsam: x 2500 ms`; NoticeBar mit Absturz-Start zeigt Text, **Ausblenden** entfernt ihn.
- [ ] **Step 2:** Tests → FAIL.
- [ ] **Step 3:** Implementieren.
- [ ] **Step 4:** Tests PASS.
- [ ] **Step 5:** Commit `feat: Programm prüft sich beim Öffnen und meldet Auffälligkeiten`.

### Task 6: Diagnose-Ordner, Ausgang und Versand an Discord (Rust)

**Files:**
- Create: `crates/app/src/commands/diagnose/mod.rs`, `…/settings.rs`, `…/outbox.rs`, `…/discord.rs`
- Modify: `crates/app/Cargo.toml` (`reqwest = { version = "0.13", default-features = false, features = ["blocking", "multipart", "rustls"] }` – Feature-Namen an 0.13 anpassen, bereits im Baum), `crates/app/src/main.rs` (Protokoll-Ziel `TargetKind::Dispatch`, das bei WARN/ERROR ausserhalb von `verwaltung_app::commands::diagnose` das Tauri-Ereignis `diagnose-ereignis` mit `{ kind: "warnung" }` sendet; Absturz-Hook sendet `{ kind: "absturz" }`; Nachsende-Thread alle 15 min), `crates/app/src/tauri_commands/session.rs`
- Test: `crates/app/tests/diagnose.rs`

**Interfaces:**
- Produces:
  - Konstanten: `DIAGNOSE_UNTIL: NaiveDate = 2027-05-31`, `DIAGNOSE_DEFAULT = true`, `MAX_PER_DAY = 10`, `MAX_OUTBOX = 20`, `HEARTBEAT_DAYS = 7`
  - `pub fn webhook() -> Option<&'static str>` = `option_env!("MV_DIAGNOSE_WEBHOOK").filter(|s| !s.is_empty())`
  - `DiagnoseSettings { enabled: Option<bool>, last_sent: Option<String>, last_heartbeat: Option<String>, sent_today: (String, u32) }` in `config_dir/diagnose.json`; `pub fn active(settings, today) -> bool` (= Webhook da ∧ heute ≤ UNTIL ∧ enabled.unwrap_or(DEFAULT))
  - `pub fn write_folder_file(db_path: &Path, host: &str, report: &str)` → `<dir>/Mitgliederverwaltung-Diagnose/<host>.txt` (Fehler → INFO)
  - `pub struct Message { kind: Kind, host: String, version: String, status_block: String, report: String, at: DateTime<Local> }`; `Kind { Absturz, Warnung, Selbsttest, Lebenszeichen }`
  - `pub fn enqueue(outbox: &Path, msg: &Message) -> AppResult<()>` (JSON-Datei; älteste über 20 löschen)
  - `pub fn flush(outbox: &Path, url: &str, settings: &mut DiagnoseSettings, today: NaiveDate) -> usize` (sendet der Reihe nach, löscht bei Erfolg, beachtet `MAX_PER_DAY`; Fehler → `log::info!("Analysedaten nicht gesendet: {}", e.without_url())`)
  - `pub fn payload(msg: &Message) -> (serde_json::Value, String /*Dateiname*/)`: Embed `title` `🔴 Absturz · HOST · 0.6.0`, `description` ```` ```\n<status_block ≤ 4000>\n``` ````, `color` (rot 0xD64545, orange 0xE08A1E, grün 0x3BA55C), Anhang `Problembericht-<host>-<JJJJMMTT-HHMM>.txt`
  - Tauri: `diagnostics_publish(kind, status_block, report)` (Ordner-Datei immer; bei `active` enqueue + flush in eigenem Thread), `diagnose_settings() -> { available: bool, enabled: bool, until: String, lastSent: Option<String> }`, `set_diagnose_enabled(bool)`, `diagnose_heartbeat_due() -> bool`

- [ ] **Step 1: Failing tests:** `active` für: kein Webhook → false (Funktion mit injizierter URL testen: `active_with(url: Option<&str>, …)`), Datum nach UNTIL → false, `enabled=Some(false)` → false, sonst true. `enqueue` 21× → 20 Dateien. `payload` Titel/Farbe/Codeblock; `status_block` mit 5000 Zeichen → gekürzt. `flush` gegen lokalen `TcpListener`-Server, der `204` antwortet → Datei gelöscht, Body enthält `Problembericht-`; Server antwortet `500` → Datei bleibt; nicht erreichbare URL `http://127.0.0.1:9/geheim-token` → erfasstes Protokoll enthält nicht `geheim-token`. `MAX_PER_DAY` greift. `write_folder_file` in Tempordner schreibt `<host>.txt`; in nicht beschreibbaren Pfad → kein Panic.
- [ ] **Step 2:** `cargo test -p verwaltung_app --test diagnose` → FAIL.
- [ ] **Step 3:** Implementieren.
- [ ] **Step 4:** Tests + clippy PASS.
- [ ] **Step 5:** Commit `chore: Analysedaten an Discord und Diagnose-Ordner`.

### Task 7: Frühwarnung in der Oberfläche, Schalter „Analysedaten senden“

**Files:**
- Modify: `ui/src/lib/earlyWarning.ts`, `ui/src/App.svelte` (Start: Ereignis-Listener, Absturz/Lebenszeichen prüfen, Ordner-Datei beim Öffnen), `ui/src/pages/settings/About.svelte`, `ui/src/lib/api.ts`, `ui/src/lib/types.ts`
- Test: `ui/src/lib/earlyWarning.test.ts`, `ui/src/pages/settings/About.test.ts`

**Interfaces:**
- Consumes: Task 4 (`buildReport`, `statusLines`), Task 6 Befehle.
- Produces:
  - `WINDOW_MS = 60_000`; `createCollector(publish: (kind) => void, setTimer = setTimeout)`: erstes Ereignis startet Fenster, weitere im Fenster zusammengefasst, Absturz sofort
  - `autoReport(kind): Promise<void>` (Snapshot holen, Bericht mit Beschreibung `(automatisch: Absturz|Warnung|Selbsttest|Lebenszeichen)`, Status-Block = Abschnitt STATUS + COMPUTER, `api.diagnosticsPublish`); schluckt alle Fehler
  - Start: wenn `crashNotice` → `autoReport('absturz')` einmal pro Vorfall (gleicher localStorage-Schlüssel getrennt: `diagnose-absturz-gesendet`); sonst wenn `diagnoseHeartbeatDue()` → `autoReport('lebenszeichen')`; Selbsttest-Befund → `autoReport('selbsttest')`
  - About: nur wenn `available`: Checkbox **Analysedaten senden**, Text aus Spec mit Datum aus `until` (Schweizer Format), Knopf **Was wird gesendet?** (zeigt Bericht in schreibgeschütztem Textfeld), „Zuletzt gesendet: …“ / „Noch nichts gesendet.“

- [ ] **Step 1: Failing tests:** Collector: 3 Warnungen in 10 s → 1 Aufruf nach 60 s; Absturz → sofort. About: `available=false` → kein Schalter; `available=true` → Checkbox angehakt, Klick ruft `setDiagnoseEnabled(false)`; **Was wird gesendet?** zeigt Text mit `PROBLEMBERICHT`; Text „31.05.2027“.
- [ ] **Step 2:** Tests → FAIL.
- [ ] **Step 3:** Implementieren.
- [ ] **Step 4:** Alle Vitest + `pnpm --dir ui check` PASS.
- [ ] **Step 5:** Commit `feat: Analysedaten in der Einführungsphase, abschaltbar unter Über das Programm`.

### Task 8: Release-Build, Handbuch, echter Test

**Files:**
- Modify: `.github/workflows/release.yml` (Schritt `tauri-action`: `MV_DIAGNOSE_WEBHOOK: ${{ secrets.DIAGNOSE_WEBHOOK_URL }}`), `docs/handbuch/21-problem-melden.md`, `docs/handbuch/19-ueber-das-programm.md`, `docs/handbuch/20-haeufige-fragen.md`, `CLAUDE.md` (Secret `DIAGNOSE_WEBHOOK_URL` und lokale Datei in „Geheimnisse“)

- [ ] **Step 1:** Handbuch nach Spec (Knopf- und Menünamen wörtlich, fett); `node --test 'scripts/*.test.mjs'` → PASS.
- [ ] **Step 2:** Alle Prüfungen der Global Constraints → PASS.
- [ ] **Step 3:** Echter Test lokal: `MV_DIAGNOSE_WEBHOOK="$(cat ~/.tauri/mitgliederverwaltung.diagnose-webhook)" cargo test -p verwaltung_app --test diagnose -- --ignored live_discord` (ein `#[ignore]`-Test schickt eine Testnachricht „🟢 Lebenszeichen · TEST · <Version>“ mit fiktivem Bericht); Ausgabe der Variable nie anzeigen. Expected: Test PASS, User bestätigt Nachricht in Discord.
- [ ] **Step 4:** Commit `docs: Handbuch zu Problembericht und Analysedaten` und `ci: Webhook für Analysedaten im Release-Build`.
- [ ] **Step 5:** Bildschirmfotos-Workflow nach Rückfrage beim User starten (`gh workflow run bildschirmfotos.yml`), Bilder prüfen, committen.

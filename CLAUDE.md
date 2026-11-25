# CLAUDE.md – Anleitung für KI-Agenten

Mitgliederverwaltung für eine Schweizer Bäckervereinigung. Ein Desktop-Programm (Tauri 2) ersetzt die bisherige Excel-Liste.

- **Laufende Kosten:** keine. Es gibt keinen Server, alle Daten liegen in einer SQLite-Datei, lokal oder auf einem Netzlaufwerk.
- **Plattformen:** Ziel ist Windows; Linux und macOS werden mitgebaut (macOS vorerst nur zum Testen).
- **Nutzer:** Die Hauptperson ist das Sekretariat, dazu gelegentlich eine Ferienvertretung. Beide sind keine IT-Fachleute.

## Grundregeln

- **Alle Texte für Nutzer sind deutsch in Schweizer Schreibweise:** „ss“, nie „ß“, dazu Begriffe wie „Betrieb“, „GV“, „VS“. Das Handbuch spricht in der Sie-Form.
- **Privat bleibt privat:**
  - `docs/documentation/` enthält private Unterlagen des Vereins und ist in `.gitignore`. **Niemals committen** und nicht in Ausgaben zitieren.
  - **Das Repo ist öffentlich.** Es kommen keine echten Personendaten hinein. Beispieldaten sind erkennbar fiktiv.
- **Geheimnisse:**
  - Der Updater-Signierschlüssel liegt nur in `~/.tauri/mitgliederverwaltung.key`, das Passwort in `~/.tauri/mitgliederverwaltung.password`; dazu als GitHub-Secrets `TAURI_SIGNING_PRIVATE_KEY` und `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
  - Niemals ins Repo, in Logs oder in Ausgaben.
- **Nur mit ausdrücklicher Zustimmung des Users:**
  - Releases veröffentlichen (Entwurf → Publish)
  - Tags pushen bzw. `pnpm release` ausführen
  - Secrets setzen
  - Branches auf GitHub löschen
- **Pushen auf `main` ist erlaubt**, wenn der User die Arbeit freigegeben hat. Grössere Arbeiten laufen auf einem eigenen Branch und werden erst nach Rückfrage gemergt.
- **Commits** haben die Form `<type>: <Beschreibung>`. Bei `feat`, `fix` und `perf` ist die Beschreibung **deutsch und aus Nutzersicht**, denn daraus entstehen die Release Notes:
  - `feat` → „Neu“
  - `fix` → „Behoben“
  - `perf`/`refactor` → „Verbessert“

  `chore`, `ci`, `test`, `docs` und `build` erscheinen nicht in den Release Notes. Commits enden mit `Co-Authored-By: …`, wie es die Umgebung vorgibt.

## Aufbau

| Pfad | Inhalt |
|---|---|
| `crates/core` (`verwaltung_core`) | Datenkern: SQLite (rusqlite), Migrationen (`src/db/*.sql`), Repos, Filter/Abfragen, Import/Export, Sperrdatei, Sicherungen. Hängt nicht von Tauri ab. |
| `crates/core/examples/beispieldaten.rs` | Erzeugt eine Datei mit fiktiven Beispieldaten (für Bildschirmfotos). |
| `crates/app` (`verwaltung_app`) | Tauri-Programm. Die Logik liegt als normale Funktionen auf `Session` (`src/session.rs`, `src/commands/`) und ist ohne Tauri getestet; `src/tauri_commands/` enthält nur dünne Wrapper. |
| `ui/` | Svelte 5 (Runes) + TypeScript + Vite. Seiten in `src/pages`, Bausteine in `src/components`, Logik in `src/lib` (Router, Session-Stores, Hilfe, Handbuch, Markdown, Updates). |
| `docs/handbuch/` | Benutzerhandbuch: Markdown-Kapitel, `inhalt.json` (Reihenfolge), `bilder/`. Einzige Quelle für die Hilfe im Programm, das PDF und GitHub. |
| `docs/superpowers/specs`, `plans` | Design-Dokumente und Umsetzungspläne der bisherigen Arbeiten. |
| `scripts/` | Release (`release.mjs`, `version.mjs`, `notes.mjs`), Handbuch-Prüfung und PDF (`handbuch*.mjs`), Bildschirmfotos holen. |
| `e2e/` | WebdriverIO + tauri-driver. Läuft nur unter Linux (WebKitWebDriver). |
| `.github/workflows/` | `ci.yml` (Tests Linux/Windows, E2E Linux), `release.yml` (bei Tag `v*`), `bildschirmfotos.yml` (von Hand starten). |

## Befehle

Benötigt werden Rust stable, Node 22 und pnpm 10. Unter macOS liegt `cargo` in `~/.cargo/bin` (ggf. `PATH` ergänzen).

```bash
pnpm install && pnpm --dir ui install          # Abhängigkeiten
pnpm tauri dev                                 # Programm im Entwicklungsmodus (Vite auf Port 1420)

# Alle Prüfungen (dieselben wie in CI und vor jedem Release)
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir ui exec vitest run
pnpm --dir ui check                            # svelte-check
node --test 'scripts/*.test.mjs'               # Skripte + Stimmigkeit des Handbuchs

# Hilfsmittel
cargo run -p verwaltung_core --example beispieldaten -- /tmp/beispiel.sqlite
node scripts/handbuch-pdf.mjs --out /tmp/Handbuch.pdf
```

Debug-Builds öffnen beim Start die Datei aus der Umgebungsvariable `MITGLIEDERVERWALTUNG_DB` (für E2E). Release-Builds ignorieren sie.

## Arbeitsweise

- **TDD:** zuerst ein fehlschlagender Test, dann der Code. Die Abdeckung in `ui/src/lib` liegt bei mindestens 80 % (Vitest-Schwelle).
- **Immutable Daten,** kleine Dateien und Funktionen, Fehler mit verständlicher deutscher Meldung.
- **Grössere Vorhaben:** Brainstorming → Design (`docs/superpowers/specs/`) → Plan (`docs/superpowers/plans/`) → Umsetzung → Review der ganzen Arbeit. Bei Unklarheiten über Abläufe des Vereins den User fragen, nicht raten.
- **Schreiben nur mit Sperre:** Schreibende Aktionen laufen in `crates/app` über `Session::write`/`write_mut`; diese prüfen die Sperre. Die Oberfläche sperrt Änderungsknöpfe mit dem Store `readOnly`.
- **Neue Dateien bringen Vorgaben mit:** `crates/core/src/repo/defaults.rs` mit den vier Mitgliederkategorien und dem Label „provisorisch“ in der Gruppe „Aufnahme“. Bestehende Dateien werden nie automatisch verändert.

## Handbuch pflegen

Bei jeder sichtbaren Änderung an der Oberfläche gilt:

1. **Kapitel anpassen:** Knopf- und Menünamen müssen **wörtlich** mit dem Programm übereinstimmen (fett geschrieben).
2. **Neue Seite oder neuer Einstellungs-Tab:** in `ui/src/lib/handbook.ts` (`chapterForRoute`) ein Kapitel zuordnen, Kapitel anlegen und in `inhalt.json` eintragen. Der Test in `handbook.test.ts` prüft, dass jede Seite ein Kapitel hat.
3. **`node --test 'scripts/*.test.mjs'`** meldet fehlende Bilder, unbenutzte Bilder, kaputte Kapitel-Links und Dateien ohne Eintrag im Inhaltsverzeichnis.
4. **Bildschirmfotos aktualisieren:**
   - Den Workflow **Bildschirmfotos** starten: `gh workflow run bildschirmfotos.yml`.
   - Danach `node scripts/bildschirmfotos.mjs` (ersetzt alle PNGs).
   - Die Bilder ansehen und mit `docs: Bildschirmfotos aktualisieren` committen.
   - Neue Motive kommen in `e2e/specs/90-bildschirmfotos.e2e.mjs`.

## Neue Version veröffentlichen

1. Voraussetzungen: auf `main`, alles committet, gleich wie `origin/main`, und der User hat zugestimmt.
2. `pnpm release X.Y.Z` (zuerst `--dry-run`). Das Skript:
   - prüft alles und lässt alle Tests laufen
   - erhöht die Version in `crates/app/Cargo.toml`, `tauri.conf.json` und den `package.json`
   - schreibt `CHANGELOG.md` per git-cliff
   - committet `chore: Version X.Y.Z`, setzt den Tag `vX.Y.Z` und pusht atomar
3. GitHub Actions (etwa 30–40 Minuten):
   - legt einen **Entwurf** an
   - baut Windows (NSIS), Linux (AppImage, deb) und macOS (universal) nacheinander, wegen `latest.json`
   - hängt `Handbuch.pdf` an
4. **Entwurf prüfen:** 11 Dateien, darunter `latest.json` mit Version, deutschen Notes und den Plattformen `windows-x86_64`, `linux-x86_64` und `darwin-*`, sowie `Handbuch.pdf`.
5. **Veröffentlichen tut der User**, oder ein Agent nur nach ausdrücklicher Freigabe: `gh release edit vX.Y.Z --draft=false`. Erst dann bieten installierte Programme das Update an.

Semver: neue Funktionen → Minor, nur Korrekturen → Patch.

## Bekannte Grenzen

- E2E läuft nur unter Linux. Unter Windows startet msedgedriver auf GitHub-Runnern keine Sitzung.
- Windows zeigt bei der Erstinstallation SmartScreen, weil es keine kostenpflichtige Code-Signatur gibt. Apple-Signierung kommt erst, wenn macOS produktiv wird.
- Lokales Bauen des DMG unter macOS scheitert in Terminal-Sitzungen (Finder-AppleEvent). Zum Prüfen genügt `pnpm tauri build --bundles app`.

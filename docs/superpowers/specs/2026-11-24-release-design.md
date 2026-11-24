# Installer, automatische Builds und Release Notes (Plan 3) – Design

Datum: 2026-11-24 · Ergänzt `2026-11-24-mitgliederverwaltung-design.md` (§9 Qualität, §10 Build und Auslieferung).
Status: vom Auftraggeber freigegeben (inkl. Abschnitt 11).

## 1. Zweck

Die Vereinigung soll das Programm ohne Entwicklerwerkzeuge installieren und aktuell halten können,
und bei jeder neuen Version verständlich auf Deutsch sehen, was neu ist. Ohne laufende Kosten und
ohne dass der Entwickler jede Installation begleitet.

## 2. Ziele / Nicht-Ziele / Anti-Ziele

- **Ziele**
  - Installer für Windows (Hauptziel), Linux und macOS, automatisch gebaut bei jedem Release.
  - Deutsche Release Notes automatisch aus den Commits.
  - Update-Funktion im Programm: „Neue Version verfügbar – Was ist neu?" → Installieren → Neustart.
  - Im Programm jederzeit „Was ist neu?" für die installierte und frühere Versionen.
  - CI prüft jeden Push und Pull Request (Tests, clippy, Typprüfung) unter Windows und Linux.
  - E2E-Tests der Kernabläufe mit `tauri-driver` in CI (Spec §9).
- **Nicht-Ziele:** Code-Signing (kostet Geld), App-Stores, Telemetrie.
- **Anti-Ziele**
  - Ein Update darf nie Daten verändern oder verlieren. Die Datenbank liegt ausserhalb der
    Installation, Migrationen laufen wie bisher mit automatischer Sicherung beim Öffnen.
  - Kein Update ohne Zustimmung der Nutzerin. Kein Update, während die Datei schreibend offen ist,
    ohne sie vorher sauber zu schliessen (Sicherung am zweiten Ort, Sperre frei).

## 3. Rahmenbedingungen

- Repo `Zer0SkiLL/b-cker-verwaltung` ist **öffentlich**. Actions-Minuten sind damit kostenlos und
  Release-Downloads brauchen kein Login.
- Kosten 0: GitHub Actions + GitHub Releases, keine weiteren Dienste.
- Windows: Installer **unsigniert**. Beim ersten Start warnt SmartScreen („Weitere Informationen →
  Trotzdem ausführen"). Das wird im Handbuch (Schritt 3) mit Bildern erklärt.
- macOS nur zum Testen, unsigniert (Rechtsklick → Öffnen).

## 4. Ansätze

| Ansatz | Bewertung |
|---|---|
| **Tag-gesteuert: lokales Release-Skript erhöht Version + schreibt CHANGELOG, Tag-Push startet Build (`tauri-action`) mit Release Notes aus `git-cliff`** ✅ gewählt | Wenig bewegliche Teile, Release bewusst ausgelöst, Notes reproduzierbar aus Commits. |
| release-please (Release-PR automatisch, Merge = Release) | Komfortabel, aber drei Versionsdateien (Cargo, tauri.conf, package.json) brauchen Zusatz-Konfiguration; mehr Magie für ein Ein-Personen-Projekt. |
| Manuell bauen und hochladen | Widerspricht „automatisch". |

Weg zur Verteilung (vom Auftraggeber gewählt): **Repo öffentlich**, Installer und Update-Daten in
den GitHub Releases desselben Repos. Verworfen: zweites öffentliches Release-Repo (Secret + Minuten)
und Verteilung ohne Auto-Update (jedes Update über den Entwickler).

## 5. Release Notes (freigegeben)

- Quelle: Conventional Commits seit dem letzten Tag, gruppiert:
  `feat` → **Neu**, `perf`/`refactor` mit Nutzerwirkung → **Verbessert**, `fix` → **Behoben**.
  `chore`, `test`, `ci`, `docs`, `build`, `style` erscheinen nicht.
- **Ab jetzt Commit-Beschreibungen auf Deutsch und aus Nutzersicht**, z.B.
  `feat: Gruppen lassen sich umbenennen`. Ältere, englische Commits erscheinen in den Notes von 0.1.0
  so, wie sie sind **[Annahme]**. Alternative: Notes 0.1.0 von Hand zusammenfassen (wird im Plan
  so gemacht: erste Version bekommt einen handgeschriebenen Text „Erste Version").
- Werkzeug: `git-cliff` mit `cliff.toml` (deutsche Überschriften, Datum TT.MM.JJJJ).
- `CHANGELOG.md` im Repo wird vom Release-Skript fortgeschrieben und committet. Daraus zeigt das
  Programm „Was ist neu?". Der Text der GitHub-Release-Seite und der Update-Hinweis kommen aus
  demselben Abschnitt.

## 6. Ablauf eines Releases

1. Entwickler (oder Claude) ruft `scripts/release.sh 0.2.0` auf. Das Skript prüft den sauberen
   Arbeitsbaum auf `main` und grüne Tests. Dann setzt es die Version in `crates/app/Cargo.toml`,
   `crates/app/tauri.conf.json` und `ui/package.json`. Danach `git cliff --tag v0.2.0` →
   `CHANGELOG.md`, Commit `chore: Version 0.2.0`, Tag `v0.2.0`, Push von Commit und Tag.
2. Workflow `release.yml` (bei Tag `v*`):
   - Builds auf `windows-latest`, `ubuntu-22.04`, `macos-latest` mit `tauri-action`.
   - Updater-Artefakte signieren (Tauri-Updater-Schlüssel aus Secrets).
   - Ein **Entwurfs-Release** mit den Release Notes des Tags und `latest.json` für den Updater.
3. Entwickler prüft den Entwurf und klickt „Veröffentlichen". Erst dann sehen Programme das Update.
   **[Annahme]**, damit ein kaputter Build nie automatisch bei der Vereinigung landet.
4. Workflow `ci.yml` (bei Push/PR auf `main`) auf Windows und Linux:
   - `cargo test --workspace`
   - `cargo clippy -- -D warnings`
   - `vitest run`
   - `svelte-check`
   - E2E (Abschnitt 8)

## 7. Update-Funktion im Programm

- `tauri-plugin-updater` und `tauri-plugin-process`. Endpoint:
  `https://github.com/Zer0SkiLL/b-cker-verwaltung/releases/latest/download/latest.json`.
- Prüfung beim Start (im Hintergrund, Fehler wie „offline" werden still ignoriert) und über
  Einstellungen › „Nach Updates suchen".
- Gibt es ein Update: In der Hinweisleiste steht „Version 0.3.0 ist verfügbar". Daneben sind die
  Knöpfe „Was ist neu?" (zeigt die Notes) und „Jetzt installieren".
- Installieren:
  1. Datei sauber schliessen (`close_file`: zweiter Sicherungsort, Sperre frei).
  2. Herunterladen mit Fortschritt.
  3. Installieren.
  4. Neustart.

  Schlägt der Download fehl, bleibt die Datei geschlossen und es erscheint eine Meldung mit „Erneut
  versuchen" bzw. „Datei wieder öffnen".
- Windows-Installer: **NSIS, Installation pro Benutzer (ohne Administratorrechte)**, Updates im
  Modus „passive" (kurzer Fortschrittsdialog).
- **Signaturschlüssel für Updates:**
  - Er wird einmal erzeugt (`cargo tauri signer generate`).
  - Der öffentliche Schlüssel kommt in `tauri.conf.json`.
  - Der private Schlüssel und das Passwort kommen als GitHub-Secrets
    `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
  - Zusätzlich **sichert der Entwickler den privaten Schlüssel an einem sicheren Ort**. Geht er
    verloren, können installierte Programme keine Updates mehr annehmen (Neuinstallation nötig).
  - Das Setzen der Secrets erfolgt nach Rückfrage.

## 8. E2E-Tests

- WebdriverIO + `tauri-driver` gegen den Debug-Build mit einer Testdatei in einem temporären Ordner.
  Dateidialoge werden in E2E umgangen: Die App liest einen optionalen Startpfad aus der
  Umgebungsvariable `MITGLIEDERVERWALTUNG_DB` **nur in Debug-Builds**.
- Abläufe:
  1. Datei öffnen → Betrieb anlegen → in Adressen gefunden.
  2. Label vergeben → Filter → Ergebniszahl.
  3. Gruppe speichern → in Gruppen sichtbar.
  4. Zweite Instanz → „Nur lesen".
- Linux (`webkit2gtk-driver`, `xvfb`) und Windows (`msedgedriver` passend zur WebView2-Version).
  macOS hat keinen `tauri-driver` und wird manuell getestet.

## 9. „Was ist neu?" im Programm

- Einstellungen › „Über das Programm":
  - installierte Version
  - Link zum Repo
  - „Was ist neu?" (gerendertes `CHANGELOG.md`, beim Build eingebettet, kein Netz nötig)
  - „Nach Updates suchen"
- Markdown-Darstellung mit einer kleinen, geprüften Bibliothek (`marked`) und bereinigtem HTML
  (`DOMPurify`). Dieselbe Darstellung nutzt später die In-App-Hilfe (Schritt 3).

## 10. Dem Umsetzer überlassen

Workflow-Details (Caching, Matrix), genaue `cliff.toml`-Vorlage, Aufbau der E2E-Tests,
Gestaltung des Update-Hinweises im Rahmen der bestehenden Hinweisleiste.

## 11. Entscheidungen nach Builder-Check (Claudes Entscheidungen, korrigierbar)

1. **Installer-Formate:**
   - Windows: **nur NSIS** (pro Benutzer), kein MSI.
   - Linux: **AppImage** (mit Updater) + **deb**.
   - macOS: **dmg** + Updater-Archiv `.app.tar.gz`, vorerst zum Testen, **potentiell später als produktive App** (Auftraggeber). Dafür bleibt der Weg offen: Signieren/Notarisieren braucht ein Apple-Entwicklerkonto (kostenpflichtig) und wird dann nur im Release-Workflow ergänzt.
   - `bundle.createUpdaterArtifacts: true`, `latest.json` mit Einträgen für alle drei Plattformen.
2. **Erste Version 0.1.0** ist eine echte, installierbare Version zum Testen unter Windows.
   - Ihr `CHANGELOG.md`-Abschnitt wird von Hand geschrieben („Erste Version" mit Funktionsüberblick).
   - Der Tag `v0.1.0` wird im Plan direkt gesetzt.
   - `release.sh` ist für alle Folgeversionen. Es verweigert eine Version, die nicht grösser ist
     als die aktuelle.
   - Die Update-Funktion wird mit einer kleinen Folgeversion 0.1.1 nachgewiesen.
3. **Reihenfolge:**
   1. CI (Tests)
   2. Release-Workflow und Notes
   3. Updater und „Über das Programm"
   4. E2E zuletzt

   `release.yml` wartet nicht auf E2E. `release.sh` führt lokal `cargo test`, `vitest`, `clippy` und
   `svelte-check` aus, bevor es taggt. E2E unter Windows gilt als Ziel. Wird es instabil, läuft es
   vorerst nur unter Linux, und unter Windows wird manuell geprüft (wird dann als Entscheidung gemeldet).
4. **Direkter Push auf `main`** durch `release.sh` ist erlaubt (keine Branch-Protection, Auftraggeber
   pusht selbst auf `main`).
5. **Update bei mehreren Instanzen:**
   - Der NSIS-Installer beendet laufende Instanzen auf demselben PC. Das ist unkritisch, weil eine
     zweite Instanz nur lesend offen ist.
   - Andere PCs haben ihre eigene Installation und sind nicht betroffen.
   - Ist der zweite Sicherungsort beim Schliessen nicht erreichbar, ist das **nur ein Hinweis**.
     Das Update läuft weiter, denn die lokale Sicherung beim Öffnen existiert.
6. **Secrets und Repo-Einstellungen:**
   - Claude erzeugt den Schlüssel lokal und setzt die Secrets mit `gh secret set`, **erst nach
     ausdrücklicher Zustimmung**.
   - Der Auftraggeber legt eine Kopie des privaten Schlüssels und des Passworts an einem sicheren
     Ort ab.
   - Workflows bekommen `permissions: contents: write`.
   - Es gibt noch keine Vorinstallationen, deshalb ist der Upgrade-Pfad kein Thema.

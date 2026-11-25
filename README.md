# Mitgliederverwaltung Bäckervereinigung

Desktop-Programm für die Mitgliederverwaltung, als Ersatz für die bisherige Excel-Liste.
Läuft unter Windows, Linux und macOS. Es braucht keinen Server, und es fallen keine
Betriebskosten an. Alle Daten liegen in **einer Datei** (`.sqlite`), lokal oder auf einem
Netzlaufwerk.

## Was es kann

- **Betriebe und Personen:** Betriebe sind juristische Personen, Personen sind natürliche
  Personen. Eine Person kann mit mehreren Betrieben verknüpft sein, mit Funktion und Von/Bis.
- **Mitgliedschaften** für Betriebe oder Einzelpersonen, mit Kategorie, Eintritt und Austritt.
- **Selbst pflegbare Labels und Listen,** z.B. Prüfungsexperte, Vorstand, Lehrbetrieb,
  ÜK-Zentrum, Funktionen und Kantonalverbände.
- **Teams** für Themen, mit Rollen.
- **Suchen und Filtern** in einer gemeinsamen Adressliste. Häufige Filter lassen sich als
  „Gruppe“ speichern.
- **Kontaktieren:**
  - Mail-Adressen kopieren (direkt in Outlook-BCC einfügbar)
  - Mailprogramm öffnen
  - Excel/CSV für Serienbriefe
- **Excel-Import** der bisherigen Liste, mit Probelauf und Bericht.
- **Gesamtexport** aller Daten als Excel-Datei.

## Datensicherung

1. **Automatisch beim Öffnen:** Das Programm sichert die Datei in den Ordner `backups/`
   daneben. Die letzten 20 Sicherungen bleiben erhalten.
2. **Zweiter Sicherungsort,** z.B. ein USB-Stick oder ein anderes Laufwerk. Er wird unter
   Einstellungen › Datensicherung eingerichtet. Gesichert wird dort beim Schliessen und mit
   „Jetzt sichern“. Ist die letzte Sicherung dort älter als 30 Tage, erscheint ein Hinweis.
3. **Gesamtexport als Excel,** zur Archivierung. Die Datei ist auch ohne dieses Programm lesbar.

**Wiederherstellen** geht unter Einstellungen › Datensicherung. Der aktuelle Stand wird
vorher gesichert.

**Zwei Personen gleichzeitig:** Wer die Datei als Zweite öffnet, kann nur lesen. Die gelbe
Leiste zeigt, wer sie geöffnet hat. Mit „Übernehmen“ kann man die Datei bei Bedarf
übernehmen, z.B. wenn das Programm vorher abgestürzt ist.

## Neue Version veröffentlichen

1. Hat sich die Oberfläche sichtbar geändert? Workflow **Bildschirmfotos** starten
   (`gh workflow run bildschirmfotos.yml`), danach `node scripts/bildschirmfotos.mjs` und die Bilder
   mit `docs: Bildschirmfotos aktualisieren` committen. Das Handbuch liegt in `docs/handbuch/`.
2. Commits auf Deutsch und aus Nutzersicht schreiben, z.B. `feat: Gruppen lassen sich umbenennen`
   oder `fix: Mail öffnen funktioniert wieder`. Daraus entstehen die Release Notes
   („Neu", „Verbessert", „Behoben"). `chore`, `ci`, `test`, `docs` erscheinen dort nicht.
3. Auf `main` mit sauberem Arbeitsbaum: `pnpm release 0.2.0` (Probelauf: `pnpm release 0.2.0 --dry-run`).
   Das Skript prüft alles, erhöht die Version, schreibt `CHANGELOG.md`, committet, taggt und pusht.
4. GitHub baut die Installer (ca. 30–40 Minuten) und legt einen **Entwurf** an:
   https://github.com/Zer0SkiLL/b-cker-verwaltung/releases
5. Entwurf prüfen (Dateien für Windows, Linux, macOS, `latest.json` und `Handbuch.pdf`) und **veröffentlichen**.
   Erst dann zeigen installierte Programme „Version … ist verfügbar".

Der Signaturschlüssel für Updates liegt in `~/.tauri/mitgliederverwaltung.key` (Passwort in
`~/.tauri/mitgliederverwaltung.password`) und als GitHub-Secrets. Beides gut sichern: ohne den
Schlüssel nehmen installierte Programme keine Updates mehr an.

Windows-Installer sind nicht signiert: beim ersten Start „Weitere Informationen → Trotzdem ausführen".

## Entwicklung

Voraussetzungen:
- Rust (stable)
- Node 22 und pnpm
- Tauri-CLI: `cargo install tauri-cli --version "^2" --locked`

```bash
pnpm --dir ui install
cd crates/app && cargo tauri dev     # Programm im Entwicklungsmodus starten
```

### Tests

```bash
cargo test --workspace               # Datenkern und Befehle
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir ui exec vitest run        # Oberflächen-Logik und Komponenten
pnpm --dir ui check                  # Typprüfung
pnpm --dir ui coverage               # Abdeckung (Schwelle 80 % für ui/src/lib)
cargo llvm-cov -p verwaltung_app --ignore-filename-regex 'tauri_commands|main\.rs|crates/core' --summary-only
```

### Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/core` | Datenkern: Datenbank, Regeln, Suche, Import/Export, Sperre, Sicherungen |
| `crates/app` | Tauri-Programm. Die Befehle sind ohne Tauri testbar (`src/commands`). `src/tauri_commands` reicht sie nur weiter. |
| `ui` | Oberfläche (Svelte 5 + TypeScript) |
| `docs/superpowers` | Design-Dokumente und Umsetzungspläne |

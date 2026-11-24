# Oberfläche (Plan 2) – Design

Datum: 2026-11-24 · Ergänzt `2026-11-24-mitgliederverwaltung-design.md` (Abschnitt 11, „UI-Layout offen").
Status: vom Auftraggeber freigegeben („passt so").

## Zweck

Der Datenkern (`verwaltung_core`) ist fertig, aber ohne Oberfläche nicht benutzbar. Dieser Schritt
macht daraus ein Desktop-Programm (Tauri 2 + Svelte + TypeScript), mit dem die Excel-Liste im
Alltag ersetzt werden kann: suchen, filtern, kontaktieren, exportieren, erfassen, sichern, importieren.

## Ziele / Nicht-Ziele

- Ziel: alle Kernabläufe aus Spec §6 und das Backup-Konzept §8 sind per Oberfläche bedienbar.
- Nicht-Ziel: Mailvorlagen, Terminierung (Nice to have, später), Installer/CI (Plan 3),
  Mitgliederbeiträge.
- Anti-Ziel: Fachlogik im Frontend. Die Tauri-Schicht ist dünn; Regeln bleiben im Core und
  bleiben dort getestet.

## Ansätze

| Ansatz | Bewertung |
|---|---|
| **Eine Hauptliste „Adressen" (Betriebe + Personen gemischt, Typ-Umschalter)** ✅ gewählt | Entspricht `search()` im Core und dem Hauptzweck „Gruppen kontaktieren"; ein Ort für Suchen, Filtern, Kontaktieren, Export. |
| Getrennte Listen Organisationen / Personen | Näher an Datenmodell, aber Kontaktieren über beide Typen braucht zwei Wege. |
| Dashboard-Startseite | Kein Mehrwert für eine Person mit Liste als Hauptwerkzeug. |

## Layout (freigegeben)

```
┌──────────────┬───────────────────────────────────────────────┐
│ Bäckerverein │ [Hinweisleiste: Nur lesen / Sicherung alt]    │
│ ▸ Adressen   │ Suche · Typ (Alle|Betriebe|Personen)          │
│   Teams      │ Filter: Labels, Funktion, Kategorie, Status, Kt│
│   Gruppen    │ Ergebnistabelle (Typ, Name, Betrieb, Ort, Mail)│
│ Einstellungen│ N Einträge · M ohne Mail                      │
│              │ [Mails kopieren][Mail öffnen][Excel][CSV][Als Gruppe speichern] │
└──────────────┴───────────────────────────────────────────────┘
```

- **Adressen**: Hauptansicht. Klick auf Zeile → Detailansicht; „Zurück" behält die Filter.
- **Detail Betrieb**: Stammdaten, verknüpfte Personen (Funktion, Von/Bis, frühere ein-/ausblendbar),
  Labels, Teams, Mitgliedschaften. Bearbeiten, Person verknüpfen/neu anlegen, löschen (mit Rückfrage).
- **Detail Person**: Stammdaten inkl. Ausbildungsbetrieb, Lehrgänge, Prüfungsexperte, Vorstand,
  ÜK-Zentrum, Honorar/Lohn (Infotext) – so wie im Core-Modell vorhanden; Betriebe, Labels, Teams,
  Mitgliedschaft.
- **Teams**: Liste der Teams, Mitglieder mit Rolle, hinzufügen/entfernen.
- **Gruppen**: gespeicherte Filter; Klick öffnet „Adressen" mit diesem Filter; umbenennen/löschen.
- **Einstellungen**: Labels & Label-Gruppen, Auswahllisten, Datensicherung (Status, Liste,
  zweiter Ort, „Jetzt sichern", Wiederherstellen), Excel-Import (Assistent), Gesamtexport.
- **Start**: Datei wählen oder neu anlegen; Pfad wird gemerkt (App-Konfigurationsordner).
- **Hinweisleiste**: nur lesend (wer, seit wann, „Übernehmen" bei Rückfrage); Erinnerung
  zweiter Sicherungsort (> 30 Tage oder nicht eingerichtet).

## Verbindliche Vorgaben aus dem Datenkern

- Vor jedem Schreibbefehl `Workspace::check_lock()`; bei Sperrverlust Wechsel auf „nur lesen"
  und Hinweis statt Schreiben.
- Wiederherstellen über `restore(&mut self, …)` auf der bestehenden Sitzung.
- Import-Zuordnung positionsbasiert (Spalte i ↔ Eintrag i), Standard aus `default_for`.
- Beim Beenden `Workspace::close()` (zweiter Sicherungsort).
- Fehlertexte aus `CoreError` (Deutsch) werden unverändert angezeigt.
- Datum TT.MM.JJJJ in Anzeige und Eingabe.

## Dem Umsetzer überlassen

Komponentenaufteilung, Styling (schlicht, gut lesbar, hell), Tabellen-Komponente, Routing-Lösung,
Form der Dialoge. Tests: Rust-Befehlsschicht mit Integrationstests, Svelte-Komponenten/Logik mit
Vitest; E2E mit `tauri-driver` folgt in Plan 3 (CI).

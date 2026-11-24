# Mitgliederverwaltung Bäckervereinigung – Design

Stand: 2026-11-24 · Status: Entwurf zur Freigabe

Markierung: **[Annahme]** = Entscheid von Claude, vom Auftraggeber noch nicht explizit bestätigt.

## 1. Zweck und Hintergrund

Die Bäckervereinigung führt ihre Mitglieder heute in einer einzigen Excel-Tabelle. Darin sind
Betriebe (juristische Personen) und Personen (natürliche Personen) vermischt, Funktionen und
Gruppen stecken in Einzelspalten. Gezieltes Anschreiben von Personengruppen (z.B. alle
Prüfungsexperten, alle Lehrbetriebe einer Fachrichtung) ist mühsam.

Ziel ist eine **vollständige Mitgliederverwaltung, die die Excel-Tabelle ablöst**: saubere
Trennung von Betrieben und Personen, frei pflegbare Kategorien/Labels, Teams, Ausbildungsdaten,
gezieltes Filtern und Kontaktieren sowie Export.

Nutzer: heute 1 Person, künftig evtl. eine zweite (Ferienvertretung), nie gleichzeitig.
Sprache der Oberfläche: Deutsch.

## 2. Ziele, Nicht-Ziele, Anti-Ziele

**Ziele (Version 1)**
- Betriebe und Personen getrennt erfassen, Personen mit Betrieben verknüpfen (mit Funktion).
- Funktionen, Kategorien und Labels dynamisch selbst verwalten (ohne Programmierung).
- Teams zu Themen bilden.
- Personendaten: Ausbildungsbetrieb, Lehrgänge, Prüfungsexperte, Vorstand, ÜK-Zentrum, Honorar/Lohn.
- Filtern nach beliebigen Kombinationen und Gruppen kontaktieren (E-Mail-Adressen, Adresslisten).
- Export (Excel/CSV) jeder gefilterten Liste.
- Einmaliger Import der bestehenden Excel-Tabelle.

**Nice to have (Version 2)**
- Mailvorlagen.
- Terminierung/Anlässe (z.B. Generalversammlung, Prüfungstermine).

**Nicht-Ziele**
- Lohnbuchhaltung (Lohnausweis, AHV). Honorar/Lohn ist nur ein Info-Feld.
- Erfassung einzelner Einsätze mit Beträgen (später möglich, Modell bleibt offen dafür).
- Cloud-Hosting, Server, Benutzerkonten.
- Gleichzeitiges Arbeiten mehrerer Personen.
- Eigener Mailversand (das Programm übergibt an das bestehende Mailprogramm).

**Anti-Ziele (wäre ein Fehlschlag, selbst wenn es technisch funktioniert)**
- Datenverlust oder unbemerkt überschriebene Daten.
- Tägliche Arbeit ist umständlicher als in Excel.
- Laufende Kosten oder Wartungsaufwand für Infrastruktur.

## 3. Rahmenbedingungen

- **Kosten: 0** für Betrieb und Wartung. Kein Server, keine Lizenzen, keine Abos.
- **Datenablage lokal**: auf dem Rechner, einem Netzlaufwerk oder in einem geteilten Ordner.
- **Plattformen**: Windows (Hauptziel), Linux (künftig), macOS (zum Testen durch den Entwickler).
- Keine Code-Signierung (kostet). Einmalige Warnung (Windows SmartScreen / macOS „Trotzdem öffnen") wird akzeptiert.
- Personendaten inkl. Honorarangaben sind schützenswert (Schweizer DSG). **[Annahme]**: Verein ist in der Schweiz (Kanton, ÜK, CHF).

## 4. Lösungsansätze

| Ansatz | Beschreibung | Abwägung |
|---|---|---|
| **A) Desktop-App Tauri (Rust) + SQLite-Datei** ✅ gewählt | Natives Programm für Win/Linux/macOS, Daten in einer Datei, frei ablegbar | 0 Kosten, kein Server, Backup = Datei kopieren. Kein gleichzeitiges Arbeiten (Dateisperre). |
| B) Lokaler Rust-Webserver + Browser-UI | Ein PC hostet, Zugriff per Browser im Netzwerk | Gleichzeitiges Arbeiten möglich, aber Host-PC muss laufen, weniger „normales Programm". |
| Verworfen: Gehostete Web-App (Gratis-Tiers) | Supabase/Vercel o.ä. | Gratis-Tiers ändern sich/pausieren, Personendaten in der Cloud. Widerspricht 0 Kosten/0 Wartung. |
| Verworfen: Access/Airtable | Low-Code | Lizenzkosten bzw. Limits, Anbieterbindung. |

**Gewählt: A** durch den Auftraggeber. Der Rust-Kern wird so geschnitten, dass ein späterer
Umbau zu B ohne Neuschreiben der Fachlogik möglich ist.

## 5. Fachliches Datenmodell

```
Organisation ──< Zugehörigkeit >── Person
     │            (Funktion, von, bis)      │
     │                                      │
     └──< Mitgliedschaft >──────────────────┘   (Inhaber ist Organisation ODER Person)
     │                                      │
     └──< Label-Zuweisung >─────────────────┘   (Label aus Label-Gruppe, opt. Datum/Notiz)
                                            │
                                 Team ──< Teammitglied >── Person
```

### 5.1 Organisation (juristische Person)
Felder aus der Excel-Tabelle: Name/Firma, Zusatz, Adresse, Postfach, PLZ, Ort, Telefon, Mail,
Website („Home"), Kantonalverband („Kt Verband"), Filialen, Anzahl Lernende („Anzahl"),
Bemerkungen („Besonderes").

- **Typ** (frei pflegbare Liste): z.B. Mitgliedsbetrieb, ÜK-Zentrum, Andere. **[Annahme]**:
  ÜK-Zentren werden als Organisationen geführt, Personen werden ihnen per Zugehörigkeit
  (z.B. Funktion „Kursleiter") zugeordnet.
- **Lehrbetrieb** und Fachrichtungen (Fr Bäck, Fr Conf, DH) werden als Labels abgebildet
  (Label-Gruppe „Ausbildung Fachrichtung"). „Lehrbetrieb" = hat mindestens ein solches Label.
  **[Annahme]**: Lernende werden nicht einzeln erfasst, nur die Anzahl pro Betrieb.

### 5.2 Person (natürliche Person)
Anrede, Vorname, Nachname, Adresse (optional, privat), Telefon, Mobiltelefon, Mail,
Honorar/Lohn (Freitext, z.B. „CHF 300 / Prüfungstag"), Bemerkungen.
- Personen können **ohne Organisation** existieren (z.B. pensionierte Prüfungsexperten).

### 5.3 Zugehörigkeit (Person ↔ Organisation)
- Funktion im Betrieb (frei pflegbare Liste: Inhaber, Mitarbeiter, Berufsbildner, Kursleiter …).
- Von-/Bis-Datum. Ohne Bis-Datum = aktuell. Ansichten zeigen standardmässig nur aktuelle,
  die Historie bleibt einsehbar.
- Eine Person kann mehrere Zugehörigkeiten haben (verschiedene Betriebe/Funktionen).

### 5.4 Mitgliedschaft
- Inhaber ist eine Organisation **oder** eine Person (für Einzel-/Ehrenmitglieder).
- Kategorie (frei pflegbar, Excel-Spalte „Kategorie"), Eintritt, Austritt.
- Aktiv = kein Austrittsdatum oder Austritt in der Zukunft.

### 5.5 Labels und Label-Gruppen (dynamisch)
- Label-Gruppen werden vom Nutzer angelegt, z.B. „Verbandsfunktion" (Vorstand, Prüfungsexperte),
  „Lehrgang" (Berufsbildnerkurs, Berufsprüfung …), „Ausbildung Fachrichtung".
- Pro Gruppe einstellbar: gilt für Personen, Organisationen oder beide.
- Eine Label-Zuweisung kann optional **Datum** und **Notiz** tragen (z.B. Lehrgang abgeschlossen 2019).
- Labels umbenennen wirkt überall. Löschen nur mit Rückfrage, zeigt die Anzahl betroffener Einträge.

### 5.6 Teams
- Name, Thema/Beschreibung, Mitglieder (Personen), optional Rolle im Team (z.B. Leitung).

### 5.7 Verwaltbare Auswahllisten
Organisationstypen, Funktionen im Betrieb, Mitgliedschaftskategorien, Kantonalverbände,
Label-Gruppen und Labels. Alle in einem Bereich „Einstellungen" pflegbar.

## 6. Kernabläufe

1. **Suchen & Filtern**: Volltextsuche (Name, Ort, Mail) plus kombinierbare Filter: Labels,
   Teams, Funktion im Betrieb, Organisationstyp, Mitgliedschaftskategorie, aktiv/ausgetreten,
   Kantonalverband. Filter lassen sich als „gespeicherte Gruppe" ablegen (z.B. „Alle Prüfungsexperten").
2. **Kontaktieren**: Aus jeder Ergebnisliste:
   - E-Mail-Adressen kopieren (mit Semikolon getrennt, direkt in Outlook-BCC einfügbar),
   - Mailprogramm öffnen (`mailto:` mit BCC, falls die Länge es zulässt, sonst Hinweis auf Kopieren),
   - Adressliste als Excel für Word-Serienbriefe exportieren.
   - Einträge ohne Mail-Adresse werden ausgewiesen.
3. **Export**: Jede Liste als XLSX und CSV (UTF-8, für Excel kompatibel).
4. **Excel-Import (einmalig)**: Assistent mit Spaltenzuordnung, Probelauf und Vorschau. Unklare
   Zeilen landen in einem Import-Bericht zur manuellen Nachbearbeitung. Der Import läuft nur in
   eine leere Datenbank und ist damit beliebig wiederholbar. Die Zuordnung ist im Assistenten
   anpassbar, damit eine falsche Annahme ohne Programmänderung korrigiert werden kann.
   Standard-Zuordnung **[Annahme]**, bis eine anonymisierte Beispieldatei vorliegt:
   - Eine Zeile = ein Eintrag. Erste Zeile = Kopfzeile, erstes Tabellenblatt.
   - **Vorname leer** → Organisation (Name/Firma, Zusatz).
   - **Vorname gefüllt** → Person (Nachname = „Name/Firma", Vorname) **plus** Organisation
     (Name = „Zusatz", falls gefüllt, sonst „Vorname Name") **plus** Zugehörigkeit „Inhaber".
   - Adresse, PLZ, Ort, Postfach, Telefon, Mail, Home, Filialen → Organisation.
     Mobiltelefon → Person, falls vorhanden, sonst Organisation.
   - ID1 → „Alt-ID" (Referenz auf die alte Liste, nur zur Nachverfolgung).
   - Kategorie, Eintritt, Austritt → Mitgliedschaft der Organisation.
   - Kt Verband → Kantonalverband (fehlende Listeneinträge werden automatisch angelegt).
   - Anzahl → Anzahl Lernende (nur ganze Zahlen, sonst Bericht).
   - Lehrbetrieb, Fr Bäck, Fr Conf, DH: Werte wie „x", „ja", „1", „true" → entsprechendes Label
     der Gruppe „Ausbildung Fachrichtung". Leer oder „nein" → kein Label. Alles andere → Bericht.
   - Besonderes → Bemerkungen.
   - Datumswerte: Excel-Datum oder TT.MM.JJJJ. Nur ein Jahr → 01.01. des Jahres plus Hinweis im Bericht.
   - Doppelte Firmennamen werden nicht automatisch zusammengeführt, sondern im Bericht gemeldet.
5. **Erfassen/Bearbeiten**: Detailansicht Organisation (mit verknüpften Personen) und
   Person (mit Betrieben, Labels, Teams, Mitgliedschaft).

## 7. Technische Entscheidungen

- **Tauri 2**, Fachlogik und Datenzugriff in **Rust**.
- **Frontend**: Svelte + TypeScript **[Annahme]** (leichtgewichtig, wenig Boilerplate).
- **Datenbank**: SQLite (eingebettet, `rusqlite` mit `bundled`), eine `.sqlite`-Datei.
  - Journal-Modus `DELETE`, **kein WAL** (WAL funktioniert auf Netzlaufwerken nicht zuverlässig).
  - Versionierte Schema-Migrationen im Programm. Vor jeder Migration automatisches Backup.
- **Datei wählen**: Beim ersten Start Speicherort wählen bzw. bestehende Datei öffnen. Pfad wird gemerkt.
- **Dateisperre**: Beim Öffnen wird neben der Datenbank eine Sperrdatei angelegt (Benutzer, Rechner, Zeit).
  Ist die Datei gesperrt, öffnet das Programm **nur lesend** und zeigt, wer sie geöffnet hat.
  Veraltete Sperren (z.B. nach Absturz) können nach Rückfrage übernommen werden.
- **Backups**: Beim Öffnen automatische Kopie in einen Backup-Ordner neben der Datei. Es werden die
  letzten 20 behalten **[Annahme]**.
- **Excel**: Import mit `calamine`, Export mit `rust_xlsxwriter`.
- **Schweizer Konventionen** **[Annahme]**: Datum TT.MM.JJJJ, „ss" statt „ß", CHF.
- **Architektur**: Rust-Workspace mit getrennter Fachlogik-Crate (Domain, Repository, Import/Export)
  und dünner Tauri-Schicht. So bleibt Variante B später möglich und die Logik ist ohne UI testbar.
- **Keine Verschlüsselung der Datei in V1** **[Annahme]**. Schutz über die Zugriffsrechte des
  Ablageorts. Kann später ergänzt werden (SQLCipher).

## 8. Qualität und Tests

- Ziel 80 % Testabdeckung.
- Unit-Tests für die Fachlogik (Rust) und UI-Komponenten (Vitest).
- Integrationstests gegen temporäre SQLite-Dateien (Migrationen, Repository, Import/Export, Sperre).
- E2E-Tests der Kernabläufe (Erfassen, Filtern, Export) mit `tauri-driver`/WebdriverIO in CI auf
  Windows und Linux. `tauri-driver` unterstützt macOS nicht, dort wird manuell getestet.

## 9. Build und Auslieferung

- GitHub-Repo: `Zer0SkiLL/b-cker-verwaltung`.
- GitHub Actions (`tauri-action`) baut Installer für Windows (MSI/NSIS), Linux (AppImage/deb) und macOS (dmg).
- Auslieferung über GitHub Releases, Update durch neuen Download. Ein Auto-Updater ist optional für später.
- Unsigniert, Warnung beim ersten Start wird dokumentiert.

## 10. Für die Umsetzung offen gelassen

- Konkretes UI-Layout und Navigation (Vorschlag: Seitenleiste Organisationen / Personen / Teams /
  Gruppen / Einstellungen, Liste + Detailansicht).
- Import-Zuordnung anhand der anonymisierten Beispieldatei überprüfen, sobald sie vorliegt.
  Bis dahin gilt die Standard-Zuordnung aus Abschnitt 6.
- Bedeutung von „Anzahl" und „Kt Verband" bestätigen (aktuelle Lesart: Anzahl Lernende, Kantonalverband).
- **Mitgliederbeiträge/Rechnungen** **[Annahme]**: nicht in Version 1. Die Mitgliedschaft ist eine
  eigene Tabelle, an die später Beitragssätze und Zahlungsstatus angehängt werden können.

# Änderungen

Hier steht, was in jeder Version neu ist. Die neueste Version steht oben.

## [0.5.0] – 26.11.2026

### Neu

- Der Ofen ist jetzt richtig vorgeheizt

## [0.4.0] – 26.11.2026

### Neu

- Adressliste per Klick auf die Spaltenköpfe sortieren
- Bilder in der Hilfe per Klick vergrössern, Hilfe breiter oder schmaler stellen

## [0.3.2] – 26.11.2026

### Behoben

- Import-Seite übersichtlicher: Spaltenzuordnung und Bericht eingeklappt, Übernehmen oben, kein Scrollen in Teilbereichen, Hinweise nicht doppelt

## [0.3.1] – 25.11.2026

### Behoben

- Abgebrochene Sicherungen hinterlassen keine halben Dateien, und schreibgeschützte Sicherungen lassen sich wiederherstellen
- Hilfe zeigt bei „Zurück“ den Suchbegriff wieder, und Esc im Suchfeld leert zuerst die Suche
- Wird die Datei beim Umbenennen nur lesend, verschwindet die Eingabe

## [0.3.0] – 25.11.2026

### Neu

- Vor dem Update fragt das Programm, ob alle Änderungen gespeichert sind
- Betriebe haben Felder für Handelsregister-Nr und Kostenstellen-Nr
- Excel-Import versteht Firma, Personen und Einzelmitglieder der Mitgliederliste
- Import-Seite mit Zusammenfassung, Filter, Rückfrage und Rückgängig
- Sicherungen werden nach Alter ausgedünnt (einstellbar)
- Sicherung während der Arbeit und sofortiger Hinweis, wenn der zweite Sicherungsort fehlt
- Das Programm führt ein Protokoll für Problemberichte
- Problem melden erstellt einen Bericht zum Kopieren

### Behoben

- Erneuter Klick auf „?“ oder „Hilfe“ schliesst die Hilfe wieder
- Datensicherung beschreibt die neue Aufbewahrung statt „die letzten 20“
- Import verliert bei Einzelmitgliedern keine Angaben und zeigt alle Hinweise; Hinweis zum zweiten Sicherungsort verschwindet zuverlässig
- Nach Wiederherstellen, Übernehmen oder „Jetzt sichern“ entsteht keine überflüssige Sicherung mehr
- Pausen oder eine falsch gehende Uhr löschen keine älteren Sicherungen mehr
- Import nennt bei abweichenden Angaben beide Werte und übernimmt doppelte Mandate nur einmal
- Import rückgängig machen nimmt nur die Sicherung dieses Imports; Öffnen und Schliessen stehen im Protokoll

## [0.2.0] – 25.11.2026

### Neu

- Neue Dateien enthalten die Mitgliederkategorien und das Label „provisorisch“
- Handbuch-Kapitel werden ins Programm eingebunden
- Hilfe-Seitenleiste mit Inhaltsverzeichnis, Suche und „?“ auf jeder Seite
- Hinweise bei leeren Listen und Tooltips erklären den nächsten Schritt
- Handbuch als PDF bei jedem Release

### Behoben

- Suche im Handbuch findet auch Wortteile wie „eintritt“ in „Neueintritt“

## [0.1.1] – 24.11.2026

### Neu

- Link zum Projekt unter „Über das Programm“ öffnet sich im Browser

## [0.1.0] – 24.11.2026

Erste Version der Mitgliederverwaltung als Ersatz für die Excel-Liste.

### Neu

- Betriebe und Personen erfassen, Personen mit Funktion und Zeitraum mit Betrieben verknüpfen
- Mitgliedschaften für Betriebe und Einzelpersonen (Kategorie, Eintritt, Austritt)
- Selbst pflegbare Labels (z.B. Prüfungsexperte, Vorstand, Lehrbetrieb) und Auswahllisten
- Teams für Themen, mit Rollen
- Adressliste mit Suche und Filtern; Filter als Gruppe speichern
- Mails kopieren (für Outlook-BCC), Mailprogramm öffnen, Excel/CSV für Serienbriefe
- Excel-Import der bisherigen Mitgliederliste mit Probelauf und Bericht
- Gesamtexport aller Daten als Excel-Datei
- Automatische Sicherung beim Öffnen, zweiter Sicherungsort, Wiederherstellen
- Gemeinsames Arbeiten: Wer die Datei als Zweite öffnet, kann nur lesen oder übernehmen

# Handbuch und Hilfe im Programm – Design

Stand: 25.11.2026 · Status: Entwurf, wartet auf Freigabe durch den User

## 1. Absicht und Warum

Das Programm wird im Alltag von **einer Hauptperson** (Sekretariat) bedient. Ab und zu, etwa in den Ferien, springt **eine Vertretung** ein. Diese Person kennt das Programm kaum und braucht es vielleicht zwei Wochen im Jahr.

Ziel des Users: **Die Nutzer können sich jederzeit selbst helfen, und das Programm ist selbsterklärend.** Niemand soll anrufen müssen, um zu erfahren, wie man ein Mitglied aufnimmt oder eine Sicherung zurückspielt.

Daraus folgt:
- Die Hilfe ist **im Programm**, und zwar genau dort, wo man gerade arbeitet.
- Das Handbuch ist **nach den Abläufen des Vereins** geschrieben, nicht nach Menüs.
- Es gibt zusätzlich ein **PDF zum Ausdrucken**.

## 2. Ziele, Nicht-Ziele, Anti-Ziele

**Ziele**
- Ein deutsches Handbuch als Markdown in `docs/handbuch/`. Es ist die einzige Quelle für die Hilfe im Programm, das PDF und die Ansicht auf GitHub.
- Auf jeder Seite des Programms ein „?“. Es öffnet rechts eine **Hilfe-Seitenleiste** mit dem passenden Kapitel. Die Arbeit bleibt daneben sichtbar und bedienbar.
- In der Seitenleiste gibt es ein Inhaltsverzeichnis, eine Volltextsuche und Links zwischen Kapiteln.
- **Hinweise direkt auf den Seiten**: Leere Listen erklären den nächsten Schritt, Knöpfe ohne eindeutige Beschriftung bekommen Tooltips.
- **Bildschirmfotos**, die automatisch mit fiktiven Beispieldaten erzeugt werden und damit aktuell bleiben.
- Das **PDF** wird bei jedem Release erzeugt und ans GitHub-Release gehängt. Im Programm öffnet der Knopf „Handbuch als PDF“ die Datei zur installierten Version.

**Nicht-Ziele**
- Mailvorlagen und Terminierung. Diese Funktionen kommen später, mit eigenem Handbuch-Kapitel.
- Ein Online-Hilfeportal oder eine Webseite.
- Mehrsprachigkeit.
- Kontextsensitive Hilfe pro Eingabefeld. Tooltips gibt es nur dort, wo eine Beschriftung allein nicht reicht.

**Anti-Ziele** (wäre ein Misserfolg, auch wenn es technisch funktioniert)
- Die Hilfe braucht Internet. Ausnahme ist nur der PDF-Download.
- Handbuch und Programm laufen auseinander: veraltete Bilder, Knöpfe, die anders heissen als im Text.
- Fachjargon. Leserin ist eine Vertretung, die selten am PC arbeitet. Gemeint sind Wörter wie „Datensatz“, „Entität“ oder „Filter-Query“.
- Echte Personendaten in Bildern oder Texten. Das Repo ist öffentlich.

## 3. Rahmenbedingungen

- Laufende Kosten: keine. Alles läuft lokal oder in GitHub Actions (öffentliches Repo).
- Die Oberfläche bleibt unverändert in Svelte 5 mit Runes und TypeScript. Markdown wird wie bisher mit `marked` und `DOMPurify` dargestellt (`ui/src/lib/markdown.ts`).
- Programmtexte sind deutsch in Schweizer Schreibweise: „ss“ statt „ß“, „Betrieb“, „GV“, „VS“.
- E2E-Tests laufen nur unter Linux (WebKitWebDriver und xvfb). Das gilt auch für die Bildschirmfotos.
- Für das Repo gilt weiterhin TDD, Abdeckung ab 80 % in `ui/src/lib`, CI auf Linux und Windows.

## 4. Vorgehen (Approaches)

Erwogene Wege für die Hilfe im Programm:
- **A) Seitenleiste rechts. Gewählt vom User.** Lesen und gleichzeitig klicken geht. Das ist für seltene Nutzer am hilfreichsten.
- **B) Eigene Hilfe-Seite.** Einfacher zu bauen, aber man muss zwischen Hilfe und Arbeit hin- und herspringen.
- **Verworfen: Handbuch online auf GitHub öffnen.** Dafür braucht es Internet, und die Nutzer landen auf GitHub.

Bildschirmfotos: **automatisch erzeugt** (gewählt vom User) statt von Hand. Handgemachte Bilder veralten unbemerkt.

PDF: Es wird in CI mit dem vorinstallierten Chrome (`--headless --print-to-pdf`) aus einer HTML-Fassung der Markdown-Dateien erzeugt. Pandoc und LaTeX wären schwer und brächten eine zweite Darstellungslogik.

## 5. Aufbau des Handbuchs

Dateien in `docs/handbuch/`. Die Reihenfolge legt eine Inhaltsdatei fest, `docs/handbuch/inhalt.json` (Titel und Datei, geordnet).

**Teil 1 – Erste Schritte**
1. Willkommen: Was das Programm kann, und eine Begriffsübersicht (Betrieb, Person, Mitgliedschaft, Kategorie, Label, Gruppe, Team).
2. Programm starten und Datei öffnen: Datei öffnen oder neu anlegen, Netzlaufwerk, „Zuletzt geöffnet“, nur lesend geöffnet (eine andere Person arbeitet gerade), übernehmen.

**Teil 2 – Abläufe** (nach den Diagrammen des Vereins)

3. Mitglied aufnehmen, in Stufen:
   - Antrag erfassen: Betrieb bzw. Person anlegen, Mitgliedschaft mit der passenden Kategorie (z.B. **„Mitglied mit Geschäft“**) **ohne Eintrittsdatum**, dazu das Label **„provisorisch“** (Label-Gruppe **„Aufnahme“**).
   - Nach der VS bleibt es provisorisch. Die Benachrichtigung der SBC geschieht vorerst ausserhalb des Programms. Wie der SBC-Kontakt erfasst wird, ist noch offen, das Kapitel wird später ergänzt. Bei **„Mitglied ohne SBC“** entfällt der Schritt.
   - Nach der GV: Label „provisorisch“ entfernen und **Eintrittsdatum** setzen.
   - Abgelehnter Antrag: Person bzw. Betrieb löschen. Er war nie Mitglied.
   - Alle provisorischen Mitglieder findet man über den Label-Filter „provisorisch“, z.B. für die Traktandenliste der GV.
   - Hinweis im Kapitel: Provisorische zählen im Filter „aktive Mitglieder“ mit, weil ihre Mitgliedschaft nicht ausgetreten ist. Wer nur die definitiven will, schliesst das Label „provisorisch“ aus.
4. Mitgliederdaten ändern (mutieren): Adresse, Personen eines Betriebs, Funktionen.
5. Austritt: **Austrittsdatum setzen, nicht löschen.** Die Geschichte bleibt so erhalten. Zusätzlich: wie man Ausgetretene findet.
6. Mitgliederdaten überprüfen: Liste filtern, Mailadressen kopieren bzw. exportieren, Korrekturen einpflegen.
7. Adressen für einen Versand zusammenstellen: Filter, Labels, Gruppe speichern, Export.
8. Datensicherung und Wiederherstellen.
9. Neue Version installieren (Update-Leiste, was dabei mit der Datei passiert).

**Teil 3 – Nachschlagen** (ein Kapitel pro Seite)

10. Adressen
11. Betrieb (Detail)
12. Person (Detail)
13. Gruppen
14. Teams
15. Einstellungen › Listen und Labels
16. Einstellungen › Datensicherung
17. Einstellungen › Excel-Import
18. Einstellungen › Gesamtexport
19. Einstellungen › Über das Programm
20. Häufige Fragen und Probleme: „nur lesend“, Datei nicht gefunden, Update schlägt fehl, Datei aus Versehen geändert.

**Schreibregeln**
- Du-Form oder Sie-Form? Das Programm verwendet bisher neutrale Formulierungen und Imperative wie „Datei öffnen“. Das Handbuch nimmt die **Sie-Form**, weil das im Vereinsumfeld üblich ist (Entscheid des Planers, vom User änderbar).
- Jeder Ablauf ist eine nummerierte Schrittfolge. Knopf- und Menünamen stehen **fett und genau wie im Programm**.
- Pro Ablauf mindestens ein Bildschirmfoto.
- Jedes Kapitel beginnt mit einem Satz „Wofür“.

## 6. Hilfe im Programm

### 6.1 Seitenleiste
- **Öffnen**: das „?“ oben rechts auf jeder Seite, die Taste **F1** und ein Eintrag „Hilfe“ in der Sidebar (öffnet das Inhaltsverzeichnis).
- **Breite**: rund 380 px rechts. Der Inhalt der Seite rückt zusammen und wird nicht verdeckt. Die Leiste kann man schliessen (×, Esc).
- **Inhalt**:
  - Kopfzeile mit Inhaltsverzeichnis-Knopf, Suchfeld, „Handbuch als PDF“ und Schliessen.
  - Darunter das gerenderte Kapitel.
- **Seite und Kapitel**: Die Leiste bleibt offen, wenn man die Seite wechselt. Sie springt dann aber **nicht** automatisch auf das Kapitel der neuen Seite, damit man einen Ablauf zu Ende lesen kann. Ein erneuter Klick auf „?“ öffnet das Kapitel zur aktuellen Seite.
- **Links zwischen Kapiteln** (relative `.md`-Links) öffnen das Ziel in der Leiste. Ein „Zurück“ in der Leiste führt zum vorherigen Kapitel.
- **Externe Links** bleiben wie bisher nicht klickbar. Ausnahme ist nur „Handbuch als PDF“, ein eigener Knopf über `openWebsite`.
- **Änderung an gemeinsamem Code**: Der heutige globale DOMPurify-Hook in `ui/src/lib/markdown.ts` entfernt jedes `href`. Er wird so umgebaut, dass `renderMarkdown` eine Option für interne Kapitel-Links bekommt. Ohne die Option, also für den CHANGELOG, bleiben alle Links entfernt wie heute. Mit der Option bleiben nur relative `*.md`-Links (ggf. mit `#anker`) erhalten, alles andere wird entfernt. Der Hook darf nicht global bleiben, damit sich beide Aufrufe nicht gegenseitig beeinflussen.
- **Suche**: Volltext über alle Kapitel, ohne Unterscheidung von Gross- und Kleinschreibung und von Umlautvarianten. Das Ergebnis ist eine Liste von Kapiteln mit Textausschnitt, ein Klick öffnet das Kapitel.

### 6.2 Zuordnung Seite → Kapitel
Eine Tabelle im Code ordnet jeder Route (`Route.page`, bei `settings` zusätzlich `tab`) genau ein Kapitel zu. Ein Test stellt sicher:
- Jede Route und jeder Einstellungs-Tab hat einen Eintrag.
- Jedes zugeordnete Kapitel existiert in `inhalt.json`.
- Jeder Eintrag in `inhalt.json` hat eine Datei.

Die Seite „Start“ führt zum Kapitel 2.

### 6.3 Einbau der Texte
- Markdown und Bilder werden zur Build-Zeit eingebunden (Vite `import.meta.glob` mit `?raw` bzw. `?url`). Damit funktioniert alles offline und passt zur Version.
- Bildpfade in Markdown sind relativ, z.B. `bilder/adressen.png`. Das Programm setzt beim Rendern die eingebundenen URLs ein. Unbekannte Bilder zeigen den Alt-Text, kein kaputtes Symbol.
- `vite.config.ts` erlaubt das Lesen ausserhalb von `ui/` bereits (`server.fs.allow ['..']`).

### 6.4 Hinweise auf den Seiten
- **Leere Zustände** bekommen je einen Satz und einen Knopf bzw. einen Hilfe-Link: keine Adressen, keine Treffer für einen Filter, keine Gruppen, keine Teams, keine Labels, keine Sicherungen. Die genaue Liste erstellt der Builder durch eine Durchsicht aller Seiten.
- **Tooltips** (`title`) bei Symbol-Knöpfen und mehrdeutigen Aktionen, z.B. „Übernehmen“ bei nur lesend geöffneten Dateien oder „Gruppe speichern“.
- **„Nur lesend“-Hinweis**: Der bestehende Hinweis bekommt einen Link „Was heisst das?“ zum passenden Abschnitt.

### 6.5 „Handbuch als PDF“
- Öffnet `https://github.com/Zer0SkiLL/b-cker-verwaltung/releases/download/v<Version>/Handbuch.pdf` über `openWebsite`. Der Opener-Scope erlaubt das Repo bereits.
- Der Link funktioniert erst, wenn das Release veröffentlicht ist. Solange es ein Entwurf ist, gibt er 404. Das ist in Ordnung: Nutzer bekommen eine Version ohnehin erst über ein veröffentlichtes Release. In Entwicklungs-Builds führt der Link ebenfalls ins Leere.

## 7. Bildschirmfotos

- **Beispieldaten**: Ein Rust-Beispielprogramm im Core (`cargo run -p verwaltung_core --example beispieldaten -- <pfad>`) legt eine Datei mit fiktiven, festen Daten an. Es nutzt dabei nur die öffentliche Core-API. Es enthält:
  - rund 15 Betriebe in den Kantonen AI, AR, GL, GR, SG, TG
  - rund 25 Personen
  - die vier Kategorien des Vereins und das Label „provisorisch“
  - ein ausgetretenes Mitglied
  - Labels, zwei Gruppen und ein Team

  Die Namen sind erkennbar fiktiv, etwa „Bäckerei Muster AG“. Die Kategorie-Namen entsprechen denen des Vereins (siehe 10).
- **Aufnahme**: eine eigene WebdriverIO-Spezifikation `e2e/specs/90-bildschirmfotos.e2e.mjs`.
  - Sie hat eine **eigene Konfiguration** `e2e/wdio.bildschirmfotos.conf.mjs`, die nur diese Spezifikation ausführt. `MITGLIEDERVERWALTUNG_DB` zeigt dort auf eine frisch erzeugte Beispieldatei, nicht auf eine leere Temp-Datei. Die bestehende `wdio.conf.mjs` schliesst die Spezifikation aus (Glob nur `0*-*.e2e.mjs` bzw. `exclude`). Der normale E2E-Lauf bleibt damit unverändert schnell, und der Kernablauf verändert die Beispieldaten nicht.
  - Sie fährt die Seiten bzw. Ablaufschritte an.
  - Sie speichert Bilder unter festen Namen in `docs/handbuch/bilder/`.
  - Fenstergrösse fest bei 1280×800, damit Bilder vergleichbar bleiben.
- **Aktualisierung**: Ein Workflow `.github/workflows/bildschirmfotos.yml` lässt sich von Hand auslösen (`workflow_dispatch`).
  - Er baut, erzeugt die Bilder und lädt sie als **Workflow-Artefakt** „bildschirmfotos“ hoch.
  - Lokal holt `node scripts/bildschirmfotos.mjs` (über `gh run download`) das Artefakt des letzten erfolgreichen Laufs und kopiert die Bilder nach `docs/handbuch/bilder/`. Danach folgt ein normaler Commit `docs: Bildschirmfotos aktualisieren`.
  - Entscheid des Planers: Ein vom Workflow geöffneter Pull Request würde ein PAT bzw. eine geänderte Repo-Einstellung brauchen, und seine CI liefe mit `GITHUB_TOKEN` nicht an. Der Weg über das Artefakt braucht keine neuen Secrets oder Einstellungen.
  - Die Bilder liegen im Repo, damit sie auch auf GitHub und im Programm-Build sichtbar sind.
  - **Wann aktualisieren**: von Hand, wenn sich die Oberfläche sichtbar geändert hat. Der Abschnitt „Neue Version veröffentlichen“ im README bekommt dafür einen Punkt in der Checkliste. CI und Release vergleichen die Bilder **nicht** automatisch (Entscheid des Planers, weil der Aufwand gross wäre und pixelgenaue Vergleiche unter WebKitGTK unzuverlässig sind).
- **Prüfung**: Ein Node-Test (`scripts/handbuch.test.mjs`) stellt sicher:
  - Jedes in Markdown referenzierte Bild existiert.
  - Jedes Bild in `bilder/` wird verwendet.
  - Jeder relative Kapitel-Link zeigt auf eine existierende Datei.
- **Bekannte Einschränkung**: Die Bilder entstehen unter Linux (WebKitGTK). Schriften können leicht von Windows abweichen. Fotografiert wird nur der Fensterinhalt, ohne Rahmen.

## 8. PDF

- Das Skript `scripts/handbuch-pdf.mjs` setzt alle Kapitel in der Reihenfolge von `inhalt.json` zu einer HTML-Datei zusammen:
  - Rendering mit `marked`
  - eingebettete Bilder
  - Titelseite mit Versionsnummer und Inhaltsverzeichnis
  - Druck-CSS mit A4, Seitenumbruch vor jedem Teil und Seitenzahlen
- Chrome bzw. Chromium (auf `ubuntu-22.04`-Runnern vorinstalliert) druckt die Datei mit `--headless --print-to-pdf` zu `Handbuch.pdf`.
- Im Release-Workflow erzeugt ein Job nach dem Notes-Job das PDF und lädt es in den Release-Entwurf hoch (`gh release upload`). Fehlt das PDF, schlägt der Release-Lauf fehl.
- Lokal: `node scripts/handbuch-pdf.mjs --out <pfad>`. Das Skript sucht Chrome an den üblichen Orten oder nimmt den Pfad aus `CHROME`.

## 9. Tests

- **UI (Vitest)**:
  - Zuordnung Seite → Kapitel ist vollständig (6.2).
  - Seitenleiste: öffnen per „?“ und F1, schliessen per Esc, Kapitel-Link öffnet das Ziel, „Zurück“ funktioniert.
  - Suche: findet Treffer unabhängig von Gross- und Kleinschreibung, „ae“/„ä“ gleichwertig, keine Treffer gibt einen Hinweistext.
  - Bild-URLs werden eingesetzt, unbekannte Bilder zeigen den Alt-Text.
  - Leere Zustände zeigen ihren Hinweis.
- **Node**: `scripts/handbuch.test.mjs` (Bilder, Links, Inhaltsdatei) und ein Test der HTML-Zusammenstellung für das PDF (Reihenfolge, Titelseite, Version).
- **E2E (Linux)**: Der bestehende Kernablauf bekommt einen Schritt „? öffnet Hilfe mit dem Kapitel Adressen“.
- **Rust**: Das Beispieldaten-Programm läuft in CI einmal durch (Datei entsteht, erwartete Anzahl Betriebe).

## 10. Entscheide des Users

- Hilfe als **Seitenleiste** (Variante A).
- **Automatische Bildschirmfotos.**
- Ausgetretene Mitglieder behalten ein **Austrittsdatum** und werden nicht gelöscht.
- Mitgliederkategorien des Vereins (genau so geschrieben): **„Mitglied mit Geschäft“**, **„Mitglied ohne Geschäft“**, **„Mitglied ohne SBC“**, **„Kein Mitglied“**.
- Provisorische Mitglieder werden mit dem **Label „provisorisch“** (Label-Gruppe „Aufnahme“, für Betriebe und Personen) markiert. Die Kategorie zeigt von Anfang an die definitive Art der Mitgliedschaft. Das ersetzt den früheren Vorschlag „Kategorie provisorisch“, weil eine Kategorie sonst die Information „mit/ohne Geschäft“ verdrängen würde (Vorschlag des Planers, wartet auf Bestätigung).
- SBC = Schweizerischer Bäcker-Confiseurmeister-Verband. Wie der Kontakt im Programm erfasst wird, ist noch offen und gehört nicht zu diesem Projekt.

**Vorgaben in neuen Dateien** (Entscheid des Planers): Eine neu angelegte Datei enthält die vier Kategorien und die Label-Gruppe „Aufnahme“ mit dem Label „provisorisch“. In bestehenden Dateien legt man sie einmal unter Einstellungen › Listen und Labels an. Das Handbuch erklärt wie (Kapitel 15, Verweis aus Kapitel 3). Umgesetzt wird das beim Anlegen der Datei im Core, ohne neue Migration. Bestehende Dateien bleiben unverändert.
- Es gibt ein PDF zum Drucken, und die Nutzer sollen sich im Programm selbst helfen können.

## 11. Dem Builder überlassen

- Gestaltung der Seitenleiste im bestehenden Stil
- Genaue Liste der leeren Zustände und Tooltips (nach Durchsicht)
- Formulierung der Kapitel nach den Schreibregeln
- Auswahl der Bildschirmfoto-Motive, mindestens eines pro Seite und pro Ablauf
- Algorithmus der Suche (einfacher Teilstring-Vergleich nach Normalisierung reicht)

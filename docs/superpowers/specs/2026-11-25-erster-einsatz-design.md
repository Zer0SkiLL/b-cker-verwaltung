# Bereit für den ersten Einsatz: Import, Sicherung, Problem melden – Design

Stand: 25.11.2026 · Status: Entwurf, wartet auf Freigabe durch den User

## 1. Absicht und Warum

Der User will das Programm möglichst bald in echten Gebrauch bringen: Das Sekretariat installiert es, importiert die bestehende Excel-Liste und arbeitet damit. Die Rückmeldungen daraus bestimmen die nächsten Schritte (Mailvorlagen, Terminierung usw.). Dafür fehlen drei Dinge:

1. **Import:** Der heutige Import passt nicht zum echten Aufbau der Excel-Liste. Er würde jede Zeile mit „Name/Firma fehlt“ verwerfen.
2. **Sicherung:** Die Aufbewahrung „die letzten 20“ zählt Öffnungen, nicht Zeit. Bleibt das Programm lange offen, entsteht keine Sicherung. Ein fehlender zweiter Sicherungsort fällt erst nach 30 Tagen auf. Der User will eine echte Sicherungsstrategie, **einstellbar, mit dem Vorschlag als Standard**.
3. **Problem melden:** Die Nutzer sollen in jeder Situation einen Bericht erzeugen können, den sie dem Entwickler auf dem gewohnten Weg schicken (die Nutzer wissen, wie sie ihn erreichen). Der Bericht soll ohne Rückfragen zeigen, was passiert ist.

Danach folgt ein Release (Minor-Version).

## 2. Ziele, Nicht-Ziele, Anti-Ziele

**Ziele**
- Die anonymisierte Musterdatei (`docs/documentation/20260220 Mitglieder OBC für Reto.xlsx`, Blatt „Mitglieder OBC“) wird ohne Handarbeit vollständig importiert.
- Die Sicherungen werden nach Alter ausgedünnt. Während der Arbeit wird regelmässig gesichert. Fehler am zweiten Ort sind sofort sichtbar. Alle Werte sind einstellbar.
- Ein Knopf **Problem melden** erzeugt einen Bericht zum Kopieren, mit Version, Zustand, letzten Aktionen und Protokoll, aber **ohne Vereinsdaten**.
- Das Handbuch erklärt alle drei Teile. Die Aufbewahrung wird mit Tabelle und Beispiel erklärt (vom User ausdrücklich gewünscht).

**Nicht-Ziele**
- Import in eine Datei, die schon Betriebe oder Personen enthält. Die heutige Regel „nur in eine leere Datei“ bleibt.
- Mehrere zweite Sicherungsorte, Sicherung in die Cloud, Verschlüsselung der Sicherungen.
- Meldungen automatisch versenden (Mail, GitHub). Keine Mailadresse im Programm. Kein Bildschirmfoto durch das Programm selbst.
- Mailvorlagen, Terminierung.

**Anti-Ziele**
- Vereinsdaten (Namen, Adressen, Mailadressen, Formulareingaben) im Bericht oder im Protokoll. Das Repo ist öffentlich, und der Bericht wird über beliebige Kanäle verschickt.
- Eine Aufbewahrung, die die einzige oder neueste Sicherung löscht.
- Ein Import, der bei Abweichungen still Daten verliert. Jede Abweichung gehört in den Bericht.

## 3. Rahmenbedingungen

- Wie bisher: Tauri 2, Rust-Core ohne Tauri-Abhängigkeit, Svelte 5, deutsch in Schweizer Schreibweise, TDD, keine laufenden Kosten, CI auf Linux und Windows.
- Bestehende Dateien dürfen durch Migrationen nur **ergänzt** werden: neue Spalten mit Standardwert, nie löschen.
- Einstellungen, die alle Nutzer einer Datei betreffen, liegen **in der Datei** (Tabelle `settings`), wie heute der zweite Sicherungsort.

## 4. Vorgehen (Approaches)

**Problem melden** (mit dem User besprochen):
- **A) Mail-Vorlage** an eine feste Adresse. Verworfen, weil der User keine Adresse im Programm will.
- **B) GitHub Issues.** Verworfen: Jede meldende Person bräuchte ein GitHub-Konto, und das Repo ist öffentlich.
- **C) Bericht zum Kopieren. Gewählt.** Ergänzt um die letzten Aktionen und das Programm-Protokoll, auf Wunsch des Users.

**Sicherung:**
- Feste Werte (Vorschlag des Planers) gegen einstellbare Werte. **Einstellbar gewählt vom User**, mit dem Vorschlag als Standard.

**Import:**
- **Aufbau umstellen** statt neuer Import-Assistent. Der bestehende Ablauf (Datei wählen → Spalten zuordnen und prüfen → Übernehmen → Bericht) bleibt. Nur die Deutung der Zeilen und die Standard-Zuordnung ändern sich.

## 5. Teil A – Import

### 5.1 Spalten der echten Liste und Standard-Zuordnung

Kopfzeilen werden ohne Rücksicht auf Gross- und Kleinschreibung und auf Leerzeichen am Rand erkannt.

Die Tabelle wurde am 25.11.2026 lokal gegen die Musterdatei geprüft: Kopfzeilen wörtlich wie unten, `OBC-ID` und `PLZ` als Zahl, `Eintritt`/`Austritt` als echte Excel-Datumswerte, die vier Ja/Nein-Spalten als echte Wahrheitswerte, leere Zellen teils als leerer Text. Kategorien in der Datei: „Mitglied mit Geschäft“, „kein Mitglied“. Der Builder darf die Musterdatei lokal lesen, um den Import zu prüfen, aber nie ins Repo übernehmen und keine Inhalte ausgeben.

| Spalte in der Datei | Wird übernommen als |
|---|---|
| OBC-ID | Alt-ID des Betriebs (bzw. bei Einzelmitgliedern in den Bemerkungen der Person: „OBC-ID: …“) |
| Kt Verband | Kantonalverband (Auswahlliste, wird angelegt falls neu, z.B. „FL“) |
| Firma | **Betrieb** (Name) |
| Name | Nachname der Person |
| Vorname | Vorname der Person |
| Zusatz/Inhaber (Handelsregister) | Zusatz des Betriebs |
| Adresszusatz | ignoriert (Entscheid des Users) |
| Adresse, PLZ, Ort | Adresse des Betriebs, bei Einzelmitgliedern der Person |
| Kategorie | Mitgliederkategorie (siehe 5.3) |
| Mail | Mail des Betriebs, bei Einzelmitgliedern der Person |
| Home | Website des Betriebs |
| Lehrbetrieb, Fr Bäck, Fr Conf, DH | wie heute: Labels in „Ausbildung Fachrichtung“, Hinweis „Lehrbetrieb ohne Fachrichtung“ |
| Besonderes | Bemerkungen des Betriebs bzw. der Person |
| Eintritt, Austritt | Mitgliedschaft |
| Filialen | Filialen des Betriebs |
| Telefon | Telefon des Betriebs, bei Einzelmitgliedern der Person |
| Mobiltelefon | Mobiltelefon der Person; ohne Person wie heute Telefon bzw. Bemerkungen des Betriebs |
| Kostenstellen-Nr | **neues Feld** Kostenstellen-Nr des Betriebs |
| Mandat | **Labels** in der Gruppe „Mandat“ (siehe 5.4) |
| Handelsregister-Nr | **neues Feld** Handelsregister-Nr des Betriebs |
| leere Kopfzeilen | ignoriert |

Die bisherigen Annahmen (`ID1`, `Name/Firma`, `Anzahl`, `Postfach`, `Zusatz`) entfallen als Standard. Im Zuordnungsschritt kann man weiterhin jede Spalte von Hand einem Feld zuordnen. Das Feld „Anzahl Lernende“ bleibt dort wählbar.

### 5.2 Wie eine Zeile gedeutet wird

- **Mit `Firma`:** Die Zeile gehört zu einem Betrieb.
  - **Zusammenfassen:** Zeilen mit gleicher Firma, gleicher PLZ und gleicher Adresse ergeben **einen** Betrieb. Verglichen wird ohne Gross-/Kleinschreibung und mit zusammengefassten Leerzeichen. Gleich heissende Betriebe an anderer Adresse bleiben getrennt.
  - **Betriebsangaben** (Adresse, Kategorie, Ein-/Austritt, Labels aus Fachrichtung, Nummern usw.) kommen aus der **ersten** Zeile des Betriebs. Leere Angaben werden aus späteren Zeilen ergänzt. Fachrichtungs-Labels werden über alle Zeilen vereinigt.
  - **Abweichungen ohne Datenverlust:** Bei einer späteren Zeile mit Person gehen **Mail, Telefon, Mobiltelefon und Besonderes**, die vom Betrieb abweichen, **an die Person dieser Zeile** (Mail, Telefon, Mobiltelefon, Bemerkungen). Eine abweichende **OBC-ID** kommt in die Bemerkungen der Person („OBC-ID: …“). Nur Angaben, die zum Betrieb gehören müssen (Adresse, Kategorie, Ein-/Austritt, Kantonalverband, Website, Filialen, Nummern), werden bei Widerspruch nicht übernommen. Der Bericht meldet „Abweichende Angabe für <Feld> (erste Zeile <n> gilt)“. Bei einer späteren Zeile **ohne** Person werden abweichende Mail/Telefon/Besonderes an die Bemerkungen des Betriebs angehängt („Zeile <n>: Mail …“).
  - **Person:** Hat die Zeile `Name` oder `Vorname`, entsteht daraus eine Person, verknüpft mit dem Betrieb, Funktion **„Kontaktperson“** (Entscheid des Planers, weil nicht alle im Handelsregister stehen; nach dem Import pro Person änderbar). Steht nur der Vorname da, wird er als Nachname übernommen (der Nachname ist Pflicht) und der Bericht meldet es.
- **Ohne `Firma`, mit `Name`/`Vorname`:** **Einzelmitglied**, also eine Person mit eigener Mitgliedschaft (Kategorie, Ein-/Austritt), Adresse, Mail, Telefon, Bemerkungen. Fachrichtungs-Spalten gelten nur für Betriebe. Sind sie bei einem Einzelmitglied gesetzt, meldet der Bericht „Fachrichtung bei Einzelmitglied ignoriert“.
- **Ohne `Firma` und ohne Name:** übersprungen, Bericht „Weder Firma noch Name, Zeile übersprungen“.
- Die heutige Meldung „Doppelter Firmenname“ entfällt, weil zusammengefasst wird. Der Bericht nennt pro zusammengefasstem Betrieb einmal „<n> Zeilen zu einem Betrieb zusammengefasst“.

### 5.3 Kategorien und Auswahllisten
- Kategorien, Kantonalverbände, Funktionen und Labels werden **beim Import** ohne Rücksicht auf Gross-/Kleinschreibung mit bestehenden Einträgen abgeglichen, über eigene Import-Helfer. Die bestehenden `find_or_create`-Funktionen und die `UNIQUE`-Regel der Datenbank bleiben unverändert (Entscheid des Planers: Die Eingabe im Programm bleibt, wie sie ist). Aus „kein Mitglied“ wird so die Vorgabe „Kein Mitglied“, es entsteht kein Duplikat.
- Neue Einträge werden in der Schreibweise der Datei angelegt.

### 5.4 Mandat
- Einträge werden an Komma und Semikolon getrennt und getrimmt. Jeder wird ein Label in der Gruppe **„Mandat“**. Die Gruppe gilt für beide und wird bei Bedarf angelegt.
- Hat die Zeile eine Person, bekommt **die Person** das Label, sonst der Betrieb.

### 5.5 Neue Felder beim Betrieb
- Migration `0003`: Spalten `commercial_register_no` (Handelsregister-Nr) und `cost_center_no` (Kostenstellen-Nr), beide Text und optional.
- **Formular Betrieb:** Felder **Handelsregister-Nr** und **Kostenstellen-Nr**.
- **Gesamtexport:** Spalten ergänzen.
- **Suchfeld auf „Adressen“:** Die Handelsregister-Nr wird mitdurchsucht (Entscheid des Planers).

### 5.6 Zusammenfassung nach dem Import
Die heutige Anzeige „Import abgeschlossen“ nennt zusätzlich die Anzahl Einzelmitglieder.

### 5.7 Vorschau und Sicherheiten (vom User verlangt)

Was heute schon gilt, bleibt: Probelauf ohne Speichern, alles oder nichts in einer Transaktion, nur in eine leere Datei. Ergänzt wird:

1. **Vollständige Vorschau** statt „erste Zeilen“, gegliedert wie nachher im Programm:
   - je **Betrieb** eine Zeile mit Ort, Kategorie, Labels (Fachrichtung, Mandat), Quellzeilen (z.B. „Zeilen 12, 13“), darunter seine Personen mit Funktion und Labels
   - **Einzelmitglieder** in einem eigenen Abschnitt
   - **übersprungene Zeilen** in einem eigenen Abschnitt mit Grund
   - Filter **Nur mit Hinweisen**: zeigt nur Betriebe/Personen/Zeilen, zu denen der Bericht etwas meldet
   - grosse Listen bleiben bedienbar (scrollbarer Bereich; bei ca. 300 Zeilen genügt das ohne Seitenaufteilung)
2. **Zusammenfassung** über der Vorschau:
   - „<n> Zeilen → <b> Betriebe (davon <z> aus mehreren Zeilen zusammengefasst), <p> Personen, <e> Einzelmitglieder, <u> Zeilen übersprungen“
   - „Neu angelegt werden: …“ mit den Einträgen pro Liste (Kategorien, Kantonalverbände, Funktionen, Labels je Gruppe), die es in der Datei noch nicht gibt (Abgleich ohne Gross-/Kleinschreibung wie in 5.3)
3. **Abgleich jeder Zeile:** Jede nicht leere Datenzeile ist genau einem Ergebnis zugeordnet: Betrieb (erste Zeile), zusammengefasst in Betrieb, Einzelmitglied oder übersprungen mit Grund. Der Probelauf prüft, dass `Zeilen = Betriebs-Erstzeilen + zusammengefasste + Einzelmitglieder + übersprungene`. Geht die Rechnung nicht auf (Programmfehler), ist **Übernehmen gesperrt**, mit der Meldung „Der Abgleich der Zeilen geht nicht auf – bitte Problem melden“.
4. **Vorschau als Excel speichern…**: eine Excel-Datei mit einer Zeile pro Quellzeile. Spalten: Zeile, Ergebnis (Betrieb / Zusammengefasst / Einzelmitglied / Übersprungen), Betrieb, Person, Kategorie, Labels, Hinweise. So kann man sie neben der Originalliste prüfen. Die Datei wird nur dort gespeichert, wo die Nutzerin sie hinlegt (Speichern-Dialog).
5. **Rückfrage vor dem Übernehmen** mit den Zahlen: „<b> Betriebe, <p> Personen und <e> Einzelmitglieder anlegen?“ (Knöpfe **Übernehmen** / **Abbrechen**).
6. **Rückgängig:**
   - Unmittelbar vor dem Import legt das Programm eine Sicherung neben der Datei an (wie **Jetzt sichern**, nur lokal).
   - Nach erfolgreichem Import erscheint **Import rückgängig machen**. Es fragt nach und spielt diese Sicherung zurück, über den bestehenden Weg „Wiederherstellen“, der vorher den aktuellen Stand sichert.
   - Der Knopf bleibt sichtbar, bis man die Seite verlässt. Danach geht es über **Einstellungen** › **Datensicherung** (Sicherung „vor dem Import“, erkennbar am Zeitpunkt).
7. **Handbuch, Kapitel 17:** empfohlener Ablauf. Zuerst mit **Neue Datei anlegen…** eine Testdatei (z.B. „Test.sqlite“) anlegen, dort importieren und sich umsehen, dann die Testdatei verwerfen und in die echte Datei importieren.

**Tests dazu:**
- Abgleich geht auf, für alle Fallarten gemischt.
- Absichtlich verfälschter Abgleich sperrt das Übernehmen (Unit-Test der Prüffunktion).
- Liste „Neu angelegt werden“ mit Gross-/Kleinschreibung.
- Vorschau-Export enthält genau eine Zeile pro Quellzeile.
- Rückfrage: Abbrechen speichert nichts.
- Rückgängig stellt eine leere Datei wieder her.

## 6. Teil B – Sicherung

### 6.1 Wann gesichert wird

| Ereignis | Neben der Datei | Zweiter Ort |
|---|---|---|
| Öffnen einer bestehenden Datei (wie heute) | ✅ | – |
| **Während der Arbeit:** alle *Intervall* Minuten, **nur wenn seit der letzten Sicherung etwas geändert wurde** und nur bei schreibendem Zugriff | ✅ | ✅ |
| Schliessen (wie heute) | – | ✅ |
| **Jetzt sichern** (wie heute) | ✅ | ✅ |
| **Übernehmen** der Sperre (Leiste „Nur lesen“, deckt den Fall nach einem Absturz ab; nicht der Import-Knopf) | ✅ (schon heute) | ✅ neu |
| Vor dem Wiederherstellen (wie heute) | ✅ | – |

„Geändert“ wird über den Änderungszähler der Verbindung erkannt (`total_changes`), verglichen mit dem Stand bei der letzten Sicherung. **Der Vergleichswert wird nach der Sicherung gelesen,** weil die Sicherung selbst `backup.last_secondary_at` schreibt. Wird die Verbindung neu aufgebaut (Sperre verloren → nur lesend), gilt die Sitzung als nicht schreibend, und es wird nicht gesichert. Den Zeitgeber führt die Oberfläche. Sie ruft einen Befehl „sichern, falls geändert“ auf. Der Befehl entscheidet selbst und ist bei nur lesendem Zugriff ein No-op.

### 6.2 Aufbewahrung nach Alter
Nach jeder Sicherung wird **an diesem Ort** ausgedünnt. Massgebend ist der Zeitstempel im Dateinamen (wie heute):

| Alter der Sicherung | Was bleibt | Standard |
|---|---|---|
| jünger als *A* Tage | **alle** | A = 2 |
| bis *B* Tage | die neueste **pro Kalendertag** | B = 14 |
| bis *C* Wochen | die neueste **pro Kalenderwoche** (Montag–Sonntag) | C = 8 |
| bis *D* Monate | die neueste **pro Kalendermonat** | D = 12 |
| älter | gelöscht | |

- **Die neueste Sicherung bleibt immer erhalten,** unabhängig vom Alter.
- **Überall die neue Regel:** Sie ersetzt `BACKUP_KEEP` an allen drei Stellen (beim Öffnen, bei `backup_now`, am zweiten Ort).
- **Vor den Migrationen:** Die Sicherung beim Öffnen läuft vor den Migrationen. Fehlt die Tabelle `settings` oder ein Wert, gelten die Standardwerte.
- Sicherungen fremder Dateien im selben Ordner bleiben unberührt (wie heute: nur der eigene Dateistamm).
- Ungültige Einstellungen (z.B. B < A) werden beim Speichern abgewiesen, mit deutscher Meldung.

### 6.3 Einstellungen
- **Ort:** **Einstellungen** › **Datensicherung**, neuer Bereich **Aufbewahrung**, mit fünf Zahlenfeldern:
  - „Alle Sicherungen behalten für … Tage“
  - „Eine pro Tag für … Tage“
  - „Eine pro Woche für … Wochen“
  - „Eine pro Monat für … Monate“
  - „Während der Arbeit sichern alle … Minuten (0 = aus)“
- **Knöpfe:** **Speichern** und **Standard wiederherstellen**.
- **Hinweis:** Ein Satz mit Link zum Handbuch-Abschnitt, der die Aufbewahrung erklärt.
- **Speicherort und Grenzen:** Die Werte liegen in der Datei (`settings`), mit den Standardwerten, solange nichts gespeichert ist. Gültige Bereiche: A 1–30, B A–90, C 1–104, D 0–120, Intervall 0 oder 15–480.
- **Nur lesend:** Ändern ist gesperrt.

### 6.4 Fehler am zweiten Ort sofort zeigen
- Schlägt die Sicherung am zweiten Ort fehl (während der Arbeit, mit **Jetzt sichern**, beim **Übernehmen**), merkt sich die Sitzung den Fehler.
- Die gelbe Leiste zeigt dann „Zweiter Sicherungsort nicht erreichbar – letzte Sicherung dort am <Datum>“, mit **Jetzt sichern**. Sie verschwindet, sobald eine Sicherung dort gelingt.
- Ein Fehler beim Schliessen kann nicht mehr angezeigt werden. Er wird ins Protokoll geschrieben (Teil C) und beim nächsten Öffnen über die bestehende 30-Tage-Erinnerung sichtbar.

### 6.5 Wegfall
Die feste Grenze „die letzten 20“ (`BACKUP_KEEP`) entfällt zugunsten von 6.2.

## 7. Teil C – Problem melden

### 7.1 Bedienung
- **Wo:** **Problem melden** in der linken Leiste (unter „Hilfe“) und unten in der Hilfe-Leiste neben „Handbuch als PDF“. Über die Hilfe-Leiste ist es auch auf der Startseite erreichbar.
- **Was:** Es öffnet ein Fenster (modaler Dialog im Programm, `role="dialog"`) mit:
  - dem Feld **Was ist passiert?** (mehrzeilig, optional; landet oben im Bericht)
  - dem Bericht als schreibgeschützter Text zur Ansicht
  - **Bericht kopieren** (Zwischenablage, danach „Kopiert.“)
  - **Schliessen**
  - dem Hinweis: „Fügen Sie den Bericht in Ihre Nachricht an den Entwickler ein. Ein Bildschirmfoto machen Sie unter Windows mit **Windows + Umschalt + S** und fügen es ebenfalls ein.“
- **Esc:** schliesst den Dialog. Das Esc-Verhalten der Hilfe darf dabei nicht greifen (sie prüft schon auf `dialog[open]`).

### 7.2 Inhalt des Berichts (reiner Text)
```
Problembericht Mitgliederverwaltung
Zeit: 25.11.2026 14:03
Version: 0.3.0 · Windows 11 (x86_64)   (Betriebssystem über die Crate `os_info`)

Was ist passiert?
<Text der Nutzerin oder „(keine Angabe)“>

Zustand
- Seite: Betrieb (Detail)
- Datei: geöffnet, schreibend | nur lesend | keine Datei
- Zweiter Sicherungsort: eingerichtet, letzte Sicherung 25.11.2026 13:10 | nicht eingerichtet | Fehler: …
- Aufbewahrung: 2/14/8/12, Intervall 60 min

Letzte Aktionen (neueste zuletzt)
14:01:12 Seite: Adressen
14:01:20 save_organisation – OK (35 ms)
14:02:05 create_label – Fehler: Name ist bereits vergeben
...

Programm-Protokoll (letzte Einträge)
2026-11-25T13:10:02 WARN Zweiter Sicherungsort nicht erreichbar: …
...
```

### 7.3 Letzte Aktionen
- **Was aufgezeichnet wird:**
  - jeder Seitenwechsel (Name der Seite bzw. des Einstellungs-Tabs, keine IDs)
  - jeder Aufruf eines Rust-Befehls über `call` in `ui/src/lib/api.ts`: **Name des Befehls**, Ergebnis OK/Fehler, Dauer, bei Fehler die Fehlermeldung
  - unerwartete Fehler der Oberfläche (`error`, `unhandledrejection`)
- **Was nicht aufgezeichnet wird:** **keine Argumente und keine Rückgabewerte**.
- **Speicher:** Ringpuffer der letzten 50 Einträge in `localStorage` (bleibt nach einem Absturz erhalten). Lesen und Schreiben in try/catch; ohne Speicher funktioniert alles weiter.
- **Bericht:** zeigt die letzten 30.

### 7.4 Programm-Protokoll
- **Technik:** Protokolliert wird über die `log`-Fassade, in Core und App gleich. Für Tests gibt es einen Logger, der die Zeilen sammelt. Im Programm schreibt `tauri-plugin-log` in den Protokoll-Ordner des Programms (`LogDir`). Die Datei rotiert (z.B. 1 MB, eine alte Datei bleibt). Stufe Info und höher.
- **Was protokolliert wird:**
  - Programmstart mit Version
  - Öffnen/Schliessen einer Datei (ohne Pfad)
  - Sperre übernommen bzw. verloren
  - Sicherungen (Ort lokal/zweiter Ort, OK/Fehler mit Fehlermeldung)
  - Wiederherstellen
  - Import (Anzahl Zeilen, Fehler)
  - alle Fehler, die an die Oberfläche zurückgegeben werden (Befehlsname und Meldung)
- **Was nicht protokolliert wird:** keine Vereinsdaten, keine Dateipfade. Fehlermeldungen können einen Pfad enthalten (z.B. „Datei nicht gefunden: …“). Das ist zulässig, weil die Nutzerin den Bericht vor dem Kopieren sieht.
- **Bericht:** Ein Befehl liefert die letzten 40 Zeilen mit Stufe WARN oder ERROR, dazu die letzten 10 Zeilen jeder Stufe, ohne doppelte Zeilen, in zeitlicher Reihenfolge.

### 7.5 Datenschutz-Test
- Ein Test füllt die Aktionsliste über typische Aufrufe mit personenbezogenen Argumenten und prüft, dass weder Argumente noch Rückgabewerte im Bericht erscheinen.
- Ein Rust-Test prüft, dass Protokollzeilen beim Speichern eines Betriebs dessen Namen nicht enthalten.

## 8. Handbuch

- **Kapitel 08 und 16:** Die Tabelle „Wann gesichert wird“ und die Tabelle zur Aufbewahrung **mit dem Beispiel** („Ein Fehler vom Montag wird erst am Freitag bemerkt …“) aus dem Gespräch mit dem User. Dazu die Einstellungen und die neue gelbe Leiste.
- **Kapitel 02b:** Abschnitt Sicherungsstrategie an die neue Aufbewahrung anpassen. Hinweis: Die Einstellungen gelten für alle, die die Datei öffnen.
- **Kapitel 01, 03, 12:** Importierte Personen haben neu die Funktion „Kontaktperson“, falls irgendwo „Inhaber“ als Import-Funktion erwähnt ist.
- **Kapitel 17:** Die neue Deutung der Excel-Zeilen (Zusammenfassen, Kontaktperson, Einzelmitglieder, Mandat) und die Spaltentabelle 5.1.
- **Kapitel 11:** neue Felder Handelsregister-Nr und Kostenstellen-Nr.
- **Neues Kapitel „Problem melden“** im Nachschlageteil, dazu ein Verweis in Kapitel 20.
- **Bildschirmfotos:** Datensicherung (mit Aufbewahrung) und „Problem melden“ ergänzen bzw. aktualisieren. Die Beispieldaten bekommen eine Handelsregister-Nr bei einigen Betrieben.

## 9. Tests (Auswahl, verbindlich)

- **Import (Rust)**, mit einer im Test erzeugten Excel-Datei mit den echten Kopfzeilen aus 5.1:
  - Zusammenfassen gleicher Betriebe
  - Abweichung wird gemeldet
  - Einzelmitglied
  - nur Vorname
  - Mandat mit zwei Einträgen an einer Person
  - „kein Mitglied“ landet bei der Vorgabe
  - leere Kopfzeilen und `Adresszusatz` werden ignoriert
  - Kostenstellen- und Handelsregister-Nr landen in den neuen Feldern
- **Musterdatei:** Ein Test, der die private Musterdatei liest, darf es **nicht** geben (sie liegt nicht im Repo). Stattdessen prüft der Builder einmal lokal von Hand, dass die Musterdatei vollständig importiert wird, und hält das Ergebnis im Ledger fest (Anzahl Betriebe/Personen, keine unerwarteten Meldungen).
- **Aufbewahrung (Rust):**
  - eine reine Funktion `retain(files, now, policy) -> löschen`, mit Tabellentests je Altersstufe
  - die neueste bleibt immer
  - Grenzen exakt (genau 2 Tage alt)
  - Wochengrenze Sonntag/Montag
  - fremde Dateien bleiben
- **Sichern, falls geändert:** keine Sicherung ohne Änderung, eine Sicherung nach Änderung, No-op bei nur lesend.
- **Zweiter Ort nicht erreichbar:** Die Sitzung meldet den Fehler, nach erfolgreicher Sicherung ist er weg.
- **UI:** Einstellungen Aufbewahrung (Validierung, Standard wiederherstellen, gesperrt bei nur lesend), gelbe Leiste bei Fehler am zweiten Ort, Dialog Problem melden (Inhalt, Kopieren, Esc), Ringpuffer (max. 50, übersteht fehlenden Speicher), Datenschutz-Test 7.5.

## 10. Entscheide des Users

- Import:
  - Zeilen ohne Firma sind Einzelmitglieder.
  - Mehrere Zeilen pro Betrieb kommen vor (zusammenfassen).
  - Mandat wird zu Labels.
  - Handelsregister-Nr und Kostenstellen-Nr werden im Programm gepflegt.
  - `Adresszusatz` wird ignoriert.
- Sicherung: einstellbar, mit dem Vorschlag 2/14/8/12 und 60 Minuten als Standard. Die Erklärung mit Tabelle und Beispiel kommt ins Handbuch.
- Problem melden: ein Bericht zum Kopieren mit letzten Aktionen und Programm-Protokoll, keine Mailadresse, kein GitHub.

## 11. Dem Builder überlassen

- Gestaltung des Dialogs und des Bereichs Aufbewahrung im bestehenden Stil
- Genaue Formulierung der Protokollzeilen, solange 7.4 eingehalten ist
- Ob „sichern, falls geändert“ den Zähler pro Sitzung oder pro Ort führt, solange 6.1 gilt

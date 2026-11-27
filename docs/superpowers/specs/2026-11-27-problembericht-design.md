# Problembericht 2.0 – Design

## Absicht

„Problem melden“ soll beim ersten Test auf dem Netzlaufwerk und später im Betrieb genug zeigen, dass der Entwickler ein Problem ohne Rückfragen eingrenzen kann: Sperre, Netzlaufwerk, Abstürze, Speicherverbrauch, Laufzeiten. Der Bericht muss für das Sekretariat lesbar bleiben und lässt sich neu auch als Datei speichern, um ihn abzulegen oder anzuhängen.

## Ziele

1. **Als Datei speichern:** Knopf **Als Datei speichern…** neben **Bericht kopieren**. Speicherdialog mit Vorschlag `Problembericht-JJJJ-MM-TT.txt`, Inhalt identisch mit dem angezeigten Bericht (UTF-8). Danach „Gespeichert: <Pfad>“.
2. **Sperre:** Bei „nur lesend“ den PC, der die Sperre hält, und seit wann; Hinweis, wenn sie vermutlich von einem Absturz stammt.
3. **Ort der Datei:** „Netzlaufwerk (S:)“, „Netzpfad“, „lokal“ (Windows über `GetDriveTypeW`; macOS/Linux: `/Volumes/…`, `/mnt/…`, `/media/…` → „externes Laufwerk oder Netz“, sonst „lokal“).
4. **Abstürze:** Panic-Hook schreibt `FEHLER Absturz: <Meldung> (<Datei:Zeile>)` ins Protokoll, dann der Standard-Hook. Unerwartete Fehler der Oberfläche (`recordError`) gehen zusätzlich ins Protokoll.
5. **Datei-Angaben:** Grösse, Datenbank-Version (`PRAGMA user_version`), Anzahl Betriebe, Personen, aktive Mitglieder, Zeit der neuesten Sicherung neben der Datei.
6. **Mehr Protokoll:** Exporte (Adressen, Gesamtexport, Import-Bericht) mit Art, Zeilenzahl, Dauer; Update-Prüfung (keine neue Version / Version X verfügbar / Fehler); „Programmende“ beim Beenden. Import, Sicherung und Wiederherstellung protokollieren schon.
7. **Computer und Programm:** CPU gesamt und Anteil des Programms; RAM belegt/gesamt, Programm aktuell und höchster Wert seit Start; Programm läuft seit; Computer eingeschaltet seit; letzte Starts mit „vorher nicht sauber beendet“.
8. **Lesbares Protokoll:** `[2026-11-26][21:33:53][modul][WARN] Text` → `26.11. 21:33:53  WARNUNG  Text`. Älteste oben, neueste unten. Lücken zwischen ausgewählten Zeilen als `  …`.
9. **Gestaltung:** Layout „Dashboard“ (vom User gewählt, siehe unten).

## Nicht-Ziele / Anti-Ziele

- **Keine Vereinsdaten:** keine Namen, Adressen, Suchbegriffe, Filterwerte, Formulareingaben, kein Dateipfad im Bericht. Der Windows-Benutzername der Sperre kommt nicht in den Bericht (nur der PC-Name). Benutzerordner in Protokollzeilen werden maskiert: `C:\Users\Hans\…` → `C:\Users\…\…`, `/Users/hans/…` und `/home/hans/…` ebenso.
- Kein automatisches Senden, kein Server.
- Kein `.md`-Export (unter Windows ohne Standardprogramm).
- Keine Messung, die das Programm spürbar bremst: die Messung für den Höchstwert läuft einmal pro Minute in einem Hintergrund-Thread.

## Layout (gewählt: Dashboard)

```
╔══════════════════════════════════════════════╗
║  PROBLEMBERICHT · Mitgliederverwaltung 0.5.0 ║
║  27.11.2026, 14:32 · Windows 11 (x86_64)     ║
╚══════════════════════════════════════════════╝

» Beim Speichern kam eine Fehlermeldung.

  STATUS AUF EINEN BLICK
  [ OK ] Datei              Netzlaufwerk (S:)
  [ !! ] Schreibzugriff     nur lesend – gesperrt von PC-SEKRETARIAT seit 27.11. 08:02
  [ OK ] Sicherung          neben der Datei 27.11. 14:00 · zweiter Ort 27.11. 14:00
  [ !! ] Letzter Start      vorher nicht sauber beendet
  [ OK ] Arbeitsspeicher    Programm 310 MB
  [ !! ] Protokoll          3 Warnungen/Fehler heute

  COMPUTER
  Prozessor  ██░░░░░░░░░░░░░░░░░░  12 % (Programm 1 %)
  Speicher   ███████████░░░░░░░░░  9.1 / 16.0 GB
  Programm   ▌░░░░░░░░░░░░░░░░░░░  310 MB (höchster Wert 420 MB)

  LAUFZEITEN
  Programm   seit 27.11. 08:01   6 h 31 min
  Computer   seit 25.11. 07:45   2 Tage 6 h
  Starts     27.11. 08:01 · 26.11. 21:33 (!) · 26.11. 07:58

  DATEI
  2.4 MB · Datenbank-Version 7 · Aufbewahrung 14/30/12/24, alle 30 min
  120 Betriebe · 340 Personen · 410 aktive Mitglieder

  SEITE
  Adressen

LETZTE AKTIONEN (neueste unten)
  14:30:02  Seite: Adressen
  14:31:55  update_organisation – Fehler: Name fehlt

PROTOKOLL (neueste unten)
  27.11. 08:01:12  INFO     Programmstart, Version 0.5.0
  …
  27.11. 14:31:55  WARNUNG  Fehler an die Oberfläche: Name fehlt
```

- Kopfzeilen-Rahmen passt seine Breite dem längeren der beiden Texte an (Breite = längste Zeile + 4).
- Fehlt die Beschreibung: `» (keine Angabe)`.
- Ohne geöffnete Datei: Status `[ -- ] Datei  keine Datei geöffnet`; Schreibzugriff, Sicherung und Abschnitt DATEI entfallen.
- Fehlt eine Messung (z.B. sysinfo liefert nichts): Zeile mit `unbekannt`, nie ein Abbruch des Berichts.

### Regeln für den Status

| Zeile | OK | !! | -- |
|---|---|---|---|
| Datei | geöffnet, Grösse lesbar → Ort | Grösse nicht lesbar → „nicht erreichbar“ | keine Datei |
| Schreibzugriff | schreibend | nur lesend; mit „gesperrt von HOST seit …“, bei `likelyCrash` zusätzlich „(vermutlich nach Absturz)“ | – |
| Sicherung | sonst | Fehler am zweiten Ort, Erinnerung fällig, oder noch keine Sicherung neben der Datei | – |
| Letzter Start | vorheriger Lauf endete mit „Programmende“ | ohne „Programmende“ → „vorher nicht sauber beendet“ | erster Start im Protokoll |
| Arbeitsspeicher | Programm < 1 GB | ≥ 1 GB | unbekannt |
| Protokoll | „keine Warnungen/Fehler heute“ | „N Warnungen/Fehler heute“ | – |

### Balken

20 Zeichen; gefüllt `█`, halb `▌`, leer `░`. Füllung = Anteil × 20, auf halbe Zeichen gerundet, mindestens `▌` bei Anteil > 0. Programm-Balken: Anteil am gesamten RAM.

## Ansätze

1. **Rust sammelt, TypeScript gestaltet (gewählt).** Ein neuer Befehl `diagnostics_snapshot` liefert alle Fakten als Daten (Messwerte, Laufzeiten, Starts, Datei-Angaben, fertig aufbereitete Protokollzeilen). `buildReport` in `ui/src/lib/diagnostics.ts` setzt das Layout. Passt zum bestehenden, getesteten Aufbau; Layout-Änderungen brauchen keinen Rust-Build.
2. Rust baut den ganzen Text. Verworfen: Aktionsliste und Beschreibung liegen in der Oberfläche, der Text müsste hin und her.
3. Messwerte per Hand je Betriebssystem (WinAPI, /proc, mach). Verworfen: viel Code; `sysinfo` deckt Windows, Linux, macOS ab.

## Entscheide

- **Bibliothek:** `sysinfo` für CPU, RAM, Prozesse, Start- und Einschaltzeit. Unter Windows zusätzlich `windows-sys` (`Win32_Storage_FileSystem`) für `GetDriveTypeW`.
- **Programm-Anteil:** eigener Prozess plus alle Nachkommen (Eltern-Kette über `parent()`); unter Windows sind das die WebView2-Prozesse. Unter macOS laufen die WebKit-Prozesse ausserhalb des Baums und fehlen (bekannt, akzeptiert).
- **CPU:** zwei Messungen im Abstand von `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`; Prozess-CPU durch Anzahl Kerne geteilt, damit beide Werte 0–100 % sind.
- **Höchstwert RAM:** Hintergrund-Thread misst alle 60 s den Programm-Baum, Höchstwert in einem `AtomicU64`; jede Momentaufnahme aktualisiert ihn ebenfalls.
- **Starts:** aus allen Protokollzeilen (nicht nur dem Auszug): jede Zeile „Programmstart“ ist ein Start; „sauber“, wenn zwischen dem vorherigen Start und diesem ein „Programmende“ steht. Die letzten 5, neueste zuerst. Ein Herunterfahren ohne `RunEvent::Exit` zählt als nicht sauber (akzeptiert).
- **Protokoll-Auszug:** Auswahl wie heute (letzte 40 Warnungen/Fehler + letzte 10 Zeilen), danach aufbereitet und maskiert. Nicht lesbare Zeilen bleiben unverändert (nur maskiert).
- **Protokoll aus der Oberfläche:** Befehl `diagnostics_log(level, message)` (`info`/`warn`/`error`), aus der Oberfläche direkt per `invoke`, nicht über `call`, damit er nicht in der Aktionsliste landet. Nachricht auf 500 Zeichen gekürzt.
- **Speichern:** Befehl `save_report(path, text)` schreibt die Datei in Rust (wie `export_report`).
- **Zeitangaben:** Rust liefert ISO-Zeiten (RFC 3339) und Sekunden; die Oberfläche formatiert in Schweizer Schreibweise (`27.11. 08:01`, `6 h 31 min`, `2 Tage 6 h`).

## Datenschutz-Prüfung

Der bestehende Test `logs_contain_no_business_data` wird um Exporte erweitert. Neuer Test: der fertige Bericht einer Sitzung mit Betrieb „Bäckerei Geheim AG“ in einem Ordner „Geheimordner“ enthält weder Namen noch Pfad.

## Handbuch

`21-problem-melden.md`: neuer Knopf **Als Datei speichern…**, Liste „Was im Bericht steht“ erweitert, Hinweis, dass der Bericht keinen Windows-Benutzernamen enthält. Bildschirmfoto `problem-melden.png` mit dem Workflow **Bildschirmfotos** erneuern.

## Dem Umsetzer überlassen

Interne Aufteilung der Rust-Module, Namen privater Funktionen, genaue Rundung der Anzeigen (eine Nachkommastelle bei GB, ganze MB, ganze Prozent).

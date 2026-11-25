# Datensicherung und Wiederherstellen

Dieses Kapitel erklärt, wie Ihre Daten gesichert werden und wie Sie eine Sicherung zurückspielen.

## Wann gesichert wird

| Ereignis | Neben der Datei (Ordner **backups**) | Zweiter Ort (z.B. USB-Stick) |
|---|---|---|
| **Öffnen** der Datei | ✅ | – |
| **Während der Arbeit**: alle 60 Minuten, aber nur wenn sich etwas geändert hat | ✅ | ✅ |
| **Schliessen** des Programms | – | ✅ |
| **Jetzt sichern** | ✅ | ✅ |
| **Übernehmen** (zum Beispiel nach einem Absturz) | ✅ | ✅ |
| **Wiederherstellen** (vorher wird der aktuelle Stand gesichert) | ✅ | – |

Wer an einem Tag drei Stunden arbeitet, hat danach etwa 4–5 Sicherungen von diesem Tag. An Tagen ohne Arbeit entsteht keine. Jede Änderung wird übrigens schon beim **Speichern** fest in die Datei geschrieben – die Sicherungen sind zusätzlich da, falls etwas schiefgeht.

## Wie lange Sicherungen aufbewahrt werden

Damit man auch Wochen und Monate zurückkommt, ohne dass sich die Dateien stapeln, werden ältere Sicherungen **ausgedünnt**. Gezählt werden dabei nur die Tage, an denen gesichert wurde, also die Tage, an denen jemand mit der Datei gearbeitet hat:

| Sicherungen | Was bleibt |
|---|---|
| vom heutigen und vom letzten Arbeitstag davor | **alle** |
| der letzten 14 Arbeitstage | die neueste **pro Tag** |
| der letzten 8 Wochen mit Arbeit | die neueste **pro Woche** |
| der letzten 12 Monate mit Arbeit | die neueste **pro Monat** |
| ältere | gelöscht |

**Ein Beispiel:** Ein Fehler vom Montag wird erst am Freitag bemerkt. Dann gibt es noch den Stand von Sonntagabend, dazu den Stand jeder Woche der letzten zwei Monate und jedes Monats im letzten Jahr. Zusammen sind das höchstens etwa 40 Dateien pro Ort. **Die neueste Sicherung bleibt immer erhalten.**

**Ferien und falsche Uhrzeit:** Weil nur Arbeitstage zählen, gehen in einer Pause keine Sicherungen verloren. Nach drei Wochen Ferien sind die Stände vor den Ferien noch alle da. Dasselbe gilt, wenn die Uhr des Computers einmal falsch geht.

Alle vier Zeiten und die 60 Minuten lassen sich unter **Einstellungen** › **Datensicherung** › **Aufbewahrung** ändern (siehe [Datensicherung](16-datensicherung.md)).

## Wenn der zweite Ort nicht erreichbar ist

Ist der zweite Ort bei einer Sicherung nicht erreichbar (zum Beispiel weil der USB-Stick nicht steckt), erscheint sofort eine gelbe Leiste: „Zweiter Sicherungsort nicht erreichbar – letzte Sicherung dort am …“. Stecken Sie den Stick ein und klicken Sie in der Leiste auf **Jetzt sichern**. Gelingt die Sicherung, verschwindet die Leiste.

## Zweiten Sicherungsort einrichten

Welche Lösung zu Ihnen passt, allein oder zu mehreren, steht unter [Einrichten: allein oder zu mehreren](02b-einrichten.md).

Ein zweiter Ort schützt auch dann, wenn das Netzlaufwerk oder der PC ausfällt. Geeignet ist zum Beispiel ein USB-Stick oder ein anderes Laufwerk.

1. Gehen Sie auf **Einstellungen** › **Datensicherung**.
2. Klicken Sie auf **Zweiten Ort wählen…** und wählen Sie den Ordner.
3. Klicken Sie auf **Jetzt sichern**, um gleich eine erste Kopie anzulegen.

![Die Seite Datensicherung](bilder/einstellungen-sicherung.png)

## Eine Sicherung zurückspielen

Zum Beispiel, wenn versehentlich etwas gelöscht wurde.

1. Gehen Sie auf **Einstellungen** › **Datensicherung**.
2. Suchen Sie in der Liste den Stand vor dem Fehler (Datum und Uhrzeit).
3. Klicken Sie auf **Wiederherstellen** und bestätigen Sie die Rückfrage.

Bevor das Programm ersetzt, sichert es den aktuellen Stand. Sie können den Schritt also rückgängig machen, indem Sie diese neueste Sicherung wiederherstellen.

**Sicherung vom USB-Stick:** Klicken Sie auf **Aus Datei wiederherstellen…** und wählen Sie die Sicherungsdatei.

> **Achtung:** Alles, was nach dem gewählten Stand erfasst wurde, ist danach nicht mehr in der Datei. Notieren Sie wichtige Änderungen vorher.

## Alles exportieren

Zusätzlich zu den Sicherungen können Sie alle Daten als Excel-Datei speichern, die auch ohne dieses Programm lesbar ist (siehe [Gesamtexport](18-gesamtexport.md)).

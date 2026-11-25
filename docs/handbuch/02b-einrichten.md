# Einrichten: allein oder zu mehreren

Dieses Kapitel erklärt die Erstinstallation, wo die Datei am besten liegt, wie sich mehrere Personen eine Datei teilen und wie Sie Ihre Daten zuverlässig sichern.

## Erstinstallation

1. Öffnen Sie im Browser die Seite mit den Versionen: **github.com/Zer0SkiLL/b-cker-verwaltung/releases/latest**.
2. Laden Sie unter „Assets“ die Datei **Mitgliederverwaltung_…_x64-setup.exe** herunter und starten Sie sie.
3. Windows zeigt beim ersten Mal wahrscheinlich die Meldung **„Der Computer wurde durch Windows geschützt“**. Das liegt daran, dass das Programm keine kostenpflichtige Herausgeber-Signatur hat. Klicken Sie auf **Weitere Informationen** und dann auf **Trotzdem ausführen**.
4. Die Installation braucht keine Administratorrechte. Danach finden Sie das Programm im Startmenü unter **Mitgliederverwaltung**.

**Updates kommen danach von selbst:** Das Programm prüft bei jedem Start, ob es eine neue Version gibt, und bietet sie in einer gelben Leiste an (siehe [Neue Version installieren](09-neue-version.md)). Das Handbuch als PDF zum Ausdrucken liegt auf derselben Seite (**Handbuch.pdf**).

## Wo soll die Datei liegen?

Alle Vereinsdaten stehen in **einer Datei** (Endung `.sqlite`). Daneben legt das Programm den Ordner **backups** mit den automatischen Sicherungen an. Wählen Sie den Ort darum bewusst:

| Situation | Empfehlung |
|---|---|
| **Eine Person, ein PC** | Lokal, zum Beispiel unter **Dokumente\Mitgliederverwaltung**. Unbedingt einen zweiten Sicherungsort einrichten (siehe unten). |
| **Mehrere Personen** (Sekretariat und Ferienvertretung) | Auf einem **gemeinsamen Netzlaufwerk** (Server oder NAS, Windows-Freigabe), das alle unter demselben Pfad erreichen, zum Beispiel **S:\Verein\Mitgliederverwaltung**. |
| **Cloud-Ordner** (OneDrive, Dropbox, Google Drive) | Nur für **eine Person** und nur, wenn das Programm geschlossen ist, bevor der Ordner auf einem anderen Gerät benutzt wird. Für mehrere Personen **nicht geeignet**: Die Synchronisation kann gleichzeitige Änderungen nicht zusammenführen und legt dann Konflikt-Kopien an. |

Die Datei anlegen Sie mit **Neue Datei anlegen…** direkt am gewählten Ort. Die bisherige Excel-Liste übernehmen Sie danach unter **Einstellungen** › **Excel-Import** (siehe [Excel-Import](17-excel-import.md)).

## Gemeinsam nutzen

Mehrere Personen können dieselbe Datei verwenden, aber **immer nur eine Person gleichzeitig ändern**. Das Programm regelt das selbst:

- Wer die Datei zuerst öffnet, kann ändern. Das Programm vermerkt das neben der Datei (in einer kleinen Datei mit der Endung `.lock`).
- Wer sie danach öffnet, sieht oben **„Nur lesen – geöffnet von …“** und kann alles ansehen, suchen und exportieren, aber nichts ändern.
- Beim Schliessen des Programms wird die Datei wieder frei.

**So installiert die Ferienvertretung:** Das Programm auf dem eigenen PC installieren (wie oben), dann mit **Datei öffnen…** dieselbe Datei auf dem Netzlaufwerk wählen. Ab dann bietet die Startseite diese Datei unter **Zuletzt geöffnet** an.

**Übergabe vor den Ferien:**

1. Die Hauptperson schliesst das Programm, bevor sie geht.
2. Die Vertretung öffnet die Datei und arbeitet normal.
3. Nach den Ferien schliesst die Vertretung das Programm, und die Hauptperson öffnet die Datei wieder.

Hat jemand das Programm offen gelassen (oder ist der PC abgestürzt), klicken Sie auf **Übernehmen**. Sprechen Sie sich wenn möglich vorher ab: Wer die Datei vorher offen hatte, kann danach nichts mehr speichern. Ist die Sperre älter als 24 Stunden, weist das Programm selbst darauf hin, dass sie wahrscheinlich von einem Absturz stammt. Mehr dazu unter [Programm starten und Datei öffnen](02-start-und-datei.md).

## Sicherungsstrategie

Die Daten werden in drei Stufen gesichert. Jede Stufe schützt vor etwas anderem:

| Stufe | Wann | Schützt vor |
|---|---|---|
| **1. Automatische Sicherung neben der Datei** (Ordner **backups**, ausgedünnt nach Alter) | Beim Öffnen, während der Arbeit und mit **Jetzt sichern** | Versehentlichem Löschen oder falschen Änderungen |
| **2. Zweiter Sicherungsort** | Während der Arbeit, beim Schliessen und mit **Jetzt sichern** | Ausfall des Laufwerks, auf dem die Datei liegt |
| **3. Gesamtexport als Excel** | Von Hand, zum Beispiel einmal pro Jahr nach der GV | Totalverlust; lesbar auch ohne dieses Programm |

Stufe 1 läuft immer von selbst. Stufe 2 richten Sie einmal ein, Stufe 3 machen Sie von Hand.

### Allein arbeiten

1. **Einstellungen** › **Datensicherung** › **Zweiten Ort wählen…**: ein anderes Laufwerk als das der Datei, zum Beispiel eine externe Festplatte, ein USB-Stick oder ein NAS.
2. Steckt der USB-Stick nicht immer, schliessen Sie ihn regelmässig an und klicken auf **Jetzt sichern**. Ist er bei einer Sicherung nicht erreichbar, zeigt eine gelbe Leiste das sofort an.
3. Einmal im Jahr: **Einstellungen** › **Gesamtexport** und die Excel-Datei an einem dritten Ort aufbewahren.

### Zu mehreren arbeiten

Der zweite Sicherungsort und die Einstellungen zur Aufbewahrung sind **in der Datei gespeichert** und gelten darum für alle, die sie öffnen. Wählen Sie deshalb einen Ort, den **alle PCs unter demselben Pfad erreichen**:

1. Gut geeignet ist eine **zweite Freigabe auf einem anderen Gerät**, zum Beispiel ein zweites NAS oder ein Ordner auf einem anderen Server.
2. Ein USB-Stick passt nur, wenn er immer am selben PC steckt und meist dort gearbeitet wird. An den anderen PCs ist er nicht erreichbar; die gelbe Leiste meldet das dort bei jeder Sicherung.
3. Fragen Sie, ob das Netzlaufwerk selbst gesichert wird (zum Beispiel Schnappschüsse auf dem NAS). Das ist ein guter zusätzlicher Schutz.
4. Den jährlichen Gesamtexport macht eine verantwortliche Person, zum Beispiel das Sekretariat nach der GV.

### Einmal ausprobieren

Spielen Sie nach dem Einrichten einmal eine Sicherung zurück: **Einstellungen** › **Datensicherung** › **Wiederherstellen** beim neuesten Eintrag. So wissen Sie im Ernstfall, wie es geht. Weil das Programm vorher den aktuellen Stand sichert, geht dabei nichts verloren. Siehe [Datensicherung und Wiederherstellen](08-sicherung-wiederherstellen.md).

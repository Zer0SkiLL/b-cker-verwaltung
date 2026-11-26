# Osternest – Design (kleines Vorhaben)

Kleine, versteckte Überraschungen für das Sekretariat. Der Hauptnutzer ist ein ehemaliger Bäckermeister mit Humor; das Umfeld ist professionell (Vorstand, GV).

## Absicht

Das Programm soll gelegentlich zum Schmunzeln bringen, mit Anspielungen auf den Bäckeralltag. Wer nicht danach sucht, merkt fast nichts.

## Ziele, Nicht-Ziele, Anti-Ziele

- **Ziele:** sechs Arten von Überraschungen (siehe unten), dezent und testbar.
- **Nicht-Ziele:** kein Handbuch-Kapitel und keine Einstellung zum Abschalten.
- **Anti-Ziele (Grundregeln):**
  - Nie Daten, Exporte, das PDF oder Ausdrucke verändern.
  - Nie die Arbeit blockieren; alles ist klein und nach wenigen Sekunden vorbei.
  - Nichts erscheint ungefragt, etwa während einer Präsentation an der GV.
  - Keine Gendersprache, immer das generische Maskulinum.

## Ansatz

Alles läuft in der Oberfläche, in `ui/src/lib/osternest/`. Das Modul besteht aus reinen Funktionen, die Datum und Uhrzeit als Eingabe bekommen und darum ohne Warten auf den Kalender testbar sind. Was sich das Programm merkt, liegt in `localStorage` (pro PC, mit try/catch, nie in der Vereinsdatei).

Verworfen wurden:

- **Im Datenkern:** Das würde die Vereinsdatei berühren und die Grundregeln verletzen.
- **Inhalte aus dem Internet:** Dafür bräuchte es einen Server, den es nicht gibt.

## Die Überraschungen

1. **Antworten in der Hilfe-Suche:** Begriffe wie „Kaffee“, „Feierabend“, „Ferien“, „Lohnerhöhung“, „Pause“, „Gipfeli“, „Osterei“ oder „Easter Egg“ zeigen eine zusätzliche Zeile mit einer Antwort. Echte Treffer bleiben darunter sichtbar.
2. **Bäckermeister-Modus:** Ein Klick auf die Version in **Über das Programm** zählt mit; nach 7 Klicks erscheint „Sie sind jetzt Bäckermeister.“ Neben dem Programmnamen in der linken Leiste steht dann 🥨. Nochmals 7 Klicks schalten den Modus aus. Der Zustand bleibt gespeichert.
3. **Besondere Tage:** Neben dem Programmnamen in der linken Leiste steht ein kleines Zeichen mit einem Tooltip. Es öffnet sich nichts von selbst. Ein besonderer Tag hat Vorrang vor der 🥨.

   | Zeitraum | Zeichen, Text |
   |---|---|
   | 2.–8. Januar | Dreikönig. Zusätzlich liegt eine winzige 👑 unten auf der Seite **Adressen**; ein Klick zeigt „Sie sind heute König.“ |
   | Schmutziger Donnerstag (Ostern − 52) bis Aschermittwoch (Ostern − 46) | Fasnachtschüechli |
   | Palmsonntag (Ostern − 7) bis Dienstag nach Ostern (Ostern + 2) | 🥚, mit dem Hinweis, dass es noch weitere Ostereier gibt |
   | 1. April | ein harmloser Aprilscherz, nur im Tooltip |
   | 25. Juli – 1. August | 1.-August-Weggli |
   | 16. Oktober | Welttag des Brotes |
   | 1.–24. Dezember | Zimtstern ✨; am **6. Dezember** hat Samichlaus mit dem Grittibänz Vorrang |

   Ostern wird mit der Gaussschen Osterformel (gregorianisch) berechnet.
4. **Spät und früh:** Erscheint auf **Adressen** der Hinweis „Keine Treffer“, steht ab 22 Uhr und vor 5 Uhr eine zusätzliche Zeile darunter, zum Beispiel „Schon wach? Alte Bäckergewohnheit, nehme ich an.“
5. **Meilenstein:**
   - Gezählt werden die aktiven Mitglieder.
   - Wird nach einer **Neuaufnahme** eine volle Hundertermarke erreicht, die noch nicht gefeiert wurde, fallen 3 Sekunden lang 🥐🥨🍞.
   - Der Regen hat `pointer-events: none`, und ein Klick beendet ihn.
   - Die erste Zählung (etwa nach dem Excel-Import) merkt sich das Programm still, ohne Feier.

## Geheimhaltung

- Kein Handbuch-Kapitel. Das ist eine bewusste Ausnahme von der Handbuch-Regel; die Prüfung in `handbook.test.ts` betrifft nur Seiten und bleibt davon unberührt.
- Nur ein Commit erscheint in den Release Notes: `feat: Der Ofen ist jetzt richtig vorgeheizt`. Alle übrigen sind `chore` oder `test`.
- Im Code steht ein ehrlicher Kommentar für spätere Entwickler.

## Dem Umsetzer überlassen

- die genauen Texte, in Schweizer Schreibweise und nicht anzüglich
- die Art der Animation, ohne neue Abhängigkeit
- wo genau die Neuaufnahme die Zählung anstösst

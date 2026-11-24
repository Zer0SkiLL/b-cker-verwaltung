# Kleinere Punkte aus dem Review von Plan 2 – Kurzentwurf

Datum: 2026-11-24 · Status: vom Auftraggeber freigegeben („jap").

## Zweck
Die beim Review von Plan 2 zurückgestellten Minors beheben, die im Alltag stören.

## Umfang (freigegeben)
1. Gespeicherte Gruppen und Label-Gruppen umbenennen (Name eindeutig, Pflichtfeld).
2. Teams: Bei fehlgeschlagenem Anlegen/Umbenennen bleibt die Eingabe; Auswahl verschwindet erst nach erfolgreichem Löschen.
3. Übernehmen schlägt fehl → Datei bleibt nur lesend geöffnet (statt „Keine Datei geöffnet").
4. Startseite: Öffnen erfolgreich, aber Pfad nicht speicherbar → trotzdem weiter, nur Hinweis.
5. Nur lesen: „Neuer Betrieb"/„Neue Person" gesperrt; ein gemeinsamer `readOnly`-Store, unbekannter Zustand = nur lesen.
6. Hinweis nach Absturz: Sperre von derselben Person/demselben Rechner oder älter als 24 h →
   „Wahrscheinlich nach einem Absturz – bitte ‚Übernehmen'".
7. Fehlermeldungen verschwinden nach der nächsten erfolgreichen Aktion (Detailseiten, Hinweisleiste).
8. „Neue Person anlegen und verknüpfen" direkt vom Betrieb aus (Nachname Pflicht, Vorname optional).

## Ansätze
Nur ein sinnvoller Weg: jede Korrektur an der Stelle, die sie betrifft (Core für Umbenennen,
App-Session für Sperre/Identität, UI für Anzeige). Neue Fachregeln bleiben im Core.

## Nicht-Ziele
Stilpunkt `dateField` (forms.ts), `LookupValue.sort` – ohne Wirkung für Nutzer.

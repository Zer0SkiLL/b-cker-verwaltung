# Osternest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Versteckte, dezente Überraschungen für das Sekretariat einbauen: besondere Tage, Bäckermeister-Modus, Antworten in der Hilfe-Suche, Hinweise spät und früh, Meilenstein.

**Architecture:** Reine Funktionen in `ui/src/lib/osternest/`, die Datum und Uhrzeit als Eingabe bekommen. Was sich das Programm merkt, liegt in `localStorage`. Dünne Einbindung in bestehende Komponenten, dazu eine neue Komponente für den Brot-Regen. Es gibt keine Änderung am Datenkern und keine neue Abhängigkeit.

**Tech Stack:** Svelte 5 (Runes), TypeScript, Vitest + Testing Library.

**Spec:** `docs/superpowers/specs/2026-11-26-osternest-design.md`

## Global Constraints

- Nie Daten, Exporte, das PDF oder Ausdrucke verändern. Keine Aufrufe schreibender `api.*`-Funktionen aus `osternest`.
- Nichts blockiert die Arbeit und nichts öffnet sich ungefragt; einzige Ausnahme ist der Meilenstein-Regen direkt nach einer Neuaufnahme (3 s, klickbar, `pointer-events: none`).
- Texte in Schweizer Schreibweise („ss“), ohne Gendersprache (generisches Maskulinum).
- `localStorage` immer in try/catch; ohne Speicher funktioniert alles, nur ohne Merken. Schlüssel: `osternest-baeckermeister`, `osternest-meilenstein`.
- Datumslogik mit lokalen Datumsteilen (`getFullYear/getMonth/getDate/getHours`), nie UTC.
- Kein Handbuch-Kapitel und keine Handbuch-Erwähnung.
- Commits: nur Task 3 als `feat: Der Ofen ist jetzt richtig vorgeheizt`, alle anderen als `chore:` oder `test:`.
- Am Anfang jeder `osternest`-Datei ein Kommentar: `// Osternest: versteckte Überraschungen fürs Sekretariat (bewusst nicht im Handbuch). Siehe docs/superpowers/specs/2026-11-26-osternest-design.md`.
- Abdeckung `ui/src/lib` bleibt ≥ 80 %.

## Review Focus

1. **Gesperrter Speicher** (`localStorage` wirft): Programm startet, Leiste und „Über“ funktionieren normal → Test in Task 3 und Task 5.
2. **Ostergrenzfälle:** frühestes (22.3.) und spätestes (25.4.) Ostern, Fasnacht im Schaltjahr-Februar → Tests in Task 1.
3. **Neuaufnahme schlägt fehl oder ist eine Änderung:** kein Regen; Zählfehler bleibt still (keine Fehlermeldung wegen eines Easter Eggs) → Tests in Task 5.
4. **Echte Suche bleibt echt:** „Pause“ oder „Ferien“ zeigt die Antwortzeile *und* die echten Treffer; „Kaffeemaschine“ löst nichts aus → Tests in Task 2 und Task 4.
5. **Mitternacht/Programm läuft über Tage:** Zeichen wechselt ohne Neustart (Uhr-Store tickt) → Test in Task 3.

---

### Task 1: Kalender der besonderen Tage

**Files:**
- Create: `ui/src/lib/osternest/calendar.ts`
- Test: `ui/src/lib/osternest/calendar.test.ts`

**Interfaces:**
- Produces:
  - `type SpecialDayId = 'dreikoenig' | 'fasnacht' | 'ostern' | 'april' | 'august' | 'brot' | 'samichlaus' | 'advent'`
  - `type SpecialDay = { id: SpecialDayId; icon: string; text: string }`
  - `easterSunday(year: number): Date` (lokale Mitternacht)
  - `specialDay(now: Date): SpecialDay | null`

- [ ] **Step 1: Failing tests** in `calendar.test.ts`:
  - `easterSunday`: 2008 → 23.3., 2011 → 24.4., 2026 → 5.4., 2027 → 28.3., 2038 → 25.4., 2285 → 22.3.
  - `specialDay(...)?.id`:
    - 2.1.2027 und 8.1.2027 → `dreikoenig`; 9.1.2027 → `null`; 1.1.2027 → `null`
    - 2026, Ostern 5.4.: 12.2.2026 (Ostern − 52) → `fasnacht`; 18.2.2026 (− 46) → `fasnacht`; 19.2.2026 → `null`
    - Schaltjahr 2028 (Ostern 16.4.): 24.2.2028 → `fasnacht`, 1.3.2028 → `fasnacht`, 2.3.2028 → `null`
    - 29.3.2026 (− 7) → `ostern`; 7.4.2026 (+ 2) → `ostern`; 8.4.2026 → `null`; 28.3.2026 → `null`
    - 1.4.2027 → `april`. 2026 fällt der 1.4. in die Osterzeit: `ostern` hat Vorrang (Test: 1.4.2026 → `ostern`)
    - 24.7.2027 → `null`; 25.7.2027 und 1.8.2027 → `august`; 2.8.2027 → `null`
    - 16.10.2027 → `brot`; 15.10.2027 → `null`
    - 1.12.2027 und 24.12.2027 → `advent`; 6.12.2027 → `samichlaus`; 25.12.2027 → `null`
    - Uhrzeit egal: `new Date(2027, 9, 16, 23, 59)` → `brot`
  - Jeder Eintrag hat nichtleeres `icon` und `text`; kein Text enthält „ß“ oder „:in“.

- [ ] **Step 2: Run** `pnpm --dir ui exec vitest run src/lib/osternest/calendar.test.ts` → FAIL (Modul fehlt)

- [ ] **Step 3: Implement.** `easterSunday` nach der anonymen gregorianischen Osterformel (Meeus/Jones/Butcher). `specialDay` vergleicht Tage über `new Date(y, m, d)` ohne Uhrzeit; die Reihenfolge der Prüfung legt den Vorrang fest: `samichlaus` vor `advent`, `ostern` vor `april`. Feste Texte:

  | id | icon | text |
  |---|---|---|
  | dreikoenig | ⭐ | Dreikönig: Wer hat heute den König im Kuchen? |
  | fasnacht | 🎭 | Fasnacht: Zeit für Fasnachtschüechli. |
  | ostern | 🥚 | Sie haben ein Osterei gefunden. Es gibt noch weitere. |
  | april | 🐟 | Wichtige Mitteilung: Ab heute gibt es nur noch Ruchbrot. |
  | august | 🇨🇭 | Bald 1. August: Die Weggli mit Fähnchen nicht vergessen. |
  | brot | 🍞 | Heute ist Welttag des Brotes. Sie wissen ja, wem man dafür dankt. |
  | samichlaus | 🎅 | Samichlaus: Haben alle Grittibänzen Rosinenaugen? |
  | advent | ✨ | Adventszeit: Hochsaison für Zimtsterne und Mailänderli. |

- [ ] **Step 4: Run** dasselbe → PASS

- [ ] **Step 5: Commit** `chore: Kalender für das Osternest`

### Task 2: Sprüche (Hilfe-Suche, spät und früh)

**Files:**
- Create: `ui/src/lib/osternest/sayings.ts`
- Test: `ui/src/lib/osternest/sayings.test.ts`

**Interfaces:**
- Produces:
  - `helpJoke(query: string): string | null`
  - `hourHint(now: Date): string | null`

- [ ] **Step 1: Failing tests:**
  - `helpJoke`:
    - `'Kaffee'`, `'  kaffee '`, `'KAFFEE'` → Text mit „Kaffee“
    - `'Kaffeemaschine'` → `null`; `''` → `null`; `'Austritt'` → `null`
    - Für jeden Begriff `kaffee, feierabend, ferien, lohnerhöhung, pause, gipfeli, osterei, easter egg` → nicht `null`
  - `hourHint`:
    - 21:59 → `null`; 22:00 → Spät-Text; 23:30 → Spät-Text
    - 0:15 → Spät-Text; 2:59 → Spät-Text
    - 3:00 → Früh-Text; 4:59 → Früh-Text; 5:00 → `null`

- [ ] **Step 2: Run** `pnpm --dir ui exec vitest run src/lib/osternest/sayings.test.ts` → FAIL

- [ ] **Step 3: Implement.** Der Vergleich normalisiert mit `trim().toLowerCase()` und prüft die ganze Eingabe gegen eine Tabelle. Feste Texte:
  - kaffee: „Kaffee finden Sie leider nicht im Handbuch. Die Kaffeemaschine steht vermutlich im Gang.“
  - feierabend: „Feierabend ist, wenn der Ofen kalt ist. Oder wenn alle Adressen stimmen.“
  - ferien: „Ferien: Vorher das Programm schliessen, damit die Vertretung ändern kann. Und dann gute Erholung!“
  - lohnerhöhung: „Dazu schweigt das Handbuch. Vielleicht an der nächsten GV?“
  - pause: „Eine Pause ist wie Teigruhe: Danach geht alles besser.“
  - gipfeli: „Gipfeli gibt es nicht im Programm, aber hoffentlich auf dem Tisch.“
  - osterei, easter egg: „Gut gesucht! Es gibt ein paar davon. Mehr wird nicht verraten.“
  - Spät (22:00–2:59): „Um diese Zeit backen nur noch die Bäcker.“
  - Früh (3:00–4:59): „Schon wach? Alte Bäckergewohnheit, nehme ich an.“

- [ ] **Step 4: Run** → PASS

- [ ] **Step 5: Commit** `chore: Sprüche für das Osternest`

### Task 3: Bäckermeister-Modus und Zeichen in der Leiste

**Files:**
- Create: `ui/src/lib/osternest/state.ts`
- Test: `ui/src/lib/osternest/state.test.ts`
- Modify: `ui/src/components/Sidebar.svelte` (`.brand`), `ui/src/pages/settings/About.svelte` (Versionszeile)
- Test: `ui/src/components/Sidebar.test.ts`, `ui/src/pages/settings/About.test.ts`

**Interfaces:**
- Consumes: `specialDay(now)` aus Task 1
- Produces:
  - `now: Readable<Date>`: tickt jede Minute
  - `bakerMode: Writable<boolean>`: Start aus `localStorage` `osternest-baeckermeister` (`'1'` = an)
  - `setBakerMode(on: boolean): void`: setzt und speichert
  - `VERSION_CLICKS = 7`
  - `countVersionClick(count: number): { count: number; toggled: boolean }`: bei 7 wird `count` 0 und `toggled` true
  - `badge(now: Date, baker: boolean): { icon: string; text: string } | null`: besonderer Tag vor 🥨 (Text „Bäckermeister-Modus. Gut Teig!“); sonst `null`

- [ ] **Step 1: Failing tests:**
  - `state.test.ts`:
    - `countVersionClick`: 0 → `{1,false}`; 6 → `{0,true}`
    - `setBakerMode(true)` speichert `'1'`; ein neues Laden liest es (Modul mit `vi.resetModules()` neu importieren)
    - `badge`: 16.10.2027 + baker → 🍞; 17.10.2027 + baker → 🥨; 17.10.2027 ohne baker → `null`
    - **Review Focus 1:** `localStorage.getItem/setItem` werfen → Import klappt, `bakerMode` ist `false`, `setBakerMode(true)` setzt den Store ohne Fehler
    - **Review Focus 5:** mit `vi.useFakeTimers()` und `vi.setSystemTime(15.10.2027 23:59)` liefert `now` nach `advanceTimersByTime(60_000)` den 16.10.
  - `Sidebar.test.ts`: mit `vi.setSystemTime(16.10.2027 10:00)` steht 🍞 mit `title` „Heute ist Welttag des Brotes. …“ neben „Bäckervereinigung“; an einem gewöhnlichen Tag ohne Modus ist kein Zeichen da.
  - `About.test.ts`:
    - 6 Klicks auf den Versionstext → kein „Sie sind jetzt Bäckermeister.“
    - 7. Klick → Text sichtbar, `bakerMode` ist `true`
    - weitere 7 Klicks → „Bäckermeister-Modus aus. Zurück an den Ofen.“, `bakerMode` ist `false`

- [ ] **Step 2: Run** `pnpm --dir ui exec vitest run src/lib/osternest src/components/Sidebar.test.ts src/pages/settings/About.test.ts` → FAIL

- [ ] **Step 3: Implement** `state.ts`. `now` ist ein `readable(new Date(), set => setInterval(() => set(new Date()), 60_000))`.

- [ ] **Step 4: Sidebar:** `<span class="osternest" title={b.text}>{b.icon}</span>` innerhalb `.brand`, aus `badge($now, $bakerMode)`. Kein Knopf und kein Fokus, nur ein Tooltip.

- [ ] **Step 5: About:**
  - Die Version steht in einem `<span>` mit `onclick`; die Klickzahl ist lokal in der Komponente.
  - Beim Umschalten erscheint der Text 4 s lang als `muted` neben der Version.
  - `svelte-ignore` für a11y mit Begründung „bewusst versteckt“.

- [ ] **Step 6: Run** Tests aus Step 2 → PASS; `pnpm --dir ui check` → 0 Fehler

- [ ] **Step 7: Commit** `feat: Der Ofen ist jetzt richtig vorgeheizt`

### Task 4: Antworten in der Hilfe-Suche, Hinweis spät/früh, Krone

**Files:**
- Modify: `ui/src/components/HelpPanel.svelte` (Suchansicht), `ui/src/pages/Addresses.svelte` („Keine Treffer“, Fusszeile)
- Test: `ui/src/components/HelpPanel.test.ts`, `ui/src/pages/Addresses.test.ts`

**Interfaces:**
- Consumes: `helpJoke`, `hourHint` (Task 2), `specialDay` (Task 1), `now` (Task 3)

- [ ] **Step 1: Failing tests:**
  - HelpPanel:
    - Suche „Kaffee“ zeigt den Kaffee-Text (Klasse `osternest`) und keinen „Keine Treffer“-Hinweis.
    - **Review Focus 4:** Suche „Ferien“ zeigt den Ferien-Text *und* die echten Treffer (mindestens ein Treffer-Knopf; das Handbuch erwähnt Ferien in „Einrichten“).
  - Addresses (Fake-Timer):
    - aktiver Filter ohne Treffer um 23:00 → „Keine Treffer für diesen Filter.“ und „Um diese Zeit backen nur noch die Bäcker.“
    - um 10:00 nur der normale Hinweis
    - am 6.1.2027 ein Knopf `aria-label="Krone"`; ein Klick zeigt „Sie sind heute König.“
    - am 10.1.2027 keine Krone

- [ ] **Step 2: Run** `pnpm --dir ui exec vitest run src/components/HelpPanel.test.ts src/pages/Addresses.test.ts` → FAIL

- [ ] **Step 3: Implement.**
  - HelpPanel: in der Suchansicht steht oberhalb der Treffer `{#if joke}<p class="osternest muted">{joke}</p>{/if}`; der „Keine Treffer“-Text erscheint nur ohne `joke`.
  - Addresses:
    - `hourHint($now)` als zweite `muted`-Zeile unter „Keine Treffer für diesen Filter.“
    - In der Fusszeile bei `specialDay($now)?.id === 'dreikoenig'` ein kleiner, unauffälliger Knopf 👑 (`aria-label="Krone"`, ohne Rahmen, `opacity: 0.5`), der den Text inline zeigt.

- [ ] **Step 4: Run** → PASS

- [ ] **Step 5: Commit** `chore: Osternest in Hilfe-Suche und Adressen`

### Task 5: Meilenstein mit Brot-Regen

**Files:**
- Create: `ui/src/lib/osternest/milestone.ts`, `ui/src/components/BreadRain.svelte`
- Test: `ui/src/lib/osternest/milestone.test.ts`, `ui/src/components/BreadRain.test.ts`
- Modify: `ui/src/components/MembershipEditor.svelte` (nach erfolgreichem `createMembership`), `ui/src/App.svelte` (`<BreadRain />` einmal einbinden)
- Test: `ui/src/components/MembershipEditor.test.ts`

**Interfaces:**
- Produces:
  - `nextMilestone(stored: number | null, count: number): { store: number; celebrate: number | null }`
  - `rain: Writable<number | null>`: die gefeierte Zahl oder `null`
  - `RAIN_MS = 3000`
  - `checkMilestone(countActive: () => Promise<number>): Promise<void>`: liest und schreibt `osternest-meilenstein`, setzt `rain`, schluckt jeden Fehler
  - `countActiveMembers(): Promise<number>`: `api.searchAddresses({ ...emptyFilter(), membership: 'Active' })` → `rows.length`

- [ ] **Step 1: Failing tests** `milestone.test.ts`:
  - `nextMilestone(null, 312)` → `{ store: 300, celebrate: null }` (erste Zählung still, etwa nach dem Import)
  - `nextMilestone(300, 399)` → `{300, null}`; `nextMilestone(300, 400)` → `{400, 400}`
  - `nextMilestone(400, 380)` → `{400, null}` (Austritte feiern nicht nochmal)
  - `nextMilestone(0, 100)` → `{100, 100}`; `nextMilestone(null, 50)` → `{0, null}`
  - `checkMilestone(async () => { throw new Error('x') })` → löst ohne Fehler auf, `rain` bleibt `null`
  - mit gespeichertem `'300'` und Zählung 400 → `rain` ist 400, gespeichert `'400'`
  - **Review Focus 1:** `localStorage` wirft → keine Ausnahme, kein Regen

- [ ] **Step 2: Failing tests:**
  - `BreadRain.test.ts` (Fake-Timer):
    - `rain.set(400)` zeigt „400 aktive Mitglieder!“
    - nach `RAIN_MS` ist es weg und `rain` ist `null`
    - ein `pointerdown` auf `window` beendet es sofort
  - `MembershipEditor.test.ts`, **Review Focus 3:**
    - erfolgreiches Neuanlegen ruft `checkMilestone` (gemockt über `vi.mock('../lib/osternest/milestone')`) einmal auf
    - Ändern (`existing` gesetzt) ruft es nicht auf
    - ein fehlgeschlagenes Anlegen (bestehender Test) ruft es nicht auf

- [ ] **Step 3: Run** `pnpm --dir ui exec vitest run src/lib/osternest/milestone.test.ts src/components/BreadRain.test.ts src/components/MembershipEditor.test.ts` → FAIL

- [ ] **Step 4: Implement** `milestone.ts`: `floor(count / 100) * 100`, gespeichert wird das Maximum aus Alt und Neu.

- [ ] **Step 5: Implement** `BreadRain.svelte`:
  - Overlay `position: fixed; inset: 0; pointer-events: none; z-index: 70`.
  - Etwa 24 Emojis aus 🥐🥨🍞 mit zufälliger `left`/`animation-delay`, CSS-Keyframes nach unten.
  - Dazu oben mittig ein Kärtchen „{n} aktive Mitglieder! 🥐“.
  - Bei `prefers-reduced-motion: reduce` nur das Kärtchen.
  - Ende nach `RAIN_MS` oder bei `pointerdown`/`keydown` auf `window`.

- [ ] **Step 6: MembershipEditor:** nach erfolgreichem `createMembership` (nicht bei Update) `void checkMilestone(countActiveMembers)`, vor `onDone()`; nicht awaiten.

- [ ] **Step 7: Run** Tests aus Step 3 → PASS

- [ ] **Step 8: Commit** `chore: Meilenstein im Osternest`

### Task 6: Gesamtprüfung

- [ ] **Step 1:** Ganze Prüfung laufen lassen:
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `pnpm --dir ui exec vitest run --coverage`
  - `pnpm --dir ui check`
  - `node --test 'scripts/*.test.mjs'`

  → alles grün, Abdeckung ≥ 80 %.
- [ ] **Step 2:** `grep -rn "ß\|:in\b" ui/src/lib/osternest ui/src/components/BreadRain.svelte` → keine Treffer.
- [ ] **Step 3:** `git log --oneline main..` → genau ein `feat:` (Task 3), Rest `chore:`/`test:`/`docs:`.
- [ ] **Step 4:** Den User fragen, ob gemergt wird. Ein Release gibt es nur mit seiner Zustimmung.

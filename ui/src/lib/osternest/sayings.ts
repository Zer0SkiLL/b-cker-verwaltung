// Osternest: versteckte Überraschungen fürs Sekretariat (bewusst nicht im Handbuch). Siehe docs/superpowers/specs/2026-11-26-osternest-design.md
// Antworten auf bestimmte Suchbegriffe in der Hilfe und Hinweise zu später oder früher Stunde.

const EGG = 'Gut gesucht! Es gibt ein paar davon. Mehr wird nicht verraten.';

const JOKES: Record<string, string> = {
  kaffee: 'Kaffee finden Sie leider nicht im Handbuch. Die Kaffeemaschine steht vermutlich im Gang.',
  feierabend: 'Feierabend ist, wenn der Ofen kalt ist. Oder wenn alle Adressen stimmen.',
  ferien: 'Ferien: Vorher das Programm schliessen, damit die Vertretung ändern kann. Und dann gute Erholung!',
  lohnerhöhung: 'Dazu schweigt das Handbuch. Vielleicht an der nächsten GV?',
  pause: 'Eine Pause ist wie Teigruhe: Danach geht alles besser.',
  gipfeli: 'Gipfeli gibt es nicht im Programm, aber hoffentlich auf dem Tisch.',
  osterei: EGG,
  'easter egg': EGG,
};

/** Antwort nur für den ganzen Begriff, damit echte Suchen wie „Kaffeemaschine“ unberührt bleiben. */
export function helpJoke(query: string): string | null {
  return JOKES[query.trim().toLowerCase()] ?? null;
}

/** 22:00–2:59 spät, 3:00–4:59 früh, sonst nichts. */
export function hourHint(now: Date): string | null {
  const hour = now.getHours();
  if (hour >= 22 || hour < 3) return 'Um diese Zeit backen nur noch die Bäcker.';
  if (hour < 5) return 'Schon wach? Alte Bäckergewohnheit, nehme ich an.';
  return null;
}

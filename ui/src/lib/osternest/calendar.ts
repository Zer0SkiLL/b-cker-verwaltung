// Osternest: versteckte Überraschungen fürs Sekretariat (bewusst nicht im Handbuch). Siehe docs/superpowers/specs/2026-11-26-osternest-design.md
// Besondere Tage im Bäckerjahr. Zeiträume statt einzelner Tage, weil viele davon Feiertage sind.

export type SpecialDayId = 'dreikoenig' | 'fasnacht' | 'ostern' | 'april' | 'august' | 'brot' | 'samichlaus' | 'advent';
export type SpecialDay = { id: SpecialDayId; icon: string; text: string };

const DAYS: Record<SpecialDayId, Omit<SpecialDay, 'id'>> = {
  dreikoenig: { icon: '⭐', text: 'Dreikönig: Wer hat heute den König im Kuchen?' },
  fasnacht: { icon: '🎭', text: 'Fasnacht: Zeit für Fasnachtschüechli.' },
  ostern: { icon: '🥚', text: 'Sie haben ein Osterei gefunden. Es gibt noch weitere.' },
  april: { icon: '🐟', text: 'Wichtige Mitteilung: Ab heute gibt es nur noch Ruchbrot.' },
  august: { icon: '🇨🇭', text: 'Bald 1. August: Die Weggli mit Fähnchen nicht vergessen.' },
  brot: { icon: '🍞', text: 'Heute ist Welttag des Brotes. Sie wissen ja, wem man dafür dankt.' },
  samichlaus: { icon: '🎅', text: 'Samichlaus: Haben alle Grittibänzen Rosinenaugen?' },
  advent: { icon: '✨', text: 'Adventszeit: Hochsaison für Zimtsterne und Mailänderli.' },
};

/** Ostersonntag (gregorianisch, anonyme Osterformel nach Meeus/Jones/Butcher), lokale Mitternacht. */
export function easterSunday(year: number): Date {
  const a = year % 19;
  const b = Math.floor(year / 100);
  const c = year % 100;
  const d = Math.floor(b / 4);
  const e = b % 4;
  const f = Math.floor((b + 8) / 25);
  const g = Math.floor((b - f + 1) / 3);
  const h = (19 * a + b - d - g + 15) % 30;
  const i = Math.floor(c / 4);
  const k = c % 4;
  const l = (32 + 2 * e + 2 * i - h - k) % 7;
  const m = Math.floor((a + 11 * h + 22 * l) / 451);
  const month = Math.floor((h + l - 7 * m + 114) / 31);
  const dayOfMonth = ((h + l - 7 * m + 114) % 31) + 1;
  return new Date(year, month - 1, dayOfMonth);
}

const MS_PER_DAY = 86_400_000;

/** Tage seit Ostern (negativ davor); über Datumsteile, damit Sommerzeit nichts verschiebt. */
function daysFromEaster(date: Date): number {
  const easter = easterSunday(date.getFullYear());
  const utc = (d: Date) => Date.UTC(d.getFullYear(), d.getMonth(), d.getDate());
  return Math.round((utc(date) - utc(easter)) / MS_PER_DAY);
}

function findId(date: Date): SpecialDayId | null {
  const month = date.getMonth() + 1;
  const dayOfMonth = date.getDate();
  const fromEaster = daysFromEaster(date);
  // Die Reihenfolge legt den Vorrang fest.
  if (month === 1 && dayOfMonth >= 2 && dayOfMonth <= 8) return 'dreikoenig';
  if (fromEaster >= -52 && fromEaster <= -46) return 'fasnacht';
  if (fromEaster >= -7 && fromEaster <= 2) return 'ostern';
  if (month === 4 && dayOfMonth === 1) return 'april';
  if ((month === 7 && dayOfMonth >= 25) || (month === 8 && dayOfMonth === 1)) return 'august';
  if (month === 10 && dayOfMonth === 16) return 'brot';
  if (month === 12 && dayOfMonth === 6) return 'samichlaus';
  if (month === 12 && dayOfMonth <= 24) return 'advent';
  return null;
}

export function specialDay(now: Date): SpecialDay | null {
  const id = findId(now);
  return id ? { id, ...DAYS[id] } : null;
}

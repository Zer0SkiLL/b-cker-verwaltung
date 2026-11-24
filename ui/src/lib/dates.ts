// Datum in der Oberfläche: TT.MM.JJJJ. Zwischen Rust und TS: ISO "YYYY-MM-DD".
import type { IsoDate } from './types';

export const DATE_ERROR = 'Ungültiges Datum (TT.MM.JJJJ)';

export type DateParse = { ok: true; iso: IsoDate | null } | { ok: false; error: string };

const pad = (n: number) => String(n).padStart(2, '0');

export function formatDate(iso: IsoDate | null): string {
  if (!iso) return '';
  const [y, m, d] = iso.split('-');
  return `${d}.${m}.${y}`;
}

/** Zeitpunkt (RFC 3339) in Ortszeit als TT.MM.JJJJ HH:MM. */
export function formatDateTime(rfc3339: string): string {
  const t = new Date(rfc3339);
  return `${pad(t.getDate())}.${pad(t.getMonth() + 1)}.${t.getFullYear()} ${pad(t.getHours())}:${pad(t.getMinutes())}`;
}

export function parseDate(input: string): DateParse {
  const text = input.trim();
  if (text === '') return { ok: true, iso: null };
  const match = /^(\d{1,2})\.(\d{1,2})\.(\d{4})$/.exec(text);
  if (!match) return { ok: false, error: DATE_ERROR };
  const [day, month, year] = [Number(match[1]), Number(match[2]), Number(match[3])];
  const date = new Date(Date.UTC(year, month - 1, day));
  const valid =
    date.getUTCFullYear() === year && date.getUTCMonth() === month - 1 && date.getUTCDate() === day;
  if (!valid) return { ok: false, error: DATE_ERROR };
  return { ok: true, iso: `${year}-${pad(month)}-${pad(day)}` };
}

export function todayIso(now: Date = new Date()): IsoDate {
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

import { easterSunday, specialDay, type SpecialDayId } from './calendar';

const day = (y: number, m: number, d: number, h = 12) => new Date(y, m - 1, d, h);
const ymd = (date: Date) => [date.getFullYear(), date.getMonth() + 1, date.getDate()];

test.each([
  [2008, [2008, 3, 23]],
  [2011, [2011, 4, 24]],
  [2026, [2026, 4, 5]],
  [2027, [2027, 3, 28]],
  [2038, [2038, 4, 25]],
  [2285, [2285, 3, 22]],
])('Ostern %i', (year, expected) => {
  const easter = easterSunday(year);
  expect(ymd(easter)).toEqual(expected);
  expect(easter.getHours()).toBe(0);
});

test.each<[string, Date, SpecialDayId | null]>([
  ['1.1. ist noch nichts', day(2027, 1, 1), null],
  ['Dreikönig Anfang', day(2027, 1, 2), 'dreikoenig'],
  ['Dreikönig Ende', day(2027, 1, 8), 'dreikoenig'],
  ['nach Dreikönig', day(2027, 1, 9), null],
  ['Schmutziger Donnerstag 2026', day(2026, 2, 12), 'fasnacht'],
  ['Aschermittwoch 2026', day(2026, 2, 18), 'fasnacht'],
  ['nach Fasnacht 2026', day(2026, 2, 19), null],
  ['Fasnacht Schaltjahr Anfang', day(2028, 2, 24), 'fasnacht'],
  ['Fasnacht Schaltjahr Ende', day(2028, 3, 1), 'fasnacht'],
  ['nach Fasnacht Schaltjahr', day(2028, 3, 2), null],
  ['vor Palmsonntag', day(2026, 3, 28), null],
  ['Palmsonntag', day(2026, 3, 29), 'ostern'],
  ['Dienstag nach Ostern', day(2026, 4, 7), 'ostern'],
  ['nach Ostern', day(2026, 4, 8), null],
  ['1. April in der Osterzeit: Ostern hat Vorrang', day(2026, 4, 1), 'ostern'],
  ['1. April', day(2027, 4, 1), 'april'],
  ['vor dem 1.-August-Zeitraum', day(2027, 7, 24), null],
  ['1.-August-Zeitraum Anfang', day(2027, 7, 25), 'august'],
  ['1. August', day(2027, 8, 1), 'august'],
  ['nach dem 1. August', day(2027, 8, 2), null],
  ['vor dem Welttag des Brotes', day(2027, 10, 15), null],
  ['Welttag des Brotes', day(2027, 10, 16), 'brot'],
  ['Welttag des Brotes spätabends', new Date(2027, 9, 16, 23, 59), 'brot'],
  ['Advent Anfang', day(2027, 12, 1), 'advent'],
  ['Samichlaus hat Vorrang', day(2027, 12, 6), 'samichlaus'],
  ['Heiligabend', day(2027, 12, 24), 'advent'],
  ['Weihnachten selbst ist frei', day(2027, 12, 25), null],
])('%s', (_name, date, expected) => {
  expect(specialDay(date)?.id ?? null).toBe(expected);
});

test('jeder besondere Tag hat Zeichen und Text in Schweizer Schreibweise, ohne Gendersprache', () => {
  const dates = [day(2027, 1, 3), day(2026, 2, 13), day(2026, 4, 2), day(2027, 4, 1), day(2027, 7, 30), day(2027, 10, 16), day(2027, 12, 6), day(2027, 12, 10)];
  const found = dates.map((d) => specialDay(d));
  expect(new Set(found.map((s) => s?.id)).size).toBe(8);
  for (const s of found) {
    expect(s?.icon).toBeTruthy();
    expect(s?.text).toBeTruthy();
    expect(s?.text).not.toMatch(/ß|:in\b|\*in/);
  }
});

import { formatDate, formatDateTime, parseDate } from './dates';

describe('formatDate', () => {
  test('ISO wird TT.MM.JJJJ', () => {
    expect(formatDate('2024-03-05')).toBe('05.03.2024');
  });
  test('leer bleibt leer', () => {
    expect(formatDate(null)).toBe('');
  });
});

describe('formatDateTime', () => {
  test('RFC 3339 wird TT.MM.JJJJ HH:MM in Ortszeit', () => {
    const local = new Date(2026, 10, 24, 8, 12);
    expect(formatDateTime(local.toISOString())).toBe('24.11.2026 08:12');
  });
});

describe('parseDate', () => {
  test('akzeptiert T.M.JJJJ', () => {
    expect(parseDate('5.3.2024')).toEqual({ ok: true, iso: '2024-03-05' });
  });
  test('akzeptiert TT.MM.JJJJ mit Leerzeichen', () => {
    expect(parseDate(' 29.02.2024 ')).toEqual({ ok: true, iso: '2024-02-29' });
  });
  test('leer ergibt null', () => {
    expect(parseDate('  ')).toEqual({ ok: true, iso: null });
  });
  test.each(['31.02.2024', '1.2.24', '2024', 'abc', '29.02.2023', '00.01.2024'])(
    '%s ist ungültig',
    (input) => {
      expect(parseDate(input)).toEqual({ ok: false, error: 'Ungültiges Datum (TT.MM.JJJJ)' });
    },
  );
});

import { todayIso } from './dates';

test('todayIso liefert das lokale Datum als ISO', () => {
  const t = new Date(2026, 0, 5, 23, 30);
  expect(todayIso(t)).toBe('2026-01-05');
});

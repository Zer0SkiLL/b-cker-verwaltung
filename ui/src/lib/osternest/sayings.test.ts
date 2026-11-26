import { helpJoke, hourHint } from './sayings';

const at = (h: number, m = 0) => new Date(2027, 2, 10, h, m);
const LATE = 'Um diese Zeit backen nur noch die Bäcker.';
const EARLY = 'Schon wach? Alte Bäckergewohnheit, nehme ich an.';

test('helpJoke erkennt den Begriff unabhängig von Gross/Klein und Leerzeichen', () => {
  for (const query of ['Kaffee', '  kaffee ', 'KAFFEE']) expect(helpJoke(query)).toMatch(/Kaffee/);
});

test('helpJoke reagiert nur auf den ganzen Begriff', () => {
  expect(helpJoke('Kaffeemaschine')).toBeNull();
  expect(helpJoke('')).toBeNull();
  expect(helpJoke('Austritt')).toBeNull();
});

test.each(['kaffee', 'feierabend', 'ferien', 'lohnerhöhung', 'pause', 'gipfeli', 'osterei', 'easter egg'])(
  'helpJoke kennt „%s“',
  (word) => {
    const joke = helpJoke(word);
    expect(joke).toBeTruthy();
    expect(joke).not.toMatch(/ß|:in\b/);
  },
);

test.each([
  [at(21, 59), null],
  [at(22), LATE],
  [at(23, 30), LATE],
  [at(0, 15), LATE],
  [at(2, 59), LATE],
  [at(3), EARLY],
  [at(4, 59), EARLY],
  [at(5), null],
  [at(12), null],
])('hourHint %s', (date, expected) => {
  expect(hourHint(date)).toBe(expected);
});

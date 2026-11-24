import { test } from 'node:test';
import assert from 'node:assert/strict';
import { preflight } from './release.mjs';

const ok = { branch: 'main', clean: true, current: '0.1.0', next: '0.2.0' };

test('preflight: alles in Ordnung', () => {
  assert.deepEqual(preflight(ok), []);
});

test('preflight: Version nicht grösser', () => {
  assert.deepEqual(preflight({ ...ok, next: '0.1.0' }), [
    'Die neue Version 0.1.0 muss grösser sein als die aktuelle 0.1.0.',
  ]);
});

test('preflight: unsauberer Arbeitsbaum und falscher Branch', () => {
  assert.deepEqual(preflight({ ...ok, clean: false, branch: 'feat/x' }), [
    'Releases nur von „main“ (aktuell: feat/x).',
    'Es gibt nicht committete Änderungen.',
  ]);
});

test('preflight: ungültige Version', () => {
  assert.deepEqual(preflight({ ...ok, next: '2' }), ['Ungültige Version: 2 (erwartet z.B. 0.2.0)']);
});

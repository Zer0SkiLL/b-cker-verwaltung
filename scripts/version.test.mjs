import { test } from 'node:test';
import assert from 'node:assert/strict';
import { isGreater, parseSemver, setCargoVersion, setJsonVersion } from './version.mjs';

test('parseSemver liest x.y.z und lehnt Unvollständiges ab', () => {
  assert.deepEqual(parseSemver('1.2.3'), { major: 1, minor: 2, patch: 3 });
  assert.throws(() => parseSemver('1.2'), /Ungültige Version/);
  assert.throws(() => parseSemver('v1.2.3'), /Ungültige Version/);
});

test('isGreater vergleicht numerisch', () => {
  assert.equal(isGreater('0.10.0', '0.9.0'), true);
  assert.equal(isGreater('0.1.0', '0.1.0'), false);
  assert.equal(isGreater('0.1.0', '0.1.1'), false);
  assert.equal(isGreater('1.0.0', '0.99.99'), true);
});

test('setCargoVersion ändert nur die Paketversion', () => {
  const cargo = [
    '[package]',
    'name = "verwaltung_app"',
    'version = "0.1.0"',
    '',
    '[dependencies]',
    'tauri = { version = "2", features = [] }',
    'serde = { version = "1" }',
    '',
  ].join('\n');
  const next = setCargoVersion(cargo, '0.2.0');
  assert.match(next, /^version = "0\.2\.0"$/m);
  assert.match(next, /tauri = \{ version = "2"/);
  assert.equal(next.split('\n').length, cargo.split('\n').length);
});

test('setJsonVersion erhält die Formatierung', () => {
  const json = '{\n  "name": "x",\n  "version": "0.1.0",\n  "private": true\n}\n';
  assert.equal(setJsonVersion(json, '0.1.1'), '{\n  "name": "x",\n  "version": "0.1.1",\n  "private": true\n}\n');
});

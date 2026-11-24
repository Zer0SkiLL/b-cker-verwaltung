import { test } from 'node:test';
import assert from 'node:assert/strict';
import { bumpedContents, preflight, recoveryHint } from './release.mjs';

const ok = { branch: 'main', clean: true, current: '0.1.0', next: '0.2.0', upToDate: true, tagExists: false };

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

test('preflight: lokaler Stand weicht von origin/main ab, Tag existiert schon', () => {
  assert.deepEqual(preflight({ ...ok, upToDate: false, tagExists: true }), [
    'Lokales „main“ entspricht nicht „origin/main“ (zuerst pull bzw. push).',
    'Der Tag v0.2.0 existiert bereits.',
  ]);
});

test('bumpedContents berechnet alle Dateien vor dem Schreiben', () => {
  const files = {
    'crates/app/Cargo.toml': '[package]\nname = "x"\nversion = "0.1.0"\n',
    'crates/app/tauri.conf.json': '{\n  "version": "0.1.0"\n}\n',
    'ui/package.json': '{\n  "version": "0.1.0"\n}\n',
    'package.json': '{\n  "version": "0.1.0"\n}\n',
    'CHANGELOG.md': '# Änderungen\n\n## [0.1.0] – 24.11.2026\n\nErste Version.\n',
  };
  const next = bumpedContents(files, '0.2.0', '## [0.2.0] – 01.12.2026\n\n### Neu\n- X\n');
  assert.match(next['crates/app/Cargo.toml'], /version = "0\.2\.0"/);
  assert.match(next['ui/package.json'], /"version": "0\.2\.0"/);
  assert.ok(next['CHANGELOG.md'].indexOf('[0.2.0]') < next['CHANGELOG.md'].indexOf('[0.1.0]'));
  assert.equal(files['package.json'], '{\n  "version": "0.1.0"\n}\n', 'Eingabe bleibt unverändert');
});

test('recoveryHint nennt den passenden Befehl', () => {
  assert.match(recoveryHint('files', '0.2.0'), /git checkout -- \./);
  assert.match(recoveryHint('tag', '0.2.0'), /git reset --hard HEAD~1/);
  assert.match(recoveryHint('push', '0.2.0'), /git push --atomic origin main v0\.2\.0/);
});

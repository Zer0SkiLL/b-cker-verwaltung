import { test } from 'node:test';
import assert from 'node:assert/strict';
import { extractSection } from './notes.mjs';

const changelog = `# Änderungen

## [0.2.0] – 05.01.2027

### Neu
- Mailvorlagen

## [0.1.0] – 24.11.2026

Erste Version.
`;

test('extractSection liefert einen mittleren Abschnitt ohne Überschrift', () => {
  assert.equal(extractSection(changelog, '0.2.0'), '### Neu\n- Mailvorlagen');
});

test('extractSection liefert den letzten Abschnitt', () => {
  assert.equal(extractSection(changelog, '0.1.0'), 'Erste Version.');
});

test('extractSection: fehlende Version ergibt null', () => {
  assert.equal(extractSection(changelog, '9.9.9'), null);
});

import { insertSection } from './notes.mjs';

test('insertSection setzt den neuen Abschnitt unter die Hauptüberschrift', () => {
  const next = insertSection(changelog, '## [0.3.0] – 01.02.2027\n\n### Behoben\n- X\n');
  assert.ok(next.startsWith('# Änderungen\n\n## [0.3.0] – 01.02.2027\n\n### Behoben\n- X\n\n## [0.2.0]'));
});

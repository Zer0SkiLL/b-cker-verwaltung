import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkHandbook, readToc } from './handbuch.mjs';
import { buildHandbookHtml, findChrome } from './handbuch-pdf.mjs';

function miniHandbook({ files, toc, images = [] }) {
  const dir = mkdtempSync(join(tmpdir(), 'hb-'));
  mkdirSync(join(dir, 'bilder'));
  for (const [name, text] of Object.entries(files)) writeFileSync(join(dir, name), text);
  for (const name of images) writeFileSync(join(dir, 'bilder', name), 'png');
  writeFileSync(join(dir, 'inhalt.json'), JSON.stringify(toc));
  return dir;
}
const toc1 = (...datei) => [{ teil: 'T', kapitel: datei.map((d) => ({ datei: d, titel: d })) }];

test('fehlendes Bild wird gemeldet', () => {
  const dir = miniHandbook({ files: { '01-a.md': '# A\n![x](bilder/x.png)' }, toc: toc1('01-a.md') });
  assert.deepEqual(checkHandbook(dir), ['Bild fehlt: bilder/x.png (in 01-a.md)']);
});

test('unbenutztes Bild wird gemeldet', () => {
  const dir = miniHandbook({ files: { '01-a.md': '# A' }, toc: toc1('01-a.md'), images: ['y.png'] });
  assert.deepEqual(checkHandbook(dir), ['Bild wird nicht verwendet: bilder/y.png']);
});

test('kaputter Kapitel-Link wird gemeldet', () => {
  const dir = miniHandbook({ files: { '01-a.md': '# A\n[z](99-z.md#teil)' }, toc: toc1('01-a.md') });
  assert.deepEqual(checkHandbook(dir), ['Link-Ziel fehlt: 99-z.md (in 01-a.md)']);
});

test('Datei ohne Eintrag im Inhaltsverzeichnis wird gemeldet', () => {
  const dir = miniHandbook({ files: { '01-a.md': '# A', '02-b.md': '# B' }, toc: toc1('01-a.md') });
  assert.deepEqual(checkHandbook(dir), ['Nicht im Inhaltsverzeichnis: 02-b.md']);
});

test('Eintrag ohne Datei wird gemeldet', () => {
  const dir = miniHandbook({ files: { '01-a.md': '# A' }, toc: toc1('01-a.md', '03-c.md') });
  assert.deepEqual(checkHandbook(dir), ['Datei fehlt: 03-c.md']);
});

test('das echte Handbuch ist stimmig', () => {
  assert.deepEqual(checkHandbook('docs/handbuch'), []);
  assert.equal(readToc('docs/handbuch').length, 3);
});

test('HTML: Titelseite, Inhaltsverzeichnis, Reihenfolge, Links und Bilder', () => {
  const toc = [
    { teil: 'Erste Schritte', kapitel: [{ datei: '01-a.md', titel: 'Willkommen' }] },
    { teil: 'Abläufe', kapitel: [{ datei: '05-austritt.md', titel: 'Austritt' }] },
  ];
  const chapters = {
    '01-a.md': '# Willkommen\n\nSiehe [Austritt](05-austritt.md#schritt) und [Web](https://x.ch).\n\n![Bild](bilder/a.png)',
    '05-austritt.md': '# Austritt\n\nText.',
  };
  const html = buildHandbookHtml({
    toc,
    readChapter: (datei) => chapters[datei],
    imageDataUri: (name) => (name === 'a.png' ? 'data:image/png;base64,QUJD' : null),
    version: '0.2.0',
  });
  assert.match(html, /Version 0\.2\.0/);
  assert.ok(html.indexOf('Erste Schritte') < html.indexOf('Abläufe'));
  assert.equal(html.match(/class="teil"/g).length, 2);
  assert.match(html, /href="#kapitel-05-austritt"/);
  assert.match(html, /id="kapitel-05-austritt"/);
  assert.doesNotMatch(html, /https:\/\/x\.ch/);
  assert.match(html, /Web/);
  assert.match(html, /src="data:image\/png;base64,QUJD"/);
  assert.match(html, /<a href="#kapitel-01-a">Willkommen<\/a>/);
});

test('findChrome nimmt CHROME, wenn die Datei existiert', () => {
  const dir = mkdtempSync(join(tmpdir(), 'chrome-'));
  const fake = join(dir, 'chrome');
  writeFileSync(fake, '');
  assert.equal(findChrome({ CHROME: fake }), fake);
  assert.notEqual(findChrome({ CHROME: join(dir, 'fehlt') }), join(dir, 'fehlt'));
});

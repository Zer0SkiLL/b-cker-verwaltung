import { renderMarkdown } from './markdown';

test('Überschriften und Listen werden HTML', () => {
  const html = renderMarkdown('## [0.1.0]\n\n- Erste Version');
  expect(html).toContain('<h2>');
  expect(html).toContain('<li>Erste Version</li>');
});

test('Skripte und Ereignis-Attribute werden entfernt', () => {
  const html = renderMarkdown('Hallo <script>alert(1)</script><img src="x" onerror="alert(2)">');
  expect(html).not.toContain('<script');
  expect(html).not.toContain('onerror');
});

test('Links verlassen das Programm nicht (kein href)', () => {
  const html = renderMarkdown('Siehe [Repo](https://example.org) und https://example.org/x');
  expect(html).not.toContain('href');
  expect(html).toContain('Repo');
});

test('ohne Option werden auch Kapitel-Links entfernt', () => {
  const html = renderMarkdown('[a](https://x.ch) [b](05-austritt.md)');
  expect(html).not.toContain('href');
});

test('mit chapterLinks bleiben nur relative Kapitel-Links erhalten', () => {
  const html = renderMarkdown(
    '[b](05-austritt.md#schritt) [a](https://x.ch) [c](javascript:alert(1)) [d](../x.md)',
    { chapterLinks: true },
  );
  expect(html).toContain('href="05-austritt.md#schritt"');
  expect(html.match(/href=/g)).toHaveLength(1);
});

test('Optionen wirken nicht auf den nächsten Aufruf', () => {
  renderMarkdown('[b](05-austritt.md)', { chapterLinks: true });
  expect(renderMarkdown('[b](05-austritt.md)')).not.toContain('href');
});

test('Bilder werden umgeschrieben, unbekannte zeigen den Alt-Text', () => {
  const imageUrl = (s: string) => (s === 'bilder/a.png' ? '/assets/a.png' : null);
  const html = renderMarkdown('![Adressen](bilder/a.png) ![Fehlt hier](bilder/b.png)', { imageUrl });
  expect(html).toContain('src="/assets/a.png"');
  expect(html).not.toContain('bilder/b.png');
  expect(html).toContain('Fehlt hier');
  const div = document.createElement('div');
  div.innerHTML = html;
  expect(div.textContent).toContain('Fehlt hier');
});

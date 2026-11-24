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

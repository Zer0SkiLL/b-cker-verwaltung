// Markdown (CHANGELOG, Hilfe) sicher als HTML darstellen.
import DOMPurify from 'dompurify';
import { marked } from 'marked';

// Links würden das Programmfenster durch die Webseite ersetzen (ohne Weg zurück): Ziel entfernen, Text bleibt.
DOMPurify.addHook('afterSanitizeAttributes', (node) => {
  if (node.tagName === 'A') node.removeAttribute('href');
});

export function renderMarkdown(markdown: string): string {
  const html = marked.parse(markdown, { async: false, gfm: true }) as string;
  return DOMPurify.sanitize(html);
}

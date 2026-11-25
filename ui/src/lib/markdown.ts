// Markdown (CHANGELOG, Handbuch) sicher als HTML darstellen.
import DOMPurify from 'dompurify';
import { marked } from 'marked';

export type MarkdownOptions = {
  /** Relative Kapitel-Links (z.B. „05-austritt.md#schritt“) behalten; alle anderen Links verlieren ihr Ziel. */
  chapterLinks?: boolean;
  /** Bildpfad aus dem Markdown → eingebundene URL; null = Bild unbekannt (Alt-Text wird gezeigt). */
  imageUrl?: (src: string) => string | null;
};

const CHAPTER_LINK = /^[0-9a-z-]+\.md(#[0-9a-z-]+)?$/;
const MISSING = 'data-bild-fehlt';

// Externe Links würden das Programmfenster durch die Webseite ersetzen (ohne Weg zurück):
// Ziel entfernen, Text bleibt. Der Hook gilt nur für den jeweiligen Aufruf.
function adjust(node: Element, options: MarkdownOptions): void {
  if (node.tagName === 'A') {
    const href = node.getAttribute('href') ?? '';
    if (!(options.chapterLinks && CHAPTER_LINK.test(href))) node.removeAttribute('href');
  }
  if (node.tagName === 'IMG' && options.imageUrl) {
    const url = options.imageUrl(node.getAttribute('src') ?? '');
    if (url === null) {
      node.removeAttribute('src');
      node.setAttribute(MISSING, '');
    } else {
      node.setAttribute('src', url);
    }
  }
}

// Unbekannte Bilder als Text zeigen statt als kaputtes Symbol.
function replaceMissingImages(html: string): string {
  if (!html.includes(MISSING)) return html;
  const template = document.createElement('template');
  template.innerHTML = html;
  for (const img of template.content.querySelectorAll(`img[${MISSING}]`)) {
    const span = document.createElement('span');
    span.className = 'bild-fehlt';
    span.textContent = img.getAttribute('alt') ?? 'Bild';
    img.replaceWith(span);
  }
  return template.innerHTML;
}

export function renderMarkdown(markdown: string, options: MarkdownOptions = {}): string {
  const html = marked.parse(markdown, { async: false, gfm: true }) as string;
  DOMPurify.addHook('afterSanitizeAttributes', (node) => adjust(node as Element, options));
  try {
    return replaceMissingImages(DOMPurify.sanitize(html));
  } finally {
    DOMPurify.removeHook('afterSanitizeAttributes');
  }
}

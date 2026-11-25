// Handbuch-Kapitel aus docs/handbuch: ins Programm eingebunden (offline, passend zur Version).
import toc from '../../../docs/handbuch/inhalt.json';
import { renderMarkdown } from './markdown';
import type { Route } from './router';

export type Chapter = { slug: string; title: string; part: string; markdown: string };
export type SearchHit = { slug: string; title: string; snippet: string };
type Toc = { teil: string; kapitel: { datei: string; titel: string }[] }[];

const REPO = 'https://github.com/Zer0SkiLL/b-cker-verwaltung';
const SNIPPET_BEFORE = 50;
const SNIPPET_AFTER = 70;

const files = import.meta.glob('../../../docs/handbuch/*.md', {
  eager: true,
  query: '?raw',
  import: 'default',
}) as Record<string, string>;
const images = import.meta.glob('../../../docs/handbuch/bilder/*.png', {
  eager: true,
  query: '?url',
  import: 'default',
}) as Record<string, string>;

const baseName = (path: string) => path.slice(path.lastIndexOf('/') + 1);
const byName = (record: Record<string, string>) =>
  new Map(Object.entries(record).map(([path, value]) => [baseName(path), value]));
const markdownByFile = byName(files);
const imageByFile = byName(images);

export const CHAPTERS: Chapter[] = (toc as Toc).flatMap((part) =>
  part.kapitel.map((k) => ({
    slug: k.datei.replace(/\.md$/, ''),
    title: k.titel,
    part: part.teil,
    markdown: markdownByFile.get(k.datei) ?? '',
  })),
);

export const PARTS: { title: string; chapters: Chapter[] }[] = (toc as Toc).map((part) => ({
  title: part.teil,
  chapters: CHAPTERS.filter((c) => c.part === part.teil),
}));

const PAGE_CHAPTERS: Record<Exclude<Route['page'], 'settings'>, string> = {
  start: '02-start-und-datei',
  addresses: '10-adressen',
  organisation: '11-betrieb',
  person: '12-person',
  groups: '13-gruppen',
  teams: '14-teams',
};
const SETTINGS_CHAPTERS = {
  catalog: '15-listen-und-labels',
  backup: '16-datensicherung',
  import: '17-excel-import',
  export: '18-gesamtexport',
  about: '19-ueber-das-programm',
} as const;

export function chapterForRoute(route: Route): string {
  if (route.page === 'settings') return SETTINGS_CHAPTERS[route.tab ?? 'catalog'];
  return PAGE_CHAPTERS[route.page];
}

export function findChapter(slug: string): Chapter | undefined {
  return CHAPTERS.find((c) => c.slug === slug);
}

export function slugFromHref(href: string): string | null {
  const match = /^([0-9a-z-]+)\.md(#[0-9a-z-]+)?$/.exec(href);
  return match && findChapter(match[1]) ? match[1] : null;
}

export function pdfUrl(version: string): string {
  return `${REPO}/releases/download/v${version}/Handbuch.pdf`;
}

export function renderChapter(slug: string): string {
  const chapter = findChapter(slug);
  if (!chapter) return '<p>Kapitel nicht gefunden.</p>';
  const imageUrl = (src: string) =>
    src.startsWith('bilder/') ? imageByFile.get(src.slice('bilder/'.length)) ?? null : null;
  return renderMarkdown(chapter.markdown, { chapterLinks: true, imageUrl });
}

/** Text ohne Markdown-Zeichen (für Suche und Ausschnitte). */
function plainText(markdown: string): string {
  return markdown
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/[#*_`>|]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

const UMLAUTS: Record<string, string> = { ä: 'a', ö: 'o', ü: 'u', ß: 'ss' };

/** Normalisiert für die Suche und merkt sich pro Zeichen die Position im Original. */
function normalize(text: string): { value: string; origin: number[] } {
  let value = '';
  const origin: number[] = [];
  for (let i = 0; i < text.length; i += 1) {
    const lower = text[i].toLowerCase();
    // „ae“, „oe“, „ue“ zählen wie „a“, „o“, „u“ (wie ä, ö, ü).
    if (lower === 'e' && /[aou]/.test(value[value.length - 1] ?? '') && /[aou]/i.test(text[i - 1])) continue;
    const mapped = UMLAUTS[lower] ?? lower;
    value += mapped;
    for (let k = 0; k < mapped.length; k += 1) origin.push(i);
  }
  return { value, origin };
}

function snippet(text: string, start: number, end: number): string {
  const from = Math.max(0, start - SNIPPET_BEFORE);
  const to = Math.min(text.length, end + SNIPPET_AFTER);
  return `${from > 0 ? '…' : ''}${text.slice(from, to).trim()}${to < text.length ? '…' : ''}`;
}

export function searchChapters(query: string, chapters: Chapter[] = CHAPTERS): SearchHit[] {
  const needle = normalize(query.trim()).value;
  if (!needle) return [];
  return chapters.flatMap((chapter) => {
    const text = plainText(chapter.markdown);
    const body = normalize(text);
    const at = body.value.indexOf(needle);
    if (at < 0 && !normalize(chapter.title).value.includes(needle)) return [];
    const shown =
      at < 0 ? snippet(text, 0, 0) : snippet(text, body.origin[at], body.origin[at + needle.length - 1] + 1);
    return [{ slug: chapter.slug, title: chapter.title, snippet: shown }];
  });
}

// Handbuch als PDF: alle Kapitel zu einer HTML-Datei zusammensetzen und mit Chrome headless drucken.
// Aufruf: node scripts/handbuch-pdf.mjs --out Handbuch.pdf [--version 0.2.0]
import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { marked } from 'marked';
import { readToc } from './handbuch.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const DIR = join(ROOT, 'docs', 'handbuch');

const CSS = `
@page { size: A4; margin: 2cm; @bottom-center { content: counter(page); font-size: 9pt; color: #666; } }
body { font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif; font-size: 11pt; line-height: 1.45; color: #1f2328; }
.titel { height: 24cm; display: flex; flex-direction: column; justify-content: center; text-align: center; }
.titel h1 { font-size: 28pt; margin: 0 0 0.5cm; border: none; }
.inhalt { break-before: page; }
.inhalt ul { list-style: none; padding-left: 0.5cm; }
.teil { break-before: page; font-size: 20pt; color: #8a4b14; }
.kapitel { break-before: page; }
.kapitel h1 { font-size: 16pt; border-bottom: 1px solid #d9d2c5; padding-bottom: 0.15cm; }
img { max-width: 100%; border: 1px solid #d9d2c5; border-radius: 3px; break-inside: avoid; }
a { color: #8a4b14; text-decoration: none; }
`;

const escape = (text) => text.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const anchor = (datei) => `kapitel-${datei.replace(/\.md$/, '')}`;

/** Kapitel-Markdown → HTML: Kapitel-Links werden Sprungmarken, externe Links reiner Text, Bilder eingebettet. */
function chapterHtml(markdown, imageDataUri) {
  const renderer = new marked.Renderer();
  renderer.link = ({ href, tokens }) => {
    const text = renderer.parser.parseInline(tokens);
    const match = /^([0-9a-z-]+\.md)(#[0-9a-z-]+)?$/.exec(href ?? '');
    return match ? `<a href="#${anchor(match[1])}">${text}</a>` : text;
  };
  renderer.image = ({ href, text }) => {
    const name = (href ?? '').replace(/^bilder\//, '');
    const uri = imageDataUri(name);
    return uri ? `<img src="${uri}" alt="${escape(text)}">` : `<em>[${escape(text)}]</em>`;
  };
  return marked.parse(markdown, { async: false, gfm: true, renderer });
}

export function buildHandbookHtml({ toc, readChapter, imageDataUri, version }) {
  const contents = toc
    .map((t) => `<li><strong>${escape(t.teil)}</strong><ul>${t.kapitel
      .map((k) => `<li><a href="#${anchor(k.datei)}">${escape(k.titel)}</a></li>`)
      .join('')}</ul></li>`)
    .join('');
  const body = toc
    .map((t) => `<h1 class="teil">${escape(t.teil)}</h1>${t.kapitel
      .map((k) => `<section class="kapitel" id="${anchor(k.datei)}">${chapterHtml(readChapter(k.datei), imageDataUri)}</section>`)
      .join('')}`)
    .join('');
  return `<!doctype html><html lang="de-CH"><head><meta charset="utf-8"><title>Handbuch Mitgliederverwaltung</title><style>${CSS}</style></head><body>
<div class="titel"><h1>Mitgliederverwaltung</h1><p>Benutzerhandbuch</p><p>Version ${escape(version)}</p></div>
<nav class="inhalt"><h2>Inhalt</h2><ul>${contents}</ul></nav>
${body}
</body></html>`;
}

const CHROME_PATHS = [
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
];

export function findChrome(env = process.env) {
  return [env.CHROME, ...CHROME_PATHS].find((p) => p && existsSync(p)) ?? null;
}

function arg(args, name) {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : undefined;
}

function main(args) {
  const out = arg(args, '--out');
  if (!out) {
    console.error('Aufruf: node scripts/handbuch-pdf.mjs --out <pfad> [--version X]');
    process.exit(1);
  }
  const version = arg(args, '--version') ?? JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8')).version;
  const chrome = findChrome();
  if (!chrome) {
    console.error('Chrome nicht gefunden. Pfad mit CHROME=… angeben.');
    process.exit(1);
  }
  const html = buildHandbookHtml({
    toc: readToc(DIR),
    readChapter: (datei) => readFileSync(join(DIR, datei), 'utf8'),
    imageDataUri: (name) => {
      const file = join(DIR, 'bilder', name);
      return existsSync(file) ? `data:image/png;base64,${readFileSync(file).toString('base64')}` : null;
    },
    version,
  });
  const htmlFile = join(mkdtempSync(join(tmpdir(), 'handbuch-')), 'handbuch.html');
  writeFileSync(htmlFile, html);
  const target = resolve(out);
  execFileSync(chrome, [
    '--headless=new', '--disable-gpu', '--no-sandbox', '--no-pdf-header-footer',
    `--print-to-pdf=${target}`, pathToFileURL(htmlFile).href,
  ], { stdio: 'inherit' });
  if (!existsSync(target)) {
    console.error('PDF wurde nicht erzeugt.');
    process.exit(1);
  }
  console.log(`Handbuch ${version}: ${target}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));

// Abschnitte aus CHANGELOG.md lesen bzw. einfügen. CLI: node scripts/notes.mjs 0.1.0
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function extractSection(changelog, version) {
  const lines = changelog.split('\n');
  const start = lines.findIndex((l) => new RegExp(`^## \\[${escape(version)}\\]`).test(l));
  if (start === -1) return null;
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((l) => l.startsWith('## '));
  return (end === -1 ? rest : rest.slice(0, end)).join('\n').trim();
}

/** Fügt einen neuen Abschnitt direkt nach der Hauptüberschrift (# …) ein. */
export function insertSection(changelog, section) {
  const lines = changelog.split('\n');
  const headerEnd = lines.findIndex((l) => l.startsWith('## '));
  const head = (headerEnd === -1 ? lines : lines.slice(0, headerEnd)).join('\n').trimEnd();
  const tail = headerEnd === -1 ? '' : lines.slice(headerEnd).join('\n');
  return `${head}\n\n${section.trim()}\n\n${tail}`.trimEnd() + '\n';
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const version = process.argv[2];
  const changelog = readFileSync(new URL('../CHANGELOG.md', import.meta.url), 'utf8');
  const section = version ? extractSection(changelog, version.replace(/^v/, '')) : null;
  if (!section) {
    console.error(`Kein Abschnitt für Version ${version} in CHANGELOG.md`);
    process.exit(1);
  }
  console.log(section);
}

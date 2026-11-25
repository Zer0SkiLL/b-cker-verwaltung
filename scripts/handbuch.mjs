// Gemeinsame Funktionen fürs Handbuch (docs/handbuch): Inhaltsverzeichnis lesen, Stimmigkeit prüfen.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const IMAGE = /!\[[^\]]*\]\((bilder\/[^)\s]+)\)/g;
const CHAPTER_LINK = /\]\(([0-9a-z-]+\.md)(#[0-9a-z-]+)?\)/g;

/** Inhaltsverzeichnis: [{ teil, kapitel: [{ datei, titel }] }]. */
export function readToc(dir) {
  return JSON.parse(readFileSync(join(dir, 'inhalt.json'), 'utf8'));
}

/** Prüft Bilder, Kapitel-Links und Inhaltsverzeichnis; liefert deutsche Fehlermeldungen (leer = in Ordnung). */
export function checkHandbook(dir) {
  const listed = readToc(dir).flatMap((t) => t.kapitel.map((k) => k.datei));
  const files = readdirSync(dir).filter((f) => f.endsWith('.md')).sort();
  const imageDir = join(dir, 'bilder');
  const images = existsSync(imageDir) ? readdirSync(imageDir).filter((f) => !f.startsWith('.')) : [];
  const errors = [];
  const used = new Set();
  for (const file of files) {
    const text = readFileSync(join(dir, file), 'utf8');
    for (const [, src] of text.matchAll(IMAGE)) {
      used.add(src);
      if (!existsSync(join(dir, src))) errors.push(`Bild fehlt: ${src} (in ${file})`);
    }
    for (const [, target] of text.matchAll(CHAPTER_LINK)) {
      if (!files.includes(target)) errors.push(`Link-Ziel fehlt: ${target} (in ${file})`);
    }
    if (!listed.includes(file)) errors.push(`Nicht im Inhaltsverzeichnis: ${file}`);
  }
  for (const image of images) {
    if (!used.has(`bilder/${image}`)) errors.push(`Bild wird nicht verwendet: bilder/${image}`);
  }
  for (const file of listed) {
    if (!files.includes(file)) errors.push(`Datei fehlt: ${file}`);
  }
  return errors;
}

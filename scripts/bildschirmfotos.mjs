// Holt die Bildschirmfotos des letzten erfolgreichen Laufs von „Bildschirmfotos“ (GitHub Actions)
// nach docs/handbuch/bilder. Aufruf: node scripts/bildschirmfotos.mjs [--run <id>]
import { execFileSync } from 'node:child_process';
import { readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const TARGET = join(ROOT, 'docs', 'handbuch', 'bilder');

const gh = (args) => execFileSync('gh', args, { cwd: ROOT, encoding: 'utf8' }).trim();

function latestRun() {
  const out = gh(['run', 'list', '--workflow', 'bildschirmfotos.yml', '--status', 'success', '--limit', '1', '--json', 'databaseId', '--jq', '.[0].databaseId']);
  if (!out) throw new Error('Kein erfolgreicher Lauf von „Bildschirmfotos“ gefunden. Zuerst den Workflow starten.');
  return out;
}

try {
  const at = process.argv.indexOf('--run');
  const run = at >= 0 ? process.argv[at + 1] : latestRun();
  gh(['run', 'download', run, '-n', 'bildschirmfotos', '-D', TARGET]);
  const count = readdirSync(TARGET).filter((f) => f.endsWith('.png')).length;
  console.log(`${count} Bilder aus Lauf ${run} in docs/handbuch/bilder.`);
  console.log('Jetzt prüfen und committen: docs: Bildschirmfotos aktualisieren');
} catch (err) {
  console.error(`Bildschirmfotos holen fehlgeschlagen: ${err.message}`);
  process.exit(1);
}

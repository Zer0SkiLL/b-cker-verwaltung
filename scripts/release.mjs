// Neue Version veröffentlichen: node scripts/release.mjs 0.2.0 [--dry-run]
// Prüft zuerst alles, ändert erst dann Dateien, committet, taggt und pusht.
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { insertSection } from './notes.mjs';
import { isGreater, parseSemver, readCargoVersion, setCargoVersion, setJsonVersion } from './version.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const FILES = {
  cargo: 'crates/app/Cargo.toml',
  tauri: 'crates/app/tauri.conf.json',
  ui: 'ui/package.json',
  root: 'package.json',
  changelog: 'CHANGELOG.md',
};

/** Prüft die Voraussetzungen; liefert deutsche Fehlermeldungen (leer = alles in Ordnung). */
export function preflight({ branch, clean, current, next }) {
  const errors = [];
  if (branch !== 'main') errors.push(`Releases nur von „main“ (aktuell: ${branch}).`);
  if (!clean) errors.push('Es gibt nicht committete Änderungen.');
  try {
    parseSemver(next);
    if (!isGreater(next, current)) {
      errors.push(`Die neue Version ${next} muss grösser sein als die aktuelle ${current}.`);
    }
  } catch (err) {
    errors.push(err.message);
  }
  return errors;
}

const run = (cmd, args, opts = {}) =>
  // shell unter Windows: pnpm/git-cliff sind dort .cmd-Dateien.
  execFileSync(cmd, args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: opts.quiet ? 'pipe' : 'inherit',
    shell: process.platform === 'win32',
    ...opts,
  });
const read = (file) => readFileSync(`${ROOT}/${file}`, 'utf8');
const write = (file, text) => writeFileSync(`${ROOT}/${file}`, text);

function runChecks() {
  run('cargo', ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings']);
  run('cargo', ['test', '--workspace']);
  run('pnpm', ['--dir', 'ui', 'exec', 'vitest', 'run']);
  run('pnpm', ['--dir', 'ui', 'check']);
  run('node', ['--test', 'scripts/*.test.mjs']);
}

function bumpFiles(version) {
  write(FILES.cargo, setCargoVersion(read(FILES.cargo), version));
  for (const file of [FILES.tauri, FILES.ui, FILES.root]) write(file, setJsonVersion(read(file), version));
  const section = run('pnpm', ['exec', 'git-cliff', '--unreleased', '--tag', `v${version}`], { quiet: true });
  write(FILES.changelog, insertSection(read(FILES.changelog), section));
  run('cargo', ['update', '-p', 'verwaltung_app', '--offline'], { quiet: true });
}

function main([version, flag]) {
  const state = {
    branch: run('git', ['rev-parse', '--abbrev-ref', 'HEAD'], { quiet: true }).trim(),
    clean: run('git', ['status', '--porcelain'], { quiet: true }).trim() === '',
    current: readCargoVersion(read(FILES.cargo)),
    next: version ?? '',
  };
  const errors = preflight(state);
  if (errors.length > 0) {
    console.error(`Release abgebrochen:\n- ${errors.join('\n- ')}`);
    process.exit(1);
  }
  if (flag === '--dry-run') {
    console.log(`Würde ${state.current} → ${version} veröffentlichen (Tests, Version, CHANGELOG, Tag v${version}, Push).`);
    return;
  }
  runChecks();
  bumpFiles(version);
  run('git', ['add', ...Object.values(FILES), 'Cargo.lock']);
  run('git', ['commit', '-m', `chore: Version ${version}`]);
  run('git', ['tag', '-a', `v${version}`, '-m', `Version ${version}`]);
  run('git', ['push', '--follow-tags', 'origin', 'main']);
  console.log(`Version ${version} ist getaggt. GitHub baut jetzt die Installer; danach den Entwurf prüfen und veröffentlichen.`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));

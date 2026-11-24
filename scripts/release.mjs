// Neue Version veröffentlichen: node scripts/release.mjs 0.2.0 [--dry-run]
// Reihenfolge: alles prüfen → alle neuen Inhalte im Speicher berechnen → schreiben → Commit → Tag
// → atomarer Push (main und Tag zusammen oder gar nicht). Bei Fehlern nennt das Skript den Ausweg.
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { insertSection } from './notes.mjs';
import { isGreater, parseSemver, readCargoVersion, setCargoVersion, setJsonVersion } from './version.mjs';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const CARGO = 'crates/app/Cargo.toml';
const JSON_FILES = ['crates/app/tauri.conf.json', 'ui/package.json', 'package.json'];
const CHANGELOG = 'CHANGELOG.md';
const FILES = [CARGO, ...JSON_FILES, CHANGELOG];

/** Prüft die Voraussetzungen; liefert deutsche Fehlermeldungen (leer = alles in Ordnung). */
export function preflight({ branch, clean, current, next, upToDate = true, tagExists = false }) {
  const errors = [];
  if (branch !== 'main') errors.push(`Releases nur von „main“ (aktuell: ${branch}).`);
  if (!clean) errors.push('Es gibt nicht committete Änderungen.');
  if (!upToDate) errors.push('Lokales „main“ entspricht nicht „origin/main“ (zuerst pull bzw. push).');
  try {
    parseSemver(next);
    if (!isGreater(next, current)) {
      errors.push(`Die neue Version ${next} muss grösser sein als die aktuelle ${current}.`);
    }
    if (tagExists) errors.push(`Der Tag v${next} existiert bereits.`);
  } catch (err) {
    errors.push(err.message);
  }
  return errors;
}

/** Neue Inhalte aller Versionsdateien; die Eingabe bleibt unverändert. */
export function bumpedContents(files, version, section) {
  const next = { ...files };
  next[CARGO] = setCargoVersion(files[CARGO], version);
  for (const file of JSON_FILES) next[file] = setJsonVersion(files[file], version);
  next[CHANGELOG] = insertSection(files[CHANGELOG], section);
  return next;
}

/** Was nach einem Fehler im jeweiligen Schritt zu tun ist. */
export function recoveryHint(step, version) {
  const hints = {
    files: 'Änderungen verwerfen und neu starten: git checkout -- . && git status',
    commit: 'Änderungen verwerfen und neu starten: git checkout -- . && git status',
    tag: `Commit zurücknehmen: git tag -d v${version} ; git reset --hard HEAD~1`,
    push: `Nur der Push fehlt. Später erneut: git push --atomic origin main v${version}`,
  };
  return hints[step];
}

// Nur pnpm braucht unter Windows eine Shell (.cmd); git/cargo laufen ohne, damit Argumente mit
// Leerzeichen (Commit-Meldung) nicht zerfallen.
const run = (cmd, args, { quiet = false } = {}) =>
  execFileSync(cmd, args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: quiet ? 'pipe' : 'inherit',
    shell: cmd === 'pnpm' && process.platform === 'win32',
  });
const read = (file) => readFileSync(`${ROOT}/${file}`, 'utf8');

function runChecks() {
  run('cargo', ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings']);
  run('cargo', ['test', '--workspace']);
  run('pnpm', ['--dir', 'ui', 'exec', 'vitest', 'run']);
  run('pnpm', ['--dir', 'ui', 'check']);
  run('node', ['--test', 'scripts/*.test.mjs']);
}

function currentState(version) {
  run('git', ['fetch', '--quiet', '--tags', 'origin', 'main'], { quiet: true });
  const head = run('git', ['rev-parse', 'HEAD'], { quiet: true }).trim();
  const remote = run('git', ['rev-parse', 'origin/main'], { quiet: true }).trim();
  const tags = run('git', ['tag', '--list', `v${version ?? ''}`], { quiet: true }).trim();
  return {
    branch: run('git', ['rev-parse', '--abbrev-ref', 'HEAD'], { quiet: true }).trim(),
    clean: run('git', ['status', '--porcelain'], { quiet: true }).trim() === '',
    current: readCargoVersion(read(CARGO)),
    next: version ?? '',
    upToDate: head === remote,
    tagExists: Boolean(version) && tags !== '',
  };
}

function step(name, version, action) {
  try {
    action();
  } catch (err) {
    console.error(`\nRelease abgebrochen (${name}): ${err.message}\n${recoveryHint(name, version)}`);
    process.exit(1);
  }
}

function main([version, flag]) {
  const state = currentState(version);
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
  // Alles berechnen, bevor etwas geschrieben wird.
  const section = run('pnpm', ['exec', 'git-cliff', '--unreleased', '--tag', `v${version}`], { quiet: true });
  const contents = bumpedContents(Object.fromEntries(FILES.map((f) => [f, read(f)])), version, section);
  step('files', version, () => {
    for (const file of FILES) writeFileSync(`${ROOT}/${file}`, contents[file]);
    run('cargo', ['update', '-p', 'verwaltung_app', '--offline'], { quiet: true });
  });
  step('commit', version, () => {
    run('git', ['add', ...FILES, 'Cargo.lock']);
    run('git', ['commit', '-m', `chore: Version ${version}`]);
  });
  step('tag', version, () => run('git', ['tag', '-a', `v${version}`, '-m', `Version ${version}`]));
  step('push', version, () => run('git', ['push', '--atomic', 'origin', 'main', `v${version}`]));
  console.log(`Version ${version} ist getaggt. GitHub baut jetzt die Installer; danach den Entwurf prüfen und veröffentlichen.`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main(process.argv.slice(2));

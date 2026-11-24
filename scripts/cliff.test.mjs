import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const cliff = join(root, 'node_modules', '.bin', 'git-cliff');

function repoWith(messages) {
  const dir = mkdtempSync(join(tmpdir(), 'cliff-'));
  const git = (...args) => execFileSync('git', args, { cwd: dir, stdio: 'pipe' });
  git('init', '-q');
  git('config', 'user.email', 'test@example.invalid');
  git('config', 'user.name', 'Test');
  git('config', 'commit.gpgsign', 'false');
  messages.forEach((m, i) => {
    writeFileSync(join(dir, `f${i}`), m);
    git('add', '.');
    git('commit', '-q', '-m', m);
  });
  return dir;
}

function notes(dir) {
  return execFileSync(cliff, ['--config', join(root, 'cliff.toml'), '--unreleased', '--tag', 'v9.9.9'], {
    cwd: dir,
    encoding: 'utf8',
  });
}

test('git-cliff gruppiert auf Deutsch und lässt Technisches weg', () => {
  const out = notes(repoWith(['feat: Gruppen umbenennen', 'fix: Mail öffnen geht wieder', 'chore: Abhängigkeiten']));
  assert.match(out, /## \[9\.9\.9\] – \d{2}\.\d{2}\.\d{4}/);
  assert.match(out, /### Neu\n+- Gruppen umbenennen/);
  assert.match(out, /### Behoben\n+- Mail öffnen geht wieder/);
  assert.doesNotMatch(out, /Abhängigkeiten/);
});

test('nur technische Commits ergeben einen Satz statt leerer Abschnitte', () => {
  const out = notes(repoWith(['chore: Aufräumen', 'ci: Workflow']));
  assert.match(out, /Interne Verbesserungen ohne sichtbare Änderungen\./);
  assert.doesNotMatch(out, /###/);
});

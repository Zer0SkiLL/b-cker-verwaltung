// E2E-Tests im echten Programm über tauri-driver (Linux: WebKitWebDriver, Windows: msedgedriver).
import { spawn } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const exe = process.platform === 'win32' ? '.exe' : '';
const application = resolve(import.meta.dirname, '..', 'target', 'debug', `mitgliederverwaltung${exe}`);
const tauriDriver = process.env.TAURI_DRIVER ?? join(homedir(), '.cargo', 'bin', `tauri-driver${exe}`);

// Eine frische Datei pro Lauf; das Programm öffnet sie über MITGLIEDERVERWALTUNG_DB (nur Debug-Builds).
process.env.MITGLIEDERVERWALTUNG_DB = join(mkdtempSync(join(tmpdir(), 'mv-e2e-')), 'verband.sqlite');

let driver;

/** Startet einen tauri-driver; ein Treiber erlaubt nur eine Sitzung (eine Programm-Instanz). */
export function startTauriDriver(port = 4444, nativePort = 4445) {
  const native = process.env.NATIVE_DRIVER ? ['--native-driver', process.env.NATIVE_DRIVER] : [];
  const args = ['--port', String(port), '--native-port', String(nativePort), ...native];
  return spawn(tauriDriver, args, { stdio: [null, process.stdout, process.stderr] });
}

export const config = {
  runner: 'local',
  hostname: '127.0.0.1',
  port: 4444,
  specs: ['./specs/**/*.e2e.mjs'],
  maxInstances: 1,
  capabilities: [{ maxInstances: 1, 'tauri:options': { application } }],
  logLevel: 'warn',
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { ui: 'bdd', timeout: 60000 },
  waitforTimeout: 15000,
  beforeSession: () => {
    driver = startTauriDriver();
  },
  afterSession: () => driver?.kill(),
};

// Updates über GitHub Releases: prüfen, Datei sauber schliessen, installieren, neu starten.
import { writable } from 'svelte/store';
import * as api from './api';
import { logToFile, type LogLevel } from './logFile';
import { sessionInfo } from './session';
import type { OpenResult, SecondaryResultDto } from './types';

export type UpdateState =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'none' }
  | { kind: 'available'; version: string; notes: string }
  | { kind: 'installing'; percent: number | null; note: string | null }
  | { kind: 'failed'; message: string; fileClosed: boolean };

type DownloadEvent =
  | { event: 'Started'; data: { contentLength?: number | null } }
  | { event: 'Progress'; data: { chunkLength: number } }
  | { event: 'Finished' };

export interface AvailableUpdate {
  version: string;
  body?: string | null;
  downloadAndInstall: (onEvent: (event: DownloadEvent) => void) => Promise<void>;
}

export interface UpdateDeps {
  check: () => Promise<AvailableUpdate | null>;
  closeFile: () => Promise<SecondaryResultDto | null>;
  relaunch: () => Promise<void>;
  lastPath: () => Promise<string | null>;
  openFile: (path: string) => Promise<OpenResult>;
  log?: (level: LogLevel, message: string) => void;
}

export const updateState = writable<UpdateState>({ kind: 'idle' });
let pending: AvailableUpdate | null = null;

const defaultDeps: UpdateDeps = {
  check: async () => (await import('@tauri-apps/plugin-updater')).check() as Promise<AvailableUpdate | null>,
  closeFile: () => api.closeFile(),
  relaunch: async () => (await import('@tauri-apps/plugin-process')).relaunch(),
  lastPath: () => api.lastPath(),
  openFile: (path) => api.openFile(path),
};

export async function checkForUpdates({ silent }: { silent: boolean }, deps = defaultDeps): Promise<void> {
  const log = deps.log ?? logToFile;
  if (!silent) updateState.set({ kind: 'checking' });
  try {
    pending = await deps.check();
  } catch {
    pending = null;
    log('warn', 'Update-Prüfung fehlgeschlagen');
    updateState.set(
      silent ? { kind: 'idle' } : { kind: 'failed', message: 'Update-Prüfung nicht möglich (keine Verbindung?).', fileClosed: false },
    );
    return;
  }
  log('info', pending ? `Update-Prüfung: Version ${pending.version} verfügbar` : 'Update-Prüfung: keine neue Version');
  if (pending) {
    updateState.set({ kind: 'available', version: pending.version, notes: pending.body ?? '' });
  } else {
    updateState.set(silent ? { kind: 'idle' } : { kind: 'none' });
  }
}

function closeNote(result: SecondaryResultDto | null): string | null {
  return result?.kind === 'unavailable' ? `Hinweis: ${result.reason}` : null;
}

let inProgress = false;

/** Schliesst die Datei sauber, lädt das Update, installiert es und startet neu. */
export async function installUpdate(deps = defaultDeps): Promise<void> {
  const update = pending;
  if (!update || inProgress) return;
  inProgress = true;
  updateState.set({ kind: 'installing', percent: null, note: 'Datei wird geschlossen …' });
  // Auch wenn das Schliessen scheitert, ist die Datei danach nicht mehr offen (Rust gibt sie frei).
  let fileClosed = true;
  try {
    const note = closeNote(await deps.closeFile().finally(() => sessionInfo.set(null)));
    let total: number | null = null;
    let received = 0;
    updateState.set({ kind: 'installing', percent: null, note });
    // Der Installer beendet das Programm ohne normales Ende; so gilt das nicht als Absturz.
    (deps.log ?? logToFile)('info', 'Programmende (Update)');
    await update.downloadAndInstall((e) => {
      if (e.event === 'Started') total = e.data.contentLength ?? null;
      if (e.event === 'Progress') received += e.data.chunkLength;
      const percent = total ? Math.min(100, Math.round((received / total) * 100)) : null;
      updateState.set({ kind: 'installing', percent, note });
    });
    await deps.relaunch();
  } catch (err) {
    updateState.set({ kind: 'failed', message: `Update fehlgeschlagen: ${(err as Error).message}`, fileClosed });
  } finally {
    inProgress = false;
  }
}

/**
 * Nach einem gescheiterten Update: zuletzt geöffnete Datei wieder öffnen.
 * Liefert die Seite, auf die es weitergeht; ohne Pfad oder bei Fehler die Startseite (Datei wählen).
 */
export async function reopenLastFile(deps = defaultDeps): Promise<'addresses' | 'start'> {
  updateState.set({ kind: 'idle' });
  try {
    const path = await deps.lastPath();
    if (!path) return 'start';
    const result = await deps.openFile(path);
    sessionInfo.set(result.info);
    return 'addresses';
  } catch {
    return 'start';
  }
}

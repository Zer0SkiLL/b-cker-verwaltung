import { get } from 'svelte/store';
import { vi } from 'vitest';
import { checkForUpdates, installUpdate, reopenLastFile, updateState, type UpdateDeps } from './updates';

function deps(overrides: Partial<UpdateDeps> = {}): UpdateDeps {
  const calls: string[] = [];
  const update = {
    version: '0.3.0',
    body: '### Neu\n- Mailvorlagen',
    downloadAndInstall: vi.fn(async (onEvent: (e: unknown) => void) => {
      calls.push('download');
      onEvent({ event: 'Started', data: { contentLength: 100 } });
      onEvent({ event: 'Progress', data: { chunkLength: 50 } });
      onEvent({ event: 'Finished' });
    }),
  };
  return {
    check: vi.fn(async () => update),
    closeFile: vi.fn(async () => {
      calls.push('close');
      return { kind: 'notConfigured' as const };
    }),
    relaunch: vi.fn(async () => {
      calls.push('relaunch');
    }),
    lastPath: vi.fn(async () => '/daten/verband.sqlite'),
    openFile: vi.fn(async () => ({
      info: { path: '/daten/verband.sqlite', readOnly: false, heldBy: null, backup: { secondaryDir: null, lastSecondary: null, reminderDue: true } },
      warning: null,
    })),
    calls,
    ...overrides,
  } as UpdateDeps & { calls: string[] };
}

beforeEach(() => updateState.set({ kind: 'idle' }));

test('stille Prüfung offline bleibt still', async () => {
  await checkForUpdates({ silent: true }, deps({ check: vi.fn(async () => { throw new Error('offline'); }) }));
  expect(get(updateState)).toEqual({ kind: 'idle' });
});

test('manuelle Prüfung offline meldet es verständlich', async () => {
  await checkForUpdates({ silent: false }, deps({ check: vi.fn(async () => { throw new Error('offline'); }) }));
  expect(get(updateState)).toEqual({ kind: 'failed', message: 'Update-Prüfung nicht möglich (keine Verbindung?).', fileClosed: false });
});

test('kein Update: still → idle, manuell → none', async () => {
  await checkForUpdates({ silent: true }, deps({ check: vi.fn(async () => null) }));
  expect(get(updateState)).toEqual({ kind: 'idle' });
  await checkForUpdates({ silent: false }, deps({ check: vi.fn(async () => null) }));
  expect(get(updateState)).toEqual({ kind: 'none' });
});

test('Update verfügbar mit Notes', async () => {
  await checkForUpdates({ silent: true }, deps());
  expect(get(updateState)).toEqual({ kind: 'available', version: '0.3.0', notes: '### Neu\n- Mailvorlagen' });
});

test('Installieren: erst Datei schliessen, dann laden, dann Neustart', async () => {
  const d = deps() as UpdateDeps & { calls: string[] };
  await checkForUpdates({ silent: true }, d);
  await installUpdate(d);
  expect(d.calls).toEqual(['close', 'download', 'relaunch']);
});

test('nicht erreichbarer zweiter Ort ist nur ein Hinweis', async () => {
  const d = deps({ closeFile: vi.fn(async () => ({ kind: 'unavailable' as const, reason: 'USB-Stick fehlt' })) });
  await checkForUpdates({ silent: true }, d);
  await installUpdate(d);
  expect(d.relaunch).toHaveBeenCalled();
});

test('Download-Fehler: Datei geschlossen, Ausweg bleibt', async () => {
  const failing = {
    version: '0.3.0',
    body: null,
    downloadAndInstall: vi.fn(async () => { throw new Error('Verbindung abgebrochen'); }),
  };
  const d = deps({ check: vi.fn(async () => failing) });
  await checkForUpdates({ silent: true }, d);
  await installUpdate(d);
  expect(get(updateState)).toEqual({
    kind: 'failed',
    message: 'Update fehlgeschlagen: Verbindung abgebrochen',
    fileClosed: true,
  });
  await reopenLastFile(d);
  expect(d.openFile).toHaveBeenCalledWith('/daten/verband.sqlite');
  expect(get(updateState)).toEqual({ kind: 'idle' });
});

test('Schliessen scheitert: Fehler sichtbar, Datei gilt als geschlossen', async () => {
  const d = deps({ closeFile: vi.fn(async () => { throw new Error('Sperre verloren'); }) });
  await checkForUpdates({ silent: true }, d);
  await installUpdate(d);
  expect(get(updateState)).toEqual({ kind: 'failed', message: 'Update fehlgeschlagen: Sperre verloren', fileClosed: true });
});

test('Doppelklick startet nur eine Installation', async () => {
  const d = deps() as UpdateDeps & { calls: string[] };
  await checkForUpdates({ silent: true }, d);
  await Promise.all([installUpdate(d), installUpdate(d)]);
  expect(d.calls.filter((c) => c === 'close')).toHaveLength(1);
  expect(d.calls.filter((c) => c === 'download')).toHaveLength(1);
});

test('Installation zeigt sofort, dass die Datei geschlossen wird', async () => {
  let seen: unknown = null;
  const d = deps({ closeFile: vi.fn(async () => { seen = get(updateState); return null; }) });
  await checkForUpdates({ silent: true }, d);
  await installUpdate(d);
  expect(seen).toEqual({ kind: 'installing', percent: null, note: 'Datei wird geschlossen …' });
});

test('Datei wieder öffnen: ohne gemerkten Pfad zur Startseite', async () => {
  expect(await reopenLastFile(deps({ lastPath: vi.fn(async () => null) }))).toBe('start');
});

test('Datei wieder öffnen: Fehler führt zur Startseite statt ins Leere', async () => {
  const d = deps({ openFile: vi.fn(async () => { throw new Error('Netzlaufwerk fehlt'); }) });
  expect(await reopenLastFile(d)).toBe('start');
  expect(get(updateState)).toEqual({ kind: 'idle' });
});

test('Datei wieder öffnen: Erfolg führt zu den Adressen', async () => {
  expect(await reopenLastFile(deps())).toBe('addresses');
});

test('Update-Prüfung landet im Protokoll', async () => {
  const log = vi.fn();
  await checkForUpdates({ silent: true }, deps({ log }));
  expect(log).toHaveBeenCalledWith('info', 'Update-Prüfung: Version 0.3.0 verfügbar');
  await checkForUpdates({ silent: true }, deps({ log, check: vi.fn(async () => null) }));
  expect(log).toHaveBeenCalledWith('info', 'Update-Prüfung: keine neue Version');
  await checkForUpdates({ silent: true }, deps({ log, check: vi.fn(async () => { throw new Error('offline'); }) }));
  expect(log).toHaveBeenCalledWith('warn', 'Update-Prüfung fehlgeschlagen');
});

// Sicherung während der Arbeit: ruft nach dem eingestellten Intervall „sichern, falls geändert“ auf.
// Das Intervall wird vor jeder Runde neu gelesen, damit geänderte Einstellungen sofort gelten.

export type AutoBackupDeps = {
  getInterval: () => Promise<number>;
  backup: () => Promise<unknown>;
  refresh: () => Promise<void>;
};

/** Ist die Sicherung ausgeschaltet (0) oder unbekannt, wird nach dieser Zeit erneut geprüft. */
const RECHECK_MINUTES = 5;
const MINUTE = 60_000;

export function startAutoBackup(deps: AutoBackupDeps): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;

  async function schedule(): Promise<void> {
    if (stopped) return;
    let minutes = 0;
    try {
      minutes = await deps.getInterval();
    } catch {
      minutes = 0;
    }
    if (stopped) return;
    if (minutes > 0) {
      timer = setTimeout(() => void run(), minutes * MINUTE);
    } else {
      timer = setTimeout(() => void schedule(), RECHECK_MINUTES * MINUTE);
    }
  }

  async function run(): Promise<void> {
    try {
      await deps.backup();
      await deps.refresh();
    } catch {
      // z.B. keine Datei offen oder nur lesend: nächste Runde versuchen.
    }
    await schedule();
  }

  void schedule();
  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
  };
}

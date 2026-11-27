// Zeile ins Programm-Protokoll schreiben (für „Problem melden“ und die Frühwarnung).
// Bewusst direkt per invoke und nicht über api.call: Sie soll nicht in der Aktionsliste stehen.
import { invoke } from '@tauri-apps/api/core';

export type LogLevel = 'info' | 'warn' | 'error';

export function logToFile(level: LogLevel, message: string): void {
  try {
    invoke('diagnostics_log', { level, message }).catch(() => {
      // Protokoll nicht erreichbar: nicht weiter schlimm.
    });
  } catch {
    // ausserhalb des Programms (Tests ohne Tauri)
  }
}

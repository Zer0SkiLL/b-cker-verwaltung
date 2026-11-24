// Dünne Hüllen um Tauri-Plugins (Dateidialoge, Rückfragen, Zwischenablage, Mailprogramm).
import { ask, message, open, save } from '@tauri-apps/plugin-dialog';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { openUrl } from '@tauri-apps/plugin-opener';

const DB_FILTER = [{ name: 'Mitgliederverwaltung', extensions: ['sqlite'] }];

export async function pickDatabase(): Promise<string | null> {
  const picked = await open({ multiple: false, directory: false, filters: DB_FILTER });
  return typeof picked === 'string' ? picked : null;
}

export async function pickNewDatabase(): Promise<string | null> {
  const picked = await save({ defaultPath: 'Mitgliederverwaltung.sqlite', filters: DB_FILTER });
  if (!picked) return null;
  return picked.endsWith('.sqlite') ? picked : `${picked}.sqlite`;
}

export async function pickFolder(): Promise<string | null> {
  const picked = await open({ multiple: false, directory: true });
  return typeof picked === 'string' ? picked : null;
}

export async function pickFile(name: string, extensions: string[]): Promise<string | null> {
  const picked = await open({ multiple: false, directory: false, filters: [{ name, extensions }] });
  return typeof picked === 'string' ? picked : null;
}

export async function pickSavePath(defaultName: string, extension: string): Promise<string | null> {
  const picked = await save({
    defaultPath: defaultName,
    filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
  });
  if (!picked) return null;
  return picked.endsWith(`.${extension}`) ? picked : `${picked}.${extension}`;
}

export function confirmAction(text: string, okLabel = 'Ja'): Promise<boolean> {
  return ask(text, { title: 'Mitgliederverwaltung', kind: 'warning', okLabel, cancelLabel: 'Abbrechen' });
}

export async function showInfo(text: string): Promise<void> {
  await message(text, { title: 'Mitgliederverwaltung', kind: 'info' });
}

export function copyText(text: string): Promise<void> {
  return writeText(text);
}

export function openMail(link: string): Promise<void> {
  return openUrl(link);
}

/** Öffnet eine Webseite im Standard-Browser (nur in den Rechten freigegebene Adressen). */
export function openWebsite(url: string): Promise<void> {
  return openUrl(url);
}

/** Heutiges Datum als JJJJ-MM-TT für Dateinamen. */
export function todayStamp(): string {
  const t = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${t.getFullYear()}-${pad(t.getMonth() + 1)}-${pad(t.getDate())}`;
}

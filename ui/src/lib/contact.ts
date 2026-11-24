// Welche Kontakt-Aktionen für eine Ergebnisliste möglich sind.
import type { SearchResult } from './types';

export const NO_MAIL_HINT = 'Keine Mail-Adressen in dieser Liste';
export const TOO_MANY_HINT = 'Zu viele Adressen für das Mailprogramm – bitte kopieren';

export interface ContactActions {
  canCopy: boolean;
  canMailto: boolean;
  hint: string | null;
}

export function contactActions(result: SearchResult): ContactActions {
  if (result.rows.length === 0) return { canCopy: false, canMailto: false, hint: null };
  if (result.emailCount === 0) return { canCopy: false, canMailto: false, hint: NO_MAIL_HINT };
  if (result.mailto === null) return { canCopy: true, canMailto: false, hint: TOO_MANY_HINT };
  return { canCopy: true, canMailto: true, hint: null };
}

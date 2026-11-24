import { contactActions, NO_MAIL_HINT, TOO_MANY_HINT } from './contact';
import type { SearchResult } from './types';

const row = {
  kind: 'Organisation' as const,
  id: 1,
  display_name: 'Beck AG',
  organisation: null,
  street: null,
  zip: null,
  city: null,
  po_box: null,
  email: 'info@beck.ch',
  phone: null,
  mobile: null,
};

const result = (partial: Partial<SearchResult>): SearchResult => ({
  rows: [],
  emailCount: 0,
  missingEmail: [],
  joinedEmails: '',
  mailto: null,
  ...partial,
});

test('leeres Ergebnis: nichts möglich', () => {
  expect(contactActions(result({}))).toEqual({ canCopy: false, canMailto: false, hint: null });
});

test('Treffer ohne Mail: Hinweis', () => {
  const r = result({ rows: [{ ...row, email: null }], missingEmail: [{ ...row, email: null }] });
  expect(contactActions(r)).toEqual({ canCopy: false, canMailto: false, hint: NO_MAIL_HINT });
});

test('zu viele Adressen: nur kopieren', () => {
  const r = result({ rows: [row], emailCount: 300, joinedEmails: 'a; b', mailto: null });
  expect(contactActions(r)).toEqual({ canCopy: true, canMailto: false, hint: TOO_MANY_HINT });
  expect(TOO_MANY_HINT).toBe('Zu viele Adressen für das Mailprogramm – bitte kopieren');
});

test('Normalfall: beides möglich', () => {
  const r = result({ rows: [row], emailCount: 1, joinedEmails: 'info@beck.ch', mailto: 'mailto:?bcc=info%40beck.ch' });
  expect(contactActions(r)).toEqual({ canCopy: true, canMailto: true, hint: null });
});

import { get } from 'svelte/store';
import { back, navigate, resetRouter, route } from './router';
import { emptyFilter } from './filters';

beforeEach(() => resetRouter());

test('Start ist die erste Seite', () => {
  expect(get(route)).toEqual({ page: 'start' });
});

test('zurück erhält den Adressen-Filter', async () => {
  const filter = { ...emptyFilter(), text: 'beck' };
  await navigate({ page: 'addresses', filter });
  await navigate({ page: 'organisation', id: 3 });
  await back();
  expect(get(route)).toEqual({ page: 'addresses', filter });
});

test('zurück ohne Verlauf bleibt stehen', async () => {
  await navigate({ page: 'teams' });
  await back();
  await back();
  expect(get(route)).toEqual({ page: 'start' });
});

import { setLeaveGuard } from './router';

test('Schutz: abgelehnt bleibt die Seite, bestätigt wird navigiert', async () => {
  await navigate({ page: 'organisation', id: 3 });
  setLeaveGuard(async () => false);
  await navigate({ page: 'teams' });
  expect(get(route)).toEqual({ page: 'organisation', id: 3 });
  await back();
  expect(get(route)).toEqual({ page: 'organisation', id: 3 });
  setLeaveGuard(async () => true);
  await navigate({ page: 'teams' });
  expect(get(route)).toEqual({ page: 'teams' });
  setLeaveGuard(null);
});

test('resetRouter entfernt den Schutz', async () => {
  setLeaveGuard(async () => false);
  resetRouter();
  await navigate({ page: 'groups' });
  expect(get(route)).toEqual({ page: 'groups' });
});

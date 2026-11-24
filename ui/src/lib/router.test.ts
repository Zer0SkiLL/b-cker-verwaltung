import { get } from 'svelte/store';
import { back, navigate, resetRouter, route } from './router';
import { emptyFilter } from './filters';

beforeEach(() => resetRouter());

test('Start ist die erste Seite', () => {
  expect(get(route)).toEqual({ page: 'start' });
});

test('zurück erhält den Adressen-Filter', () => {
  const filter = { ...emptyFilter(), text: 'beck' };
  navigate({ page: 'addresses', filter });
  navigate({ page: 'organisation', id: 3 });
  back();
  expect(get(route)).toEqual({ page: 'addresses', filter });
});

test('zurück ohne Verlauf bleibt stehen', () => {
  navigate({ page: 'teams' });
  back();
  back();
  expect(get(route)).toEqual({ page: 'start' });
});

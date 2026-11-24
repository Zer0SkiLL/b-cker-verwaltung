import {
  activeFilterCount,
  emptyFilter,
  setKinds,
  setMembership,
  toggleId,
  withText,
} from './filters';

test('toggleId zweimal ergibt die Ausgangsliste, Original bleibt unverändert', () => {
  const original = Object.freeze({ ...emptyFilter(), label_ids: Object.freeze([1]) as number[] });
  const once = toggleId(original, 'label_ids', 2);
  expect(once.label_ids).toEqual([1, 2]);
  expect(toggleId(once, 'label_ids', 2).label_ids).toEqual([1]);
  expect(original.label_ids).toEqual([1]);
});

test('withText: leerer Text wird null', () => {
  expect(withText(emptyFilter(), '  ').text).toBeNull();
  expect(withText(emptyFilter(), ' beck ').text).toBe('beck');
});

test('setKinds und setMembership liefern neue Objekte', () => {
  const base = Object.freeze(emptyFilter());
  expect(setKinds(base, ['Person']).kinds).toEqual(['Person']);
  expect(setMembership(base, 'Active').membership).toBe('Active');
  expect(base.membership).toBe('Any');
});

test('activeFilterCount zählt Text, Arten, IDs und Status', () => {
  expect(activeFilterCount(emptyFilter())).toBe(0);
  const f = setMembership(
    toggleId(toggleId(setKinds(withText(emptyFilter(), 'x'), ['Person']), 'team_ids', 1), 'label_ids', 3),
    'Exited',
  );
  expect(activeFilterCount(f)).toBe(5);
});

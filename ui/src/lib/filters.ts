// Filterzustand der Adressliste. Alle Funktionen liefern neue Objekte.
import type { EntryKind, Filter, FilterIdKey, MembershipStatus } from './types';

export function emptyFilter(): Filter {
  return {
    text: null,
    kinds: [],
    label_ids: [],
    team_ids: [],
    function_ids: [],
    org_type_ids: [],
    category_ids: [],
    cantonal_ids: [],
    membership: 'Any',
  };
}

const ID_KEYS: FilterIdKey[] = [
  'label_ids',
  'team_ids',
  'function_ids',
  'org_type_ids',
  'category_ids',
  'cantonal_ids',
];

export function withText(filter: Filter, text: string): Filter {
  const trimmed = text.trim();
  return { ...filter, text: trimmed === '' ? null : trimmed };
}

export function toggleId(filter: Filter, key: FilterIdKey, id: number): Filter {
  const current = filter[key];
  const next = current.includes(id) ? current.filter((x) => x !== id) : [...current, id];
  return { ...filter, [key]: next };
}

export function setKinds(filter: Filter, kinds: EntryKind[]): Filter {
  return { ...filter, kinds: [...kinds] };
}

export function setMembership(filter: Filter, membership: MembershipStatus): Filter {
  return { ...filter, membership };
}

export function activeFilterCount(filter: Filter): number {
  const ids = ID_KEYS.reduce((sum, key) => sum + filter[key].length, 0);
  return (filter.text ? 1 : 0) + (filter.kinds.length > 0 ? 1 : 0) + ids + (filter.membership === 'Any' ? 0 : 1);
}

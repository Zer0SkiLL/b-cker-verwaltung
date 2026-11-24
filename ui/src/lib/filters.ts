// Filterzustand der Adressliste. Alle Funktionen liefern neue Objekte.
import type { Filter } from './types';

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

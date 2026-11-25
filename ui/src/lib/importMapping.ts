// Deutsche Namen der Import-Felder und unveränderliche Bearbeitung der Zuordnung.

export const FIELD_LABELS: Record<string, string> = {
  company: 'Firma (Betrieb)',
  last_name: 'Nachname',
  first_name: 'Vorname',
  addition: 'Zusatz',
  legacy_id: 'Alt-ID (OBC-ID)',
  cantonal_association: 'Kantonalverband',
  street: 'Adresse',
  zip: 'PLZ',
  city: 'Ort',
  category: 'Mitgliederkategorie',
  email: 'Mail',
  website: 'Website (Home)',
  phone: 'Telefon',
  mobile: 'Mobiltelefon',
  apprenticeship: 'Lehrbetrieb (Label)',
  fr_baeck: 'Fr Bäck (Label)',
  fr_conf: 'Fr Conf (Label)',
  dh: 'DH (Label)',
  notes: 'Bemerkungen (Besonderes)',
  joined: 'Eintritt',
  left: 'Austritt',
  branches: 'Filialen',
  apprentice_count: 'Anzahl Lernende',
  cost_center_no: 'Kostenstellen-Nr',
  commercial_register_no: 'Handelsregister-Nr',
  mandate: 'Mandat (Labels)',
  ignore: '– nicht übernehmen –',
};

export function fieldLabel(key: string): string {
  return FIELD_LABELS[key] ?? key;
}

export function setColumn(mapping: string[], index: number, key: string): string[] {
  return mapping.map((current, i) => (i === index ? key : current));
}

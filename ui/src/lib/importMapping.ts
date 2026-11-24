// Deutsche Namen der Import-Felder und unveränderliche Bearbeitung der Zuordnung.

export const FIELD_LABELS: Record<string, string> = {
  legacy_id: 'Alt-ID (ID1)',
  apprentice_count: 'Anzahl Lernende',
  cantonal_association: 'Kantonalverband',
  name: 'Name/Firma',
  first_name: 'Vorname',
  addition: 'Zusatz',
  street: 'Adresse',
  zip: 'PLZ',
  city: 'Ort',
  category: 'Mitgliederkategorie',
  email: 'Mail',
  website: 'Website (Home)',
  apprenticeship: 'Lehrbetrieb (Label)',
  fr_baeck: 'Fr Bäck (Label)',
  fr_conf: 'Fr Conf (Label)',
  dh: 'DH (Label)',
  notes: 'Bemerkungen (Besonderes)',
  joined: 'Eintritt',
  left: 'Austritt',
  branches: 'Filialen',
  phone: 'Telefon',
  mobile: 'Mobiltelefon',
  po_box: 'Postfach',
  ignore: '– nicht übernehmen –',
};

export function fieldLabel(key: string): string {
  return FIELD_LABELS[key] ?? key;
}

export function setColumn(mapping: string[], index: number, key: string): string[] {
  return mapping.map((current, i) => (i === index ? key : current));
}

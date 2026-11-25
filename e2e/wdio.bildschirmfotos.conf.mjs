// Bildschirmfotos fürs Handbuch: nur diese Spezifikation, mit der Beispieldatei (BEISPIEL_DB).
import { config as base } from './wdio.conf.mjs';

const beispiel = process.env.BEISPIEL_DB;
if (!beispiel) {
  console.error('BEISPIEL_DB fehlt (Pfad zur Datei aus: cargo run -p verwaltung_core --example beispieldaten -- <pfad>).');
  process.exit(1);
}
// Erst nach dem Import setzen: die Basiskonfiguration legt sonst eine leere Testdatei fest.
// Das Programm erbt die Variable beim Start durch tauri-driver (beforeSession).
process.env.MITGLIEDERVERWALTUNG_DB = beispiel;

export const config = {
  ...base,
  specs: ['./specs/90-bildschirmfotos.e2e.mjs'],
  exclude: [],
};

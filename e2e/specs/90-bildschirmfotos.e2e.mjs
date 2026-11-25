// Bildschirmfotos fürs Handbuch (docs/handbuch/bilder), mit den Beispieldaten.
// Erzeugte Bilder: adressen, adressen-filter, adressen-gruppe-speichern, neuer-betrieb, betrieb,
// betrieb-offiziell, betrieb-provisorisch, betrieb-ausgetreten, person, gruppen, teams, einstellungen-labels,
// einstellungen-kategorien, einstellungen-sicherung, einstellungen-import, einstellungen-export,
// einstellungen-ueber, einstellungen-aufbewahrung, problem-melden, hilfe, hilfe-suche.
// Die Seiten werden nur angesehen, nicht gespeichert: die Beispieldaten bleiben unverändert.
import { join, resolve } from 'node:path';
import { mkdirSync } from 'node:fs';
import { clickButton, goTo } from './helpers.mjs';
import { entfernen, markieren, zeiger } from './markierungen.mjs';

const OUT = resolve(import.meta.dirname, '..', '..', 'docs', 'handbuch', 'bilder');
mkdirSync(OUT, { recursive: true });

async function shot(name) {
  await browser.pause(600); // Ladeanzeigen und Übergänge abwarten
  await browser.saveScreenshot(join(OUT, `${name}.png`));
  await entfernen();
}

// Zeile über die Spalte „Name“ finden (der Betriebsname steht auch bei dessen Personen).
const row = (name) => $(`//tr[td[2][normalize-space()="${name}"]]`);

async function openRow(text) {
  await goTo('Adressen');
  // Die Zelle anklicken (eine ganze Tabellenzeile gilt im WebDriver nicht als klickbar).
  const cell = await $(`//tr[td[2][normalize-space()="${text}"]]/td[2]`);
  await cell.waitForDisplayed();
  await cell.scrollIntoView();
  await cell.click();
  await (await $(`h1=${text}`)).waitForDisplayed();
}

/** Scrollt einen Abschnitt (Überschrift h2) an den oberen Rand, z.B. „Personen“. */
async function scrollToSection(title) {
  const heading = await $(`h2=${title}`);
  await heading.waitForDisplayed();
  await browser.execute((el) => el.scrollIntoView({ block: 'start' }), heading);
}

async function settingsTab(name) {
  await goTo('Einstellungen');
  await clickButton(name);
}

describe('Bildschirmfotos', () => {
  it('Adressen', async () => {
    await (await $('h1=Adressen')).waitForDisplayed();
    await (await row('Bäckerei Muster AG')).waitForDisplayed();
    await shot('adressen');
  });

  it('Adressen nach Label gefiltert', async () => {
    await (await $('summary*=Labels')).click();
    await (await $('label*=provisorisch')).click();
    await (await row('Bäckerei Neuanfang')).waitForDisplayed();
    await markieren('summary*=Labels', { nummer: 1 });
    await markieren('label*=provisorisch', { nummer: 2 });
    await markieren('button=Als Gruppe speichern', { nummer: 3 });
    await shot('adressen-filter');
    await clickButton('Als Gruppe speichern');
    await (await $('input[placeholder="Name der Gruppe"]')).setValue('Provisorische Mitglieder');
    await shot('adressen-gruppe-speichern');
    await clickButton('Abbrechen');
    await (await $('button=Filter zurücksetzen')).click();
  });

  it('Neuer Betrieb (Formular)', async () => {
    await goTo('Adressen');
    await clickButton('Neuer Betrieb');
    await (await $('#org-name')).waitForDisplayed();
    await shot('neuer-betrieb');
  });

  it('Betriebe', async () => {
    await openRow('Bäckerei Muster AG');
    await shot('betrieb');
    await scrollToSection('Personen');
    await shot('betrieb-offiziell');
    await openRow('Bäckerei Neuanfang');
    await scrollToSection('Personen');
    await shot('betrieb-provisorisch');
    await openRow('Bäckerei Abschied');
    await scrollToSection('Personen');
    await shot('betrieb-ausgetreten');
  });

  it('Person', async () => {
    await openRow('Anna Muster');
    await shot('person');
  });

  it('Gruppen und Teams', async () => {
    await goTo('Gruppen');
    await (await $('button=Lehrbetriebe')).waitForDisplayed();
    await shot('gruppen');
    await goTo('Teams');
    await clickButton('Prüfungsexperten (4)');
    await (await $('h2=Prüfungsexperten')).waitForDisplayed();
    await shot('teams');
  });

  it('Einstellungen', async () => {
    await settingsTab('Listen und Labels');
    await (await $('strong=Aufnahme')).waitForDisplayed();
    await shot('einstellungen-labels');
    await clickButton('Mitgliederkategorien');
    await shot('einstellungen-kategorien');
    await settingsTab('Datensicherung');
    await markieren('button=Jetzt sichern');
    await zeiger('button=Jetzt sichern');
    await shot('einstellungen-sicherung');
    await scrollToSection('Aufbewahrung');
    await (await $('#ret-intervalMinutes')).waitForDisplayed();
    await shot('einstellungen-aufbewahrung');
    await settingsTab('Excel-Import');
    await shot('einstellungen-import');
    await settingsTab('Gesamtexport');
    await shot('einstellungen-export');
    await settingsTab('Über das Programm');
    await shot('einstellungen-ueber');
  });

  it('Problem melden', async () => {
    await goTo('Adressen');
    await clickButton('Problem melden');
    const field = await $('#report-description');
    await field.waitForDisplayed();
    await field.setValue('Beispiel: Beim Speichern erschien eine Fehlermeldung');
    await markieren('#report-description', { nummer: 1 });
    await markieren('button=Bericht kopieren', { nummer: 2 });
    await shot('problem-melden');
    await clickButton('Schliessen');
  });

  it('Hilfe-Seitenleiste', async () => {
    await goTo('Adressen');
    await (await $('button[aria-label="Hilfe"]')).click();
    await (await $('section[aria-label="Hilfe"]')).waitForDisplayed();
    await shot('hilfe');
    await (await $('input[placeholder="Im Handbuch suchen"]')).setValue('Austritt');
    await shot('hilfe-suche');
    await browser.keys('Escape');
  });
});

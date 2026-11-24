import { remote } from 'webdriverio';
import { config, startTauriDriver } from '../wdio.conf.mjs';
import { button, byText, clickButton, goTo } from './helpers.mjs';

describe('Kernabläufe', () => {
  it('startet direkt in den Adressen (Testdatei ist offen)', async () => {
    await (await $('h1=Adressen')).waitForDisplayed();
  });

  it('legt eine Label-Gruppe mit Label an', async () => {
    await goTo('Einstellungen');
    await (await $('input[placeholder="Neue Gruppe"]')).setValue('Ausbildung');
    await (await $('select[aria-label="Gilt für"]')).selectByAttribute('value', 'Organisation');
    await clickButton('Gruppe anlegen');
    const label = await $('input[placeholder="Neues Label"]');
    await label.waitForDisplayed();
    await label.setValue('Lehrbetrieb');
    await clickButton('Hinzufügen');
    await (await byText('0× vergeben')).waitForDisplayed();
  });

  it('legt einen Betrieb an und findet ihn in den Adressen', async () => {
    await goTo('Adressen');
    await clickButton('Neuer Betrieb');
    await (await $('#org-name')).setValue('E2E Beck AG');
    await (await $('#org-city')).setValue('Bern');
    await clickButton('Speichern');
    await (await $('h1=E2E Beck AG')).waitForDisplayed();
  });

  it('vergibt ein Label und filtert danach', async () => {
    await (await $('select[aria-label="Label hinzufügen"]')).selectByVisibleText('Lehrbetrieb');
    await clickButton('Hinzufügen');
    await (await $('.chip*=Lehrbetrieb')).waitForDisplayed();
    await goTo('Adressen');
    await (await $('summary*=Labels')).click();
    await (await $('label*=Lehrbetrieb')).click();
    await (await byText('1 Einträge')).waitForDisplayed();
  });

  it('speichert den Filter als Gruppe', async () => {
    await clickButton('Als Gruppe speichern');
    await (await $('input[placeholder="Name der Gruppe"]')).setValue('Lehrbetriebe');
    await clickButton('Speichern');
    await goTo('Gruppen');
    await (await button('Lehrbetriebe')).waitForDisplayed();
  });

  it('zweite Instanz derselben Datei öffnet nur lesend', async () => {
    const secondDriver = startTauriDriver(4446, 4447);
    await new Promise((r) => setTimeout(r, 2000));
    const second = await remote({
      hostname: config.hostname,
      port: 4446,
      logLevel: 'warn',
      // Nur die WebDriver-Fähigkeiten, ohne runner-interne Optionen wie maxInstances.
      capabilities: { 'tauri:options': config.capabilities[0]['tauri:options'] },
    });
    try {
      const notice = await second.$('//*[contains(normalize-space(), "Nur lesen")]');
      await notice.waitForDisplayed({ timeout: 20000 });
    } finally {
      await second.deleteSession();
      secondDriver.kill();
    }
  });
});

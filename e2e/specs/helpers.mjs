// Kleine Helfer: Elemente über sichtbaren Text finden.
export const button = (text) => $(`//button[normalize-space()="${text}"]`);
export const byText = (text) => $(`//*[contains(normalize-space(), "${text}")]`);

export async function clickButton(text) {
  const el = await button(text);
  await el.waitForClickable();
  await el.click();
}

export async function goTo(menu) {
  await clickButton(menu);
}

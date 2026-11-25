// Markierungen für Handbuch-Bildschirmfotos: Rahmen, nummerierte Kreise und ein Mauszeiger.
// Sie werden direkt in die Seite gelegt und richten sich nach der echten Position des Elements,
// wandern also mit, wenn sich das Layout ändert. Nach dem Foto mit entfernen() wegnehmen.

const FARBE = '#e8590c'; // kräftiges Orange, kommt im Programm sonst nicht vor
const LAYER = 'mv-markierungen';

/** Element per WebdriverIO-Selektor holen und warten, bis es sichtbar ist. */
async function element(selector) {
  const el = await $(selector);
  await el.waitForDisplayed();
  return el;
}

/**
 * Markiert ein Element: `nummer` setzt einen Kreis oben links, `rahmen` (Standard) zieht einen Rahmen.
 * Beispiel: await markieren('button=Speichern', { nummer: 1 });
 */
export async function markieren(selector, { nummer = null, rahmen = true } = {}) {
  const el = await element(selector);
  await browser.execute(
    (target, opts) => {
      const layer =
        document.getElementById(opts.layer) ??
        Object.assign(document.body.appendChild(document.createElement('div')), { id: opts.layer });
      layer.style.cssText = 'position:fixed;inset:0;pointer-events:none;z-index:2147483647';
      const r = target.getBoundingClientRect();
      const pad = 4;
      if (opts.rahmen) {
        const box = layer.appendChild(document.createElement('div'));
        box.style.cssText = `position:fixed;left:${r.left - pad}px;top:${r.top - pad}px;width:${r.width + 2 * pad}px;height:${r.height + 2 * pad}px;border:3px solid ${opts.farbe};border-radius:7px;box-shadow:0 0 0 2px rgba(255,255,255,.9);box-sizing:border-box`;
      }
      if (opts.nummer !== null) {
        const size = 26;
        const dot = layer.appendChild(document.createElement('div'));
        dot.textContent = String(opts.nummer);
        dot.style.cssText = `position:fixed;left:${r.left - pad - size / 2}px;top:${r.top - pad - size / 2}px;width:${size}px;height:${size}px;border-radius:50%;background:${opts.farbe};color:#fff;font:700 15px/26px system-ui,sans-serif;text-align:center;border:2px solid #fff;box-shadow:0 1px 3px rgba(0,0,0,.35)`;
      }
    },
    el,
    { nummer, rahmen, farbe: FARBE, layer: LAYER },
  );
}

/** Legt einen Mauszeiger auf das Element (etwas rechts unterhalb der Mitte). */
export async function zeiger(selector) {
  const el = await element(selector);
  await browser.execute(
    (target, opts) => {
      const layer =
        document.getElementById(opts.layer) ??
        Object.assign(document.body.appendChild(document.createElement('div')), { id: opts.layer });
      layer.style.cssText = 'position:fixed;inset:0;pointer-events:none;z-index:2147483647';
      const r = target.getBoundingClientRect();
      const svg = layer.appendChild(document.createElement('div'));
      svg.innerHTML =
        '<svg width="22" height="30" viewBox="0 0 22 30"><path d="M2 2 L2 24 L8 18.5 L12 28 L16 26.3 L12 17 L20 17 Z" fill="#111" stroke="#fff" stroke-width="2" stroke-linejoin="round"/></svg>';
      svg.style.cssText = `position:fixed;left:${r.left + r.width * 0.6}px;top:${r.top + r.height * 0.55}px;filter:drop-shadow(0 1px 2px rgba(0,0,0,.4))`;
    },
    el,
    { layer: LAYER },
  );
}

/** Entfernt alle Markierungen. */
export async function entfernen() {
  await browser.execute((id) => document.getElementById(id)?.remove(), LAYER);
}

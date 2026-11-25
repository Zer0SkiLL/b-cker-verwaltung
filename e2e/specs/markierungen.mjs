// Markierungen für Handbuch-Bildschirmfotos: Rahmen und nummerierte Kreise.
// Sie werden direkt in die Seite gelegt und richten sich nach der echten Position des Elements,
// wandern also mit, wenn sich das Layout ändert. Nach dem Foto mit entfernen() wegnehmen.

const FARBE = '#1c7ed6'; // kräftiges Blau, hebt sich vom Braun des Programms ab
const LAYER = 'mv-markierungen';

/** Element per WebdriverIO-Selektor holen und warten, bis es sichtbar ist. */
async function element(selector) {
  const el = await $(selector);
  await el.waitForDisplayed();
  return el;
}

/**
 * Markiert ein Element: `nummer` setzt einen Kreis links daneben, `rahmen` (Standard) zieht einen Rahmen.
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
        const gap = 6;
        // Links neben dem Element, vertikal mittig; ganz am linken Rand stattdessen auf die Ecke.
        const left = r.left - pad - size - gap;
        const x = left >= 2 ? left : r.left - pad - size / 2;
        const y = left >= 2 ? r.top + r.height / 2 - size / 2 : r.top - pad - size / 2;
        const dot = layer.appendChild(document.createElement('div'));
        dot.textContent = String(opts.nummer);
        dot.style.cssText = `position:fixed;left:${x}px;top:${y}px;width:${size}px;height:${size}px;border-radius:50%;background:${opts.farbe};color:#fff;font:700 15px/26px system-ui,sans-serif;text-align:center;border:2px solid #fff;box-shadow:0 1px 3px rgba(0,0,0,.35)`;
      }
    },
    el,
    { nummer, rahmen, farbe: FARBE, layer: LAYER },
  );
}

/** Beantwortet Datei-Dialoge ohne Fenster (z.B. „Excel-Datei wählen…“) mit festen Pfaden. */
export async function dialogeBeantworten(antworten) {
  const ok = await browser.execute((answers) => {
    const internals = window.__TAURI_INTERNALS__;
    const original = internals.invoke.bind(internals);
    const wrapped = (cmd, args, options) =>
      cmd in answers ? Promise.resolve(answers[cmd]) : original(cmd, args, options);
    try {
      Object.defineProperty(internals, 'invoke', { value: wrapped, configurable: true, writable: true });
    } catch {
      // schreibgeschützt: unten über einen Proxy ersetzen
    }
    if (window.__TAURI_INTERNALS__.invoke !== wrapped) {
      const proxy = new Proxy(internals, {
        get: (target, prop) => (prop === 'invoke' ? wrapped : Reflect.get(target, prop)),
      });
      Object.defineProperty(window, '__TAURI_INTERNALS__', { value: proxy, configurable: true, writable: true });
    }
    return window.__TAURI_INTERNALS__.invoke === wrapped;
  }, antworten);
  if (!ok) throw new Error('Dateidialog liess sich nicht vorbeantworten.');
}

/** Ruft einen Programmbefehl direkt auf (nur für die Vorbereitung von Bildern). */
export async function befehl(cmd, args) {
  return browser.executeAsync(
    (c, a, done) => window.__TAURI_INTERNALS__.invoke(c, a).then(done, (e) => done({ fehler: String(e) })),
    cmd,
    args,
  );
}

/** Entfernt alle Markierungen. */
export async function entfernen() {
  await browser.execute((id) => document.getElementById(id)?.remove(), LAYER);
}

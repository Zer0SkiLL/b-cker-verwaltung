// Versionsnummern lesen, vergleichen und in Cargo.toml / JSON-Dateien setzen.

const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;

export function parseSemver(text) {
  const match = SEMVER.exec(String(text).trim());
  if (!match) throw new Error(`Ungültige Version: ${text} (erwartet z.B. 0.2.0)`);
  const [major, minor, patch] = match.slice(1).map(Number);
  return { major, minor, patch };
}

export function isGreater(a, b) {
  const x = parseSemver(a);
  const y = parseSemver(b);
  if (x.major !== y.major) return x.major > y.major;
  if (x.minor !== y.minor) return x.minor > y.minor;
  return x.patch > y.patch;
}

/** Ersetzt nur `version = "…"` im Abschnitt [package]. */
export function setCargoVersion(text, version) {
  parseSemver(version);
  let inPackage = false;
  let done = false;
  return text
    .split('\n')
    .map((line) => {
      if (/^\[.*\]\s*$/.test(line)) inPackage = line.trim() === '[package]';
      if (inPackage && !done && /^version\s*=/.test(line)) {
        done = true;
        return `version = "${version}"`;
      }
      return line;
    })
    .join('\n');
}

export function readCargoVersion(text) {
  const match = /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(text);
  if (!match) throw new Error('Keine Version in Cargo.toml gefunden');
  return match[1];
}

/** Ersetzt den ersten "version"-Eintrag und erhält die übrige Formatierung. */
export function setJsonVersion(text, version) {
  parseSemver(version);
  return text.replace(/("version"\s*:\s*")[^"]*(")/, `$1${version}$2`);
}

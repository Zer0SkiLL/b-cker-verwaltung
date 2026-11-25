//! Erzeugt eine fiktive Mitgliederliste im Aufbau der bisherigen Excel-Liste (zum Ausprobieren
//! des Imports). Alle Namen, Adressen und Nummern sind erfunden.
//! Aufruf: cargo run -p verwaltung_core --example musterliste -- <ordner | datei.xlsx>
//! Mit einem Ordner entstehen vier Listen mit unterschiedlichen Fällen (siehe LIESMICH.txt).
//!
//! Enthalten sind auch die Fälle, die der Import abfangen soll: mehrere Personen eines Betriebs,
//! abweichende Angaben, gleicher Firmenname an anderer Adresse, Einzelmitglieder, nur Vorname,
//! leere Zeilen, Jahr statt Datum, ungültiges Datum, doppelte Mandate.

use std::path::Path;
use std::process::ExitCode;

use rust_xlsxwriter::{ExcelDateTime, Format, Workbook, XlsxError};

const HEADERS: [&str; 26] = [
    "OBC-ID",
    "Kt Verband",
    "Firma",
    "Name",
    "Vorname",
    "Zusatz/Inhaber (Handelsregister)",
    "Adresszusatz",
    "Adresse",
    "PLZ",
    "Ort",
    "Kategorie",
    "Mail",
    "Home",
    "Lehrbetrieb",
    "Fr Bäck",
    "Fr Conf",
    "DH",
    "Besonderes",
    "Eintritt",
    "Austritt",
    "Filialen",
    "Telefon",
    "Mobiltelefon",
    "Kostenstellen-Nr",
    "Mandat",
    "Handelsregister-Nr",
];

#[derive(Clone, Copy)]
enum V<'a> {
    T(&'a str),
    N(f64),
    B(bool),
    /// Excel-Datum (Jahr, Monat, Tag)
    D(u16, u8, u8),
}
use V::{B, D, N, T};

type Row = &'static [(&'static str, V<'static>)];

const MIT: V<'static> = T("Mitglied mit Geschäft");
const OHNE: V<'static> = T("Mitglied ohne Geschäft");
const OHNE_SBC: V<'static> = T("Mitglied ohne SBC");
const KEIN: V<'static> = T("kein Mitglied");
const SG: V<'static> = T("Bäcker-Confiseurverband St. Gallen (fiktiv)");
const TG: V<'static> = T("Bäcker-Confiseurverband Thurgau (fiktiv)");
const AR: V<'static> = T("Bäcker-Confiseurverband Appenzell (fiktiv)");

/// Ja/Nein-Spalten wie in der echten Liste als Excel-Wahrheitswerte.
const fn flags(lehr: bool, baeck: bool, conf: bool, dh: bool) -> [(&'static str, V<'static>); 4] {
    [
        ("Lehrbetrieb", B(lehr)),
        ("Fr Bäck", B(baeck)),
        ("Fr Conf", B(conf)),
        ("DH", B(dh)),
    ]
}

const ROWS: &[Row] = &[
    // Ein Betrieb, eine Person.
    &[
        ("OBC-ID", N(1001.0)),
        ("Kt Verband", SG),
        ("Firma", T("Bäckerei Sonnenkorn AG")),
        ("Name", T("Muster")),
        ("Vorname", T("Anna")),
        ("Zusatz/Inhaber (Handelsregister)", T("Anna Muster")),
        ("Adresse", T("Musterstrasse 1")),
        ("PLZ", N(9000.0)),
        ("Ort", T("St. Gallen")),
        ("Kategorie", MIT),
        ("Mail", T("info@sonnenkorn.example")),
        ("Home", T("www.sonnenkorn.example")),
        ("Lehrbetrieb", B(true)),
        ("Fr Bäck", B(true)),
        ("Fr Conf", B(false)),
        ("DH", B(false)),
        ("Eintritt", D(1998, 3, 1)),
        ("Filialen", N(2.0)),
        ("Telefon", T("071 000 10 01")),
        ("Mandat", T("Vorstand; Präsident")),
        ("Handelsregister-Nr", T("CHE-100.000.001")),
    ],
    // Ein Betrieb mit drei Personen (gleiche Firma, PLZ, Adresse). Zweite Zeile mit eigener Mail,
    // dritte mit abweichender Kategorie (erste Zeile gilt, Hinweis im Bericht).
    &[
        ("OBC-ID", N(1002.0)),
        ("Kt Verband", TG),
        ("Firma", T("Confiserie Zuckerwerk GmbH")),
        ("Name", T("Beispiel")),
        ("Vorname", T("Marco")),
        ("Adresse", T("Bahnhofplatz 5")),
        ("PLZ", N(8500.0)),
        ("Ort", T("Frauenfeld")),
        ("Kategorie", MIT),
        ("Mail", T("hallo@zuckerwerk.example")),
        ("Lehrbetrieb", B(true)),
        ("Fr Bäck", B(false)),
        ("Fr Conf", B(true)),
        ("DH", B(false)),
        ("Eintritt", T("15.04.2005")),
        ("Telefon", T("052 000 20 02")),
        ("Kostenstellen-Nr", T("KS-2002")),
    ],
    &[
        ("Kt Verband", TG),
        ("Firma", T("Confiserie Zuckerwerk GmbH")),
        ("Name", T("Beispiel")),
        ("Vorname", T("Laura")),
        ("Adresse", T("Bahnhofplatz 5")),
        ("PLZ", N(8500.0)),
        ("Ort", T("Frauenfeld")),
        ("Kategorie", MIT),
        ("Mail", T("laura@zuckerwerk.example")),
        ("Mobiltelefon", T("079 000 20 03")),
        ("Mandat", T("Prüfungsexpertin")),
    ],
    &[
        ("Firma", T("Confiserie Zuckerwerk GmbH")),
        ("Name", T("Probst")),
        ("Vorname", T("Jonas")),
        ("Adresse", T("Bahnhofplatz 5")),
        ("PLZ", N(8500.0)),
        ("Ort", T("Frauenfeld")),
        ("Kategorie", OHNE),
        ("Besonderes", T("Lernender im 3. Lehrjahr")),
    ],
    // Gleicher Firmenname, andere Adresse: eigener Betrieb (Hinweis im Bericht).
    &[
        ("OBC-ID", N(1003.0)),
        ("Kt Verband", TG),
        ("Firma", T("Confiserie Zuckerwerk GmbH")),
        ("Name", T("Beispiel")),
        ("Vorname", T("Marco")),
        ("Adresse", T("Marktgasse 12")),
        ("PLZ", N(8570.0)),
        ("Ort", T("Weinfelden")),
        ("Kategorie", MIT),
        ("Besonderes", T("Filiale")),
    ],
    // Betrieb ohne Person (nur Firma), zweite Zeile ohne Person mit anderer Telefonnummer.
    &[
        ("OBC-ID", N(1004.0)),
        ("Kt Verband", AR),
        ("Firma", T("Backstube am Bach")),
        ("Adresse", T("Dorfstrasse 3")),
        ("PLZ", N(9100.0)),
        ("Ort", T("Herisau")),
        ("Kategorie", MIT),
        ("Telefon", T("071 000 40 04")),
        ("Lehrbetrieb", B(false)),
        ("Fr Bäck", B(true)),
        ("Fr Conf", B(false)),
        ("DH", B(true)),
        ("Eintritt", N(2012.0)),
    ],
    &[
        ("Firma", T("Backstube am Bach")),
        ("Adresse", T("Dorfstrasse 3")),
        ("PLZ", N(9100.0)),
        ("Ort", T("Herisau")),
        ("Telefon", T("071 000 40 05")),
        ("Mandat", T("Revisor, revisor")),
    ],
    // Ausgetretener Betrieb.
    &[
        ("OBC-ID", N(1005.0)),
        ("Kt Verband", SG),
        ("Firma", T("Bäckerei Alte Mühle")),
        ("Name", T("Keller")),
        ("Vorname", T("Peter")),
        ("Adresse", T("Mühleweg 7")),
        ("PLZ", N(9500.0)),
        ("Ort", T("Wil SG")),
        ("Kategorie", MIT),
        ("Eintritt", D(1985, 1, 1)),
        ("Austritt", D(2024, 12, 31)),
        ("Besonderes", T("Betrieb aufgegeben")),
    ],
    // Einzelmitglieder (ohne Firma).
    &[
        ("OBC-ID", N(2001.0)),
        ("Kt Verband", SG),
        ("Name", T("Frei")),
        ("Vorname", T("Sandra")),
        ("Adresse", T("Lindenweg 4")),
        ("PLZ", N(9200.0)),
        ("Ort", T("Gossau SG")),
        ("Kategorie", OHNE),
        ("Mail", T("sandra.frei@mail.example")),
        ("Eintritt", T("01.08.2019")),
        ("Mandat", T("Vorstand")),
    ],
    &[
        ("OBC-ID", N(2002.0)),
        ("Name", T("Huber")),
        ("Vorname", T("Thomas")),
        ("Adresse", T("Seeblick 9")),
        ("PLZ", N(8590.0)),
        ("Ort", T("Romanshorn")),
        ("Kategorie", OHNE_SBC),
        ("Fr Bäck", B(true)),
        ("Eintritt", T("Frühling 2010")),
    ],
    // Nur Vorname angegeben (wird als Nachname übernommen, Hinweis).
    &[
        ("Vorname", T("Rosa")),
        ("PLZ", N(9050.0)),
        ("Ort", T("Appenzell")),
        ("Kategorie", KEIN),
    ],
    // Weder Firma noch Name: wird übersprungen.
    &[
        ("Ort", T("St. Gallen")),
        ("Besonderes", T("Zeile ohne Namen")),
    ],
];

/// Weitere einfache Betriebe, damit die Liste nach Produktion aussieht.
const MORE: [(&str, &str, &str, f64, &str); 24] = [
    ("Bäckerei Morgenrot", "Huber", "Lena", 9000.0, "St. Gallen"),
    (
        "Brotzeit Bäckerei-Café",
        "Meier",
        "Urs",
        9008.0,
        "St. Gallen",
    ),
    ("Konditorei Himmelblau", "Graf", "Nina", 9230.0, "Flawil"),
    ("Bäckerei Ährenfeld", "Brunner", "Simon", 9240.0, "Uzwil"),
    ("Café Mandelblüte", "Steiner", "Eva", 9400.0, "Rorschach"),
    (
        "Bäckerei Kornblume GmbH",
        "Fischer",
        "Daniel",
        9430.0,
        "St. Margrethen",
    ),
    (
        "Confiserie Kakaobohne",
        "Weber",
        "Mia",
        9450.0,
        "Altstätten",
    ),
    ("Bäckerei zum Ofen", "Schmid", "Luca", 9470.0, "Buchs SG"),
    ("Backhaus Talblick", "Keller", "Sara", 9630.0, "Wattwil"),
    (
        "Bäckerei Sauerteig",
        "Baumann",
        "Reto",
        9642.0,
        "Ebnat-Kappel",
    ),
    (
        "Konditorei Rosengarten",
        "Frei",
        "Julia",
        8280.0,
        "Kreuzlingen",
    ),
    ("Bäckerei Hafenbrot", "Moser", "Marc", 8590.0, "Romanshorn"),
    (
        "Brotmanufaktur Thur",
        "Zimmermann",
        "Lea",
        8500.0,
        "Frauenfeld",
    ),
    ("Bäckerei Dinkelhof", "Wyss", "Pascal", 8360.0, "Eschlikon"),
    (
        "Confiserie Nougat & Co.",
        "Gerber",
        "Anja",
        8580.0,
        "Amriswil",
    ),
    ("Bäckerei Bergkorn", "Roth", "Fabian", 9100.0, "Herisau"),
    (
        "Backstube Säntisblick",
        "Koch",
        "Claudia",
        9104.0,
        "Waldstatt",
    ),
    ("Bäckerei Gipfeli", "Bühler", "Michael", 9410.0, "Heiden"),
    (
        "Konditorei Alpenrose",
        "Lüthi",
        "Sabrina",
        9050.0,
        "Appenzell",
    ),
    ("Bäckerei Ruchbrot", "Marti", "Stefan", 9056.0, "Gais"),
    ("Café Zimtstern", "Egli", "Tanja", 9320.0, "Arbon"),
    ("Bäckerei Feldweg", "Kunz", "Andreas", 9306.0, "Freidorf"),
    ("Backwerk Rheintal", "Hofer", "Monika", 9442.0, "Berneck"),
    ("Bäckerei Hefezopf", "Suter", "Patrick", 9500.0, "Wil SG"),
];

/// Zelle mit eigenem Text (für erzeugte Zeilen).
#[derive(Clone)]
enum Cell {
    T(String),
    N(f64),
    B(bool),
    D(u16, u8, u8),
}

impl From<V<'_>> for Cell {
    fn from(v: V<'_>) -> Self {
        match v {
            T(s) => Cell::T(s.to_string()),
            N(n) => Cell::N(n),
            B(b) => Cell::B(b),
            D(y, m, d) => Cell::D(y, m, d),
        }
    }
}

/// Eine Zeile als (Spaltenname, Wert); `None` ist eine leere Zeile.
type Line = Option<Vec<(String, Cell)>>;

fn line(cells: &[(&str, V<'_>)]) -> Line {
    Some(
        cells
            .iter()
            .map(|(h, v)| (h.to_string(), Cell::from(*v)))
            .collect(),
    )
}

fn text(s: &str) -> Cell {
    Cell::T(s.to_string())
}

/// Schreibt ein Blatt; Werte zu Spalten, die es nicht gibt, werden weggelassen.
fn save(path: &Path, headers: &[&str], lines: &[Line]) -> Result<usize, XlsxError> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Mitglieder")?;
    let bold = Format::new().set_bold();
    let date = Format::new().set_num_format("dd.mm.yyyy");
    for (c, h) in headers.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *h, &bold)?;
    }
    for (i, l) in lines.iter().enumerate() {
        let r = i as u32 + 1;
        for (h, v) in l.iter().flatten() {
            let Some(c) = headers.iter().position(|x| x == h) else {
                continue;
            };
            let c = c as u16;
            match v {
                Cell::T(s) => ws.write_string(r, c, s).map(|_| ())?,
                Cell::N(n) => ws.write_number(r, c, *n).map(|_| ())?,
                Cell::B(b) => ws.write_boolean(r, c, *b).map(|_| ())?,
                Cell::D(y, m, d) => ws
                    .write_datetime_with_format(r, c, ExcelDateTime::from_ymd(*y, *m, *d)?, &date)
                    .map(|_| ())?,
            }
        }
    }
    wb.save(path)?;
    Ok(lines.iter().filter(|l| l.is_some()).count())
}

fn kanton(plz: f64) -> V<'static> {
    if (9000.0..9100.0).contains(&plz) || plz >= 9200.0 {
        SG
    } else if plz < 9000.0 {
        TG
    } else {
        AR
    }
}

/// Ein einfacher Betrieb mit einer Person (Nummer `i` macht Mail, Telefon und Adresse eindeutig).
fn simple_firm(i: usize, firma: &str, name: &str, vorname: &str, plz: f64, ort: &str) -> Line {
    let cats = [MIT, MIT, MIT, OHNE, KEIN];
    let mut cells = vec![
        ("OBC-ID".to_string(), Cell::N(1100.0 + i as f64)),
        ("Kt Verband".into(), kanton(plz).into()),
        ("Firma".into(), text(firma)),
        ("Name".into(), text(name)),
        ("Vorname".into(), text(vorname)),
        ("Adresse".into(), Cell::T(format!("Hauptstrasse {}", i + 2))),
        ("PLZ".into(), Cell::N(plz)),
        ("Ort".into(), text(ort)),
        ("Kategorie".into(), cats[i % cats.len()].into()),
        (
            "Mail".into(),
            Cell::T(format!("info{}@betrieb.example", i + 10)),
        ),
        (
            "Telefon".into(),
            Cell::T(format!("071 000 {:02} {:02}", i / 100, i % 100)),
        ),
        (
            "Eintritt".into(),
            Cell::D(1980 + (i as u16 % 44), 1 + (i as u8 % 12), 1),
        ),
    ];
    for (h, v) in flags(
        i.is_multiple_of(3),
        i % 4 != 3,
        i % 4 == 3,
        i.is_multiple_of(5),
    ) {
        cells.push((h.to_string(), v.into()));
    }
    Some(cells)
}

/// Standardliste: alle Sonderfälle plus 24 einfache Betriebe (36 Zeilen).
fn normal() -> Vec<Line> {
    let mut lines: Vec<Line> = ROWS.iter().map(|r| line(r)).collect();
    lines.push(None); // leere Zeile mitten in der Liste
    for (i, (firma, name, vorname, plz, ort)) in MORE.iter().enumerate() {
        lines.push(simple_firm(i, firma, name, vorname, *plz, ort));
    }
    lines
}

const PREFIXES: [&str; 6] = [
    "Bäckerei",
    "Confiserie",
    "Konditorei",
    "Backstube",
    "Café",
    "Brothaus",
];

/// Etwa so gross wie die echte Liste: rund 290 Zeilen, mit zusammengefassten Betrieben.
fn gross() -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..240 {
        let (_, name, vorname, plz, ort) = MORE[i % MORE.len()];
        let firma = format!("{} {} {}", PREFIXES[i % PREFIXES.len()], name, i + 1);
        let first = simple_firm(i, &firma, name, vorname, plz, ort);
        let street = format!("Hauptstrasse {}", i + 2);
        lines.push(first);
        // Jeder sechste Betrieb hat eine zweite Person.
        if i.is_multiple_of(6) {
            let (_, name2, vorname2, _, _) = MORE[(i + 7) % MORE.len()];
            lines.push(Some(vec![
                ("Firma".into(), text(&firma)),
                ("Name".into(), text(name2)),
                ("Vorname".into(), text(vorname2)),
                ("Adresse".into(), text(&street)),
                ("PLZ".into(), Cell::N(plz)),
                ("Ort".into(), text(ort)),
                (
                    "Mandat".into(),
                    text(if i.is_multiple_of(12) {
                        "Vorstand"
                    } else {
                        "Prüfungsexperte"
                    }),
                ),
            ]));
        }
        // Jeder zwanzigste ist ausgetreten.
        if i % 20 == 19 {
            if let Some(Some(cells)) = lines.last_mut() {
                cells.push(("Austritt".into(), Cell::D(2023, 12, 31)));
            }
        }
    }
    for i in 0..12 {
        let (_, name, vorname, plz, ort) = MORE[(i * 5) % MORE.len()];
        lines.push(Some(vec![
            ("OBC-ID".into(), Cell::N(3000.0 + i as f64)),
            ("Name".into(), text(name)),
            ("Vorname".into(), text(vorname)),
            ("PLZ".into(), Cell::N(plz)),
            ("Ort".into(), text(ort)),
            ("Kategorie".into(), OHNE.into()),
        ]));
    }
    lines
}

/// Andere Spaltennamen, andere Reihenfolge, eine Spalte fehlt, eine ist zusätzlich:
/// zum Ausprobieren der Spaltenzuordnung.
fn andere_spalten(path: &Path) -> Result<usize, XlsxError> {
    let renames = [
        ("Firma", "Firmenname"),
        ("Name", "Nachname"),
        ("Mail", "E-Mail"),
        ("Home", "Webseite"),
        ("Adresse", "Strasse"),
    ];
    fn renamed<'a>(renames: &[(&'a str, &'a str)], h: &'a str) -> &'a str {
        renames.iter().find(|(a, _)| *a == h).map_or(h, |(_, b)| *b)
    }
    let mut headers: Vec<&str> = HEADERS
        .iter()
        .filter(|h| **h != "Kt Verband")
        .map(|h| renamed(&renames, h))
        .collect();
    headers.rotate_left(2); // Firma und Nachname nicht mehr vorne
    headers.push("Interne Notiz");
    let lines: Vec<Line> = normal()
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            l.map(|cells| {
                let mut cells: Vec<_> = cells
                    .into_iter()
                    .map(|(h, v)| (renamed(&renames, &h).to_string(), v))
                    .collect();
                if i.is_multiple_of(4) {
                    cells.push(("Interne Notiz".into(), text("nur intern, nicht übernehmen")));
                }
                cells
            })
        })
        .collect();
    save(path, &headers, &lines)
}

/// Unsaubere Werte, wie sie in gewachsenen Listen vorkommen.
fn fehlerhaft() -> Vec<Line> {
    let row = |cells: Vec<(&str, Cell)>| -> Line {
        Some(cells.into_iter().map(|(h, v)| (h.to_string(), v)).collect())
    };
    vec![
        row(vec![
            ("Firma", text("Bäckerei Sonnenkorn AG")),
            ("Name", text("Muster")),
            ("Vorname", text("Anna")),
            ("Adresse", text("Musterstrasse 1")),
            ("PLZ", Cell::N(9000.0)),
            ("Ort", text("St. Gallen")),
            ("Kategorie", text("Mitglied mit Geschäft")),
            ("Lehrbetrieb", text("ja")),
            ("Fr Bäck", text("x")),
            ("Eintritt", text("31.02.2020")),
        ]),
        // Gleicher Betrieb, anders geschrieben (Leerzeichen, Kleinbuchstaben): wird zusammengefasst.
        row(vec![
            ("Firma", text("  bäckerei  sonnenkorn ag ")),
            ("Name", text("Muster")),
            ("Vorname", text("Ben")),
            ("Adresse", text("musterstrasse 1")),
            ("PLZ", text(" 9000 ")),
            ("Ort", text("St. Gallen")),
            (
                "Mail",
                text("ben@sonnenkorn.example; info@sonnenkorn.example"),
            ),
        ]),
        row(vec![
            ("Firma", text("Confiserie Zuckerwerk GmbH")),
            ("Adresse", text("Bahnhofplatz 5")),
            ("PLZ", text("CH-8500")),
            ("Ort", text("Frauenfeld")),
            ("Kategorie", text("MITGLIED MIT GESCHÄFT")),
            ("Mail", text("info(at)zuckerwerk.example")),
            ("Lehrbetrieb", text("nein")),
            ("DH", text("vielleicht")),
            ("Eintritt", text("2020-05-01")),
            ("Austritt", text("05/2019")),
        ]),
        row(vec![
            ("Firma", text("Backstube am Bach")),
            ("Adresse", text("Dorfstrasse 3")),
            ("PLZ", Cell::N(9100.0)),
            ("Ort", text("Herisau")),
            ("Kategorie", text("Gönner")),
            ("Filialen", text("zwei")),
            ("Eintritt", Cell::N(1999.0)),
            ("Austritt", Cell::D(1995, 1, 1)),
        ]),
        row(vec![
            ("Firma", text("Bäckerei Alte Mühle")),
            ("Name", text("Keller")),
            ("Adresse", text("Mühleweg 7")),
            ("PLZ", Cell::N(9500.0)),
            ("Ort", text("Wil SG")),
            ("Besonderes", Cell::T("Sehr lange Bemerkung. ".repeat(20))),
            ("Mandat", text("Vorstand;;Präsident , , Vorstand")),
        ]),
        None,
        None,
        row(vec![("Name", text("   "))]),
        row(vec![
            ("Name", text("Huber")),
            ("Vorname", text("Thomas")),
            ("PLZ", Cell::N(8590.0)),
            ("Ort", text("Romanshorn")),
            ("Fr Conf", Cell::B(true)),
            ("Eintritt", text("1.1.10")),
        ]),
    ]
}

const LIESMICH: &str = "Fiktive Mitgliederlisten zum Ausprobieren des Excel-Imports.
Alle Namen, Adressen und Nummern sind erfunden. Neu erzeugen: pnpm testdaten

1 Standard.xlsx         Aufbau wie die echte Liste, 36 Zeilen mit allen Sonderfällen
                        (mehrere Personen pro Betrieb, Filiale, Einzelmitglieder, ungültige Daten).
2 Gross.xlsx            Rund 290 Zeilen, etwa so gross wie die echte Liste.
3 Andere Spalten.xlsx   Andere Spaltennamen und Reihenfolge, eine fehlt, eine zusätzlich:
                        zum Ausprobieren der Spaltenzuordnung.
4 Unsaubere Werte.xlsx  Tippfehler, Text statt Zahlen, unmögliche Daten, doppelte Einträge.

Importieren geht nur in eine leere Datei: zuerst \"Neue Datei anlegen…\".
";

fn write_all(dir: &Path) -> Result<(), XlsxError> {
    std::fs::create_dir_all(dir)?;
    let files: [(&str, Result<usize, XlsxError>); 4] = [
        (
            "1 Standard.xlsx",
            save(&dir.join("1 Standard.xlsx"), &HEADERS, &normal()),
        ),
        (
            "2 Gross.xlsx",
            save(&dir.join("2 Gross.xlsx"), &HEADERS, &gross()),
        ),
        (
            "3 Andere Spalten.xlsx",
            andere_spalten(&dir.join("3 Andere Spalten.xlsx")),
        ),
        (
            "4 Unsaubere Werte.xlsx",
            save(&dir.join("4 Unsaubere Werte.xlsx"), &HEADERS, &fehlerhaft()),
        ),
    ];
    for (name, result) in files {
        println!("{name}: {} Zeilen", result?);
    }
    std::fs::write(dir.join("LIESMICH.txt"), LIESMICH)?;
    Ok(())
}

fn main() -> ExitCode {
    let Some(target) = std::env::args().nth(1) else {
        eprintln!(
            "Aufruf: cargo run -p verwaltung_core --example musterliste -- <ordner | datei.xlsx>"
        );
        return ExitCode::FAILURE;
    };
    let target = Path::new(&target);
    let result = if target.extension().is_some_and(|e| e == "xlsx") {
        save(target, &HEADERS, &normal()).map(|n| println!("{}: {n} Zeilen", target.display()))
    } else {
        write_all(target)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}

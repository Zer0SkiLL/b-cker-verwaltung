//! Erzeugt eine fiktive Mitgliederliste im Aufbau der bisherigen Excel-Liste (zum Ausprobieren
//! des Imports). Alle Namen, Adressen und Nummern sind erfunden.
//! Aufruf: cargo run -p verwaltung_core --example musterliste -- <pfad.xlsx>
//!
//! Enthalten sind auch die Fälle, die der Import abfangen soll: mehrere Personen eines Betriebs,
//! abweichende Angaben, gleicher Firmenname an anderer Adresse, Einzelmitglieder, nur Vorname,
//! leere Zeilen, Jahr statt Datum, ungültiges Datum, doppelte Mandate.

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

fn write(path: &str) -> Result<usize, XlsxError> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Mitglieder")?;
    let bold = Format::new().set_bold();
    let date = Format::new().set_num_format("dd.mm.yyyy");
    for (c, h) in HEADERS.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *h, &bold)?;
    }
    let col = |h: &str| HEADERS.iter().position(|x| *x == h).expect("Spalte") as u16;
    let mut r = 1u32;
    let mut put = |r: u32, cells: &[(&str, V<'_>)]| -> Result<(), XlsxError> {
        for (h, v) in cells {
            let c = col(h);
            match *v {
                T(s) => ws.write_string(r, c, s).map(|_| ())?,
                N(n) => ws.write_number(r, c, n).map(|_| ())?,
                B(b) => ws.write_boolean(r, c, b).map(|_| ())?,
                D(y, m, d) => ws
                    .write_datetime_with_format(r, c, ExcelDateTime::from_ymd(y, m, d)?, &date)
                    .map(|_| ())?,
            }
        }
        Ok(())
    };
    for row in ROWS {
        put(r, row)?;
        r += 1;
    }
    // Leere Zeile mitten in der Liste (wird ignoriert).
    r += 1;
    let cats = [MIT, MIT, MIT, OHNE, KEIN];
    for (i, (firma, name, vorname, plz, ort)) in MORE.iter().enumerate() {
        let n = i as f64;
        let kt = if *plz >= 9000.0 && *plz < 9100.0 || *plz >= 9200.0 {
            SG
        } else if *plz < 9000.0 {
            TG
        } else {
            AR
        };
        let mail = format!("info{}@betrieb.example", i + 10);
        let phone = format!("071 000 5{} {:02}", i % 10, i + 10);
        let street = format!("Hauptstrasse {}", i + 2);
        put(
            r,
            &[
                ("OBC-ID", N(1100.0 + n)),
                ("Kt Verband", kt),
                ("Firma", T(firma)),
                ("Name", T(name)),
                ("Vorname", T(vorname)),
                ("Adresse", T(&street)),
                ("PLZ", N(*plz)),
                ("Ort", T(ort)),
                ("Kategorie", cats[i % cats.len()]),
                ("Mail", T(&mail)),
                ("Telefon", T(&phone)),
                ("Eintritt", D(1990 + (i as u16 % 30), 1 + (i as u8 % 12), 1)),
            ],
        )?;
        put(r, &flags(i % 3 == 0, i % 4 != 3, i % 4 == 3, i % 5 == 0))?;
        r += 1;
    }
    wb.save(path)?;
    Ok((r - 2) as usize)
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Aufruf: cargo run -p verwaltung_core --example musterliste -- <pfad.xlsx>");
        return ExitCode::FAILURE;
    };
    match write(&path) {
        Ok(rows) => {
            println!("{path}: {rows} Zeilen geschrieben");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}

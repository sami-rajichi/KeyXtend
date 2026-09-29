//! `capture-layouts`: writes the fake's golden layout tables from the layouts installed on this PC.
//! It only reads layouts: it never loads, adds or removes one, moves the mouse or types.
#![cfg(windows)]

use std::collections::BTreeMap;

use kx_module_api::ports::{KeyChars, LayoutId, LayoutSource, Scan};
use kx_platform_fake::{FILE_EXT, LayoutFile, golden_dir};
use kx_platform_windows::{WinLayouts, langinfo};
use serde::Deserialize;
use serde_json::{Value, json};

/// The subcommand's name.
pub const COMMAND: &str = "capture-layouts";
/// The first line of every table it writes.
const HEADER: &str = "# Captured from an installed layout by `kx-gates capture-layouts`; capture again rather than edit.\n";

/// `[capture]` in kx-gates.toml.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    /// ISO 639 languages to capture, each from its first installed layout.
    pub languages: Vec<String>,
    /// First scan code read, as `0x` hex.
    pub first: String,
    /// Last scan code read.
    pub last: String,
    /// Extended keys read after the range.
    pub extended: Vec<String>,
}

impl Capture {
    /// Every scan code to read: `first` to `last`, then `extended`.
    pub fn scans(&self) -> Result<Vec<Scan>, String> {
        let code = |text: &String| text.parse::<Scan>().map_err(|e| e.to_string());
        let (first, last) = (code(&self.first)?.code(), code(&self.last)?.code());
        if first > last {
            return Err(format!("capture: {} comes after {}", self.first, self.last));
        }
        let extended: Vec<Scan> = self.extended.iter().map(code).collect::<Result<_, _>>()?;
        Ok((first..=last).map(Scan::new).chain(extended).collect())
    }
}

/// One table to write: its file name and content.
pub struct Captured {
    /// The file name, such as `fr-fr.toml`.
    pub name: String,
    /// The table.
    pub file: LayoutFile,
}

/// The tables of `cfg`'s languages in `source`, each named by `name`, and the languages with no layout.
pub fn capture(
    source: &dyn LayoutSource,
    cfg: &Capture,
    name: impl Fn(LayoutId, &str) -> String,
) -> Result<(Vec<Captured>, Vec<String>), String> {
    let scans = cfg.scans()?;
    let (mut tables, mut missing) = (Vec::new(), Vec::new());
    for iso in &cfg.languages {
        match find(source, iso) {
            Some(id) => tables.push(Captured {
                name: name(id, iso),
                file: table(source, id, &scans),
            }),
            None => missing.push(iso.clone()),
        }
    }
    Ok((tables, missing))
}

/// The first installed layout whose language is `iso`.
fn find(source: &dyn LayoutSource, iso: &str) -> Option<LayoutId> {
    let mut installed = source.installed().into_iter();
    installed.find(|&id| source.facts(id).iso == iso)
}

/// The table of layout `id`: its facts, the keys that type a control, and every key that types a character.
pub fn table(source: &dyn LayoutSource, id: LayoutId, scans: &[Scan]) -> LayoutFile {
    let (mut controls, mut keys) = (Vec::new(), BTreeMap::new());
    for (&scan, chars) in scans.iter().zip(source.keys(id, scans)) {
        if chars != KeyChars::default() {
            keys.insert(scan, chars);
        } else if source.types_any(id, scan, &[]) {
            controls.push(scan);
        }
    }
    LayoutFile {
        id,
        facts: source.facts(id),
        controls,
        keys,
    }
}

/// The file name of a table: the layout's locale, such as `fr-fr.toml`, else its language; only letters, digits and `-` stay.
pub fn file_name(locale: Option<&str>, iso: &str) -> String {
    let plain = |c: &char| c.is_ascii_alphanumeric() || *c == '-';
    let stem: String = locale.unwrap_or(iso).chars().filter(plain).collect();
    format!("{}.{FILE_EXT}", stem.to_lowercase())
}

/// The file text: the header, then the table as TOML.
fn render(file: &LayoutFile) -> Result<String, String> {
    let table = file.to_toml().map_err(|e| e.to_string())?;
    Ok(format!("{HEADER}{table}"))
}

/// Captures the configured languages from this PC's layouts and writes their tables.
pub fn run(cfg: &Capture) -> Result<Value, String> {
    let name = |id, iso: &str| file_name(langinfo::locale(id).as_deref(), iso);
    let (tables, missing) = capture(&WinLayouts, cfg, name)?;
    let dir = golden_dir();
    let mut written = Vec::new();
    for Captured { name, file } in tables {
        let text = render(&file)?;
        std::fs::write(dir.join(&name), text).map_err(|e| format!("{name}: {e}"))?;
        written.push(json!({
            "file": name, "id": file.id.to_string(), "layout": file.facts.layout,
            "keys": file.keys.len(), "controls": file.controls.len(),
        }));
    }
    Ok(report(&written, &missing))
}

/// The command's answer; a missing language adds an `error` naming it, so the run fails.
fn report(written: &[Value], missing: &[String]) -> Value {
    let mut v = json!({ "command": COMMAND, "written": written, "missing": missing });
    if !missing.is_empty() {
        v["error"] = json!(format!("no installed layout for: {}", missing.join(", ")));
    }
    v
}

#[cfg(test)]
mod tests {
    use kx_platform_fake::FakeLayoutSource;

    use super::*;
    use crate::config;

    /// The fake's own example table, a slice of French AZERTY.
    const EXAMPLE: &str = "example";

    fn example() -> (FakeLayoutSource, LayoutFile) {
        let fake = FakeLayoutSource::golden(&[EXAMPLE]).unwrap();
        let path = golden_dir().join(EXAMPLE).with_extension(FILE_EXT);
        (fake, LayoutFile::read(&path).unwrap())
    }

    fn settings() -> Capture {
        config::load().expect("kx-gates.toml loads").capture
    }

    fn sorted(mut scans: Vec<Scan>) -> Vec<Scan> {
        scans.sort();
        scans
    }

    #[test]
    fn the_scan_list_runs_first_to_last_then_the_extended_keys() {
        let cfg = Capture {
            first: "0x0001".into(),
            last: "0x0003".into(),
            extended: vec!["0xE035".into()],
            ..settings()
        };
        let want = [0x01, 0x02, 0x03, 0xE035].map(Scan::new);
        assert_eq!(cfg.scans().unwrap(), want);
    }

    #[test]
    fn a_bad_or_reversed_range_is_refused() {
        for (first, last) in [
            ("zz", "0x0003"),
            ("0x0003", "0x0001"),
            ("0x0001", "0x10000"),
        ] {
            let cfg = Capture {
                first: first.into(),
                last: last.into(),
                ..settings()
            };
            assert!(cfg.scans().is_err(), "{first}..{last}");
        }
    }

    #[test]
    fn a_capture_of_the_example_table_gives_it_back() {
        let (fake, want) = example();
        let got = table(&fake, want.id, &settings().scans().unwrap());
        assert_eq!(
            (got.id, &got.facts, &got.keys),
            (want.id, &want.facts, &want.keys)
        );
        assert_eq!(sorted(got.controls), sorted(want.controls));
    }

    #[test]
    fn each_language_comes_from_its_first_layout_or_is_reported_missing() {
        let (fake, want) = example();
        let cfg = Capture {
            languages: vec!["en".into(), "fr".into(), "ar".into()],
            ..settings()
        };
        let (tables, missing) = capture(&fake, &cfg, |_, iso| format!("{iso}.toml")).unwrap();
        assert_eq!(missing, ["en", "ar"]);
        assert_eq!(tables.len(), 1);
        assert_eq!(
            (tables[0].name.as_str(), tables[0].file.id),
            ("fr.toml", want.id)
        );
        assert_eq!(find(&fake, "fr"), Some(want.id));
    }

    #[test]
    fn a_missing_language_is_an_error_that_names_it() {
        let v = report(&[], &["ar".into()]);
        let error = v["error"].as_str().unwrap_or_default();
        assert!(error.contains("ar"), "{v}");
        assert_eq!(v["missing"], json!(["ar"]));
        let v = report(&[json!({ "file": "fr-fr.toml" })], &[]);
        assert!(v.get("error").is_none(), "{v}");
    }

    #[test]
    fn the_file_is_named_after_the_locale_else_the_language() {
        assert_eq!(file_name(Some("fr-FR"), "fr"), "fr-fr.toml");
        assert_eq!(file_name(None, "ar"), "ar.toml");
        assert_eq!(file_name(Some(r"..\x/sr-Latn"), "sr"), "xsr-latn.toml");
    }

    #[test]
    fn the_written_text_starts_with_the_header_and_reads_back() {
        let (_, want) = example();
        let text = render(&want).unwrap();
        assert!(text.starts_with(HEADER), "{text}");
        assert_eq!(LayoutFile::parse(&text).unwrap(), want);
    }

    #[test]
    fn the_shipped_settings_name_languages_and_scans_and_the_help_names_the_command() {
        let cfg = settings();
        assert!(!cfg.languages.is_empty());
        assert!(!cfg.scans().unwrap().is_empty());
        assert!(crate::usage::usage().contains(COMMAND));
    }
}

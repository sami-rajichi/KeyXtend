//! Tests of the settings loader.

use super::*;
use kx_test_support::tempdir::TempDir;

/// The shipped settings file.
const SHIPPED: &str = include_str!("../../kx-target-window.toml");

/// A stand-in settings file that lives in `cfg/dir`.
fn file() -> &'static Path {
    Path::new("cfg/dir/kx-target-window.toml")
}

/// The shipped file with the line of `key` set to `value`, a TOML value.
fn with(key: &str, value: &str) -> String {
    let start = format!("{key} =");
    let mut hit = false;
    let lines: Vec<String> = SHIPPED
        .lines()
        .map(|line| {
            hit |= line.starts_with(&start);
            if line.starts_with(&start) {
                format!("{start} {value}")
            } else {
                line.to_string()
            }
        })
        .collect();
    assert!(hit, "the shipped file has no {key}");
    lines.join("\n")
}

#[test]
fn the_shipped_file_parses() {
    let cfg = parse(SHIPPED, file()).expect("the shipped file parses");
    assert!(!cfg.title.is_empty() && !cfg.password_arg.is_empty());
    assert!(cfg.font_px > 0 && cfg.width_px > 0 && cfg.height_px > 0);
}

#[test]
fn an_unknown_key_fails() {
    let text = format!("{SHIPPED}\nsurprise = 1\n");
    let err = parse(&text, file()).expect_err("unknown key");
    assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
    assert!(err.to_string().contains("surprise"), "{err}");
}

#[test]
fn a_missing_key_fails() {
    let text: String = SHIPPED
        .lines()
        .filter(|l| !l.starts_with("font_px"))
        .collect::<Vec<_>>()
        .join("\n");
    let err = parse(&text, file()).expect_err("missing key");
    assert!(matches!(err, ConfigError::Parse { .. }), "{err}");
}

#[test]
fn an_empty_text_value_fails() {
    for key in ["title", "log", "font", "password_arg"] {
        let err = parse(&with(key, "\"\""), file()).expect_err(key);
        assert!(
            matches!(err, ConfigError::Empty { key: k, .. } if k == key),
            "{err}"
        );
    }
}

#[test]
fn a_zero_or_negative_size_fails() {
    for key in ["font_px", "width_px", "height_px"] {
        for bad in ["0", "-5"] {
            let err = parse(&with(key, bad), file()).expect_err(key);
            let named = matches!(err, ConfigError::NotPositive { key: k, .. } if k == key);
            assert!(named, "{key} = {bad}: {err}");
        }
    }
}

#[test]
fn a_relative_log_starts_at_the_files_folder() {
    let cfg = parse(&with("log", "\"out/x.log\""), file()).expect("parses");
    assert_eq!(cfg.log, Path::new("cfg/dir/out/x.log"));
}

#[test]
fn an_absolute_log_stays_as_it_is() {
    let abs = std::env::temp_dir().join("abs.log");
    let cfg = parse(&with("log", &format!("'{}'", abs.display())), file()).expect("parses");
    assert_eq!(cfg.log, abs);
}

#[test]
fn every_error_names_the_file() {
    let bad = [
        with("title", "\"\""),
        with("width_px", "0"),
        format!("{SHIPPED}\nsurprise = 1\n"),
    ];
    for text in bad {
        let err = parse(&text, file()).expect_err("bad settings");
        assert!(err.to_string().contains("kx-target-window.toml"), "{err}");
    }
}

#[test]
fn the_file_beside_the_exe_wins_when_it_exists() {
    let (exe, krate) = (Path::new("exe/dir"), Path::new("crate/dir"));
    assert_eq!(pick(FILE, Some(exe), krate, true), exe.join(FILE));
    assert_eq!(pick(FILE, Some(exe), krate, false), krate.join(FILE));
    assert_eq!(pick(FILE, None, krate, true), krate.join(FILE));
    assert_eq!(pick(FILE, None, krate, false), krate.join(FILE));
    assert_eq!(
        pick("other.toml", Some(exe), krate, true),
        exe.join("other.toml")
    );
}

#[test]
fn locate_finds_a_file_beside_the_exe_else_the_crate_folder() {
    let dir = TempDir::new("tw-locate").expect("temp folder");
    let (exe, krate) = (dir.path().join("tool.exe"), Path::new("crate/dir"));
    assert_eq!(
        locate_for(Some(&exe), "x.toml", krate),
        krate.join("x.toml")
    );
    std::fs::write(dir.path().join("x.toml"), "").expect("temp file");
    let beside = locate_for(Some(&exe), "x.toml", krate);
    assert_eq!(beside, dir.path().join("x.toml"));
    assert_eq!(locate_for(None, "x.toml", krate), krate.join("x.toml"));
}

#[test]
fn a_test_run_loads_the_file_from_the_crate_folder() {
    let crate_file = Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE);
    assert_eq!(path(), crate_file);
    let cfg = load().expect("the crate folder's file loads");
    assert!(
        cfg.log.starts_with(env!("CARGO_MANIFEST_DIR")),
        "{:?}",
        cfg.log
    );
}

#[test]
fn a_missing_file_gives_the_read_error_naming_its_path() {
    let file = Path::new("no/such/dir").join(FILE);
    let err = load_from(&file).expect_err("no such file");
    assert!(matches!(err, ConfigError::Read { .. }), "{err}");
    assert!(
        err.to_string().starts_with(&file.display().to_string()),
        "{err}"
    );
}

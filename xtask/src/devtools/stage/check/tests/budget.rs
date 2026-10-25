//! The budget: the stage's download, its files deflated, must fit `max_mb`, whatever its installed size; that installed
//! size is held to the install's limit, and a megabyte is 10⁶ bytes, rounded up.

use super::*;
use crate::devtools::install::fits;
use crate::test_support::noise;

/// The spec, whose §10 row sets the download budget, from the workspace root.
const SPEC: &str = "docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md";
/// The start of that row.
const DOWNLOAD_ROW: &str = "| Download size, without voice models |";

/// A stage whose program holds `bytes`, and the test settings with a 1 MB budget.
fn program(bytes: &[u8]) -> (Fake, StageConfig) {
    let stage = Fake::new(&[(PROGRAM, &[])]);
    std::fs::write(stage.dir.path().join(PROGRAM), bytes).unwrap();
    let mut config = settings();
    config.max_mb = 1;
    (stage, config)
}

#[test]
fn a_stage_over_the_budget_installed_but_within_it_deflated_passes() {
    let size = usize::try_from(paths::BYTES_PER_MB).unwrap() * 2;
    let (stage, config) = program(&vec![0u8; size]);
    let summary = stage.check_with(&config).unwrap();
    assert_eq!((summary.raw_mb(), summary.mb()), (2, 1));
}

#[test]
fn a_stage_whose_download_passes_the_budget_is_refused() {
    let size = usize::try_from(paths::BYTES_PER_MB).unwrap() + 1;
    let (stage, mut config) = program(&noise(size));
    let over = StageError::OverBudget { mb: 2, max: 1 };
    assert_eq!(refusal(stage.check_with(&config)), Some(over));
    config.max_mb = 2;
    assert!(stage.check_with(&config).is_ok());
}

#[test]
fn the_summary_counts_files_installed_bytes_and_deflated_bytes() {
    let stage = Fake::new(&[(PROGRAM, &[]), ("qml/QtQuick/qmldir", &[])]);
    std::fs::write(stage.dir.path().join(PROGRAM), [0u8; 7]).unwrap();
    let level = crate::words::download_level().unwrap();
    let each = |b: &[u8]| u64::try_from(crate::deflate::deflated(b, level).unwrap()).unwrap();
    let deflated = each(&[0u8; 7]) + each(&[]);
    let want = Summary {
        files: 2,
        bytes: 7,
        deflated,
    };
    assert_eq!(stage.check(), Ok(want));
}

#[test]
fn the_shipped_budget_is_spec_10s_download_size() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let read = |rel: &str| std::fs::read_to_string(root.join(rel)).unwrap();
    let manifest: toml::Table = read("Cargo.toml").parse().unwrap();
    let max_mb = manifest["workspace"]["metadata"]["stage"]["max_mb"].as_integer();
    let spec = read(SPEC);
    let row = spec.lines().find(|l| l.starts_with(DOWNLOAD_ROW));
    let limit = row.and_then(|r| r.split("≤ ").nth(1)?.split_whitespace().next());
    assert_eq!(limit.and_then(|n| n.parse().ok()), max_mb, "{row:?}");
}

#[test]
fn a_megabyte_is_a_million_bytes_rounded_up_as_spec_10_counts() {
    assert_eq!((paths::mb(1_000_000), paths::mb(1_000_001)), (1, 2));
    let summary = Summary {
        files: 1,
        bytes: paths::BYTES_PER_MB + 1,
        deflated: paths::BYTES_PER_MB,
    };
    assert_eq!((summary.raw_mb(), summary.mb()), (2, 1));
}

#[test]
fn the_installed_size_is_held_to_the_install_limit() {
    let limit = 500;
    assert_eq!(fits(limit * paths::BYTES_PER_MB, limit), Ok(()));
    let over = DevError::TooBig {
        mb: limit + 1,
        max: limit,
    };
    assert_eq!(fits(limit * paths::BYTES_PER_MB + 1, limit), Err(over));
}

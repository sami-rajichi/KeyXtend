//! The budget: the stage's download, its files deflated, must fit `max_mb`, whatever its installed size; that installed
//! size is held to the install's limit, and a megabyte is 10⁶ bytes, rounded up.

use super::*;
use crate::devtools::install::fits;

/// `len` bytes that deflate cannot shrink: a xorshift stream.
fn noise(len: usize) -> Vec<u8> {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x.to_le_bytes()[0]
    };
    (0..len).map(|_| next()).collect()
}

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
fn a_megabyte_is_a_million_bytes_rounded_up_as_spec_10_and_g8_count() {
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

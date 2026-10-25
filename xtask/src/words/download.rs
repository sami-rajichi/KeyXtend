//! The download budget: every file `words build` writes is measured deflated, as a zip, MSIX or MSI carries it, and
//! a build over `[packs] download_kb` is refused.

use super::WordsError;
use super::manifest::Packs;
use crate::devtools::paths::BYTES_PER_KB;

/// The bytes of `bytes` deflated at `level`, by the measure the stage's budget shares (`crate::deflate`).
pub(crate) fn deflated(bytes: &[u8], level: u32) -> Result<usize, WordsError> {
    crate::deflate::deflated(bytes, level).map_err(|e| WordsError::Io(format!("deflate: {e}")))
}

/// The line that totals the files' raw and deflated `sizes`, or the refusal when the deflated total passes the budget.
pub(crate) fn check(sizes: &[(usize, usize)], packs: &Packs) -> Result<String, WordsError> {
    let raw: usize = sizes.iter().map(|s| s.0).sum();
    let deflated: usize = sizes.iter().map(|s| s.1).sum();
    let budget = packs.download_kb.saturating_mul(BYTES_PER_KB);
    let count = sizes.len();
    let line = format!(
        "download: {count} files, {raw} bytes, {deflated} deflated of the {budget} allowed"
    );
    if u64::try_from(deflated).is_ok_and(|d| d <= budget) {
        Ok(line)
    } else {
        Err(WordsError::Pack(format!("{line}; the build is refused")))
    }
}

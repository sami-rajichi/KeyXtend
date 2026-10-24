//! The download budget: every file `words build` writes is measured deflated, as a zip, MSIX or MSI carries it, and
//! a build over `[packs] download_kb` is refused.

use std::io::Write;

use flate2::Compression;
use flate2::write::DeflateEncoder;

use super::WordsError;
use super::manifest::Packs;
use crate::devtools::paths::BYTES_PER_KB;

/// The bytes of `bytes` deflated at `level`.
pub(crate) fn deflated(bytes: &[u8], level: u32) -> Result<usize, WordsError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
    let failed = |e: std::io::Error| WordsError::Io(format!("deflate: {e}"));
    encoder.write_all(bytes).map_err(failed)?;
    Ok(encoder.finish().map_err(failed)?.len())
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

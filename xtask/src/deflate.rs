//! The download measure every budget shares: bytes deflated, as a zip, MSIX or MSI carries them. It stands in,
//! conservatively, for the installer's LZMA2, which packs smaller.

use std::io::Write;

use flate2::Compression;
use flate2::write::DeflateEncoder;

/// The length of `bytes` deflated at `level`.
pub(crate) fn deflated(bytes: &[u8], level: u32) -> std::io::Result<usize> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?.len())
}

//! Screenshots of the whole desktop as BMP files, taken when something blocks a test.

use std::path::Path;

use spike_core::capture;

/// Saves the whole desktop to `path` as a BMP file, if it fits in `cap` bytes.
pub fn save(path: &Path, cap: usize) -> Result<(), String> {
    let bmp = capture::screen(cap)?.to_bmp()?;
    std::fs::write(path, bmp).map_err(|e| format!("{}: {e}", path.display()))
}

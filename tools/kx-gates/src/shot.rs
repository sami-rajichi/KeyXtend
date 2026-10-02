//! Screenshots as BMP files: the whole desktop when something blocks a test, and a box or a window that a gate times
//! or keeps for the owner's eye.
#![cfg(windows)]

use std::path::Path;

use spike_core::capture::{self, BYTES_PP, Shot};
use windows::Win32::Foundation::{HWND, RECT};

use crate::{out, win};

/// The file extension of the pictures `save` and `keep` write.
pub const EXT: &str = ".bmp";

/// The file name suffix `stem` with the picture extension, for a gate's output file.
pub fn named(stem: &str) -> String {
    format!("{stem}{EXT}")
}

/// Saves the whole desktop to `path` as a BMP file, if it fits in `cap` bytes.
pub fn save(path: &Path, cap: usize) -> Result<(), String> {
    let bmp = capture::screen(cap)?.to_bmp()?;
    std::fs::write(path, bmp).map_err(|e| format!("{}: {e}", path.display()))
}

/// A copy of screen box `r`, in physical pixels.
pub fn of_box(r: &RECT) -> Result<Shot, String> {
    let (width, height) = (r.right - r.left, r.bottom - r.top);
    let cap = usize::try_from(width.saturating_mul(height)).unwrap_or(0) * BYTES_PP as usize;
    let bgra = capture::grab(r.left, r.top, width, height, cap)?;
    Ok(Shot {
        left: r.left,
        top: r.top,
        width,
        height,
        bgra,
    })
}

/// A copy of window `hwnd` as it shows on screen.
pub fn of_window(hwnd: HWND) -> Result<Shot, String> {
    of_box(&win::rect(hwnd)?)
}

/// Saves `shot` to `path` as a BMP file.
pub fn keep(path: &Path, shot: &Shot) -> Result<(), String> {
    out::write_bytes(path, &shot.to_bmp()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_name_ends_in_the_format_save_and_keep_write() {
        let name = named("-look-3");
        assert!(name.starts_with("-look-3") && name.ends_with(EXT), "{name}");
        let bmp = Shot {
            left: 0,
            top: 0,
            width: 1,
            height: 1,
            bgra: vec![0; 4],
        }
        .to_bmp()
        .expect("one pixel");
        assert!(bmp.starts_with(b"BM"), "a BMP file, as `EXT` says");
    }
}

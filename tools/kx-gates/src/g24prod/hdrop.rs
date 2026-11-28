//! The files a clipboard file drop (`CF_HDROP`) names, read from its bytes, and whether one lies in a folder.

use std::path::{Component, Path, PathBuf};

use super::pic::u32_at;

/// Where `DROPFILES` says its names start.
const NAMES_AT: usize = 0;
/// Where `DROPFILES` says whether its names are wide.
const WIDE_AT: usize = 16;
/// Bytes of a wide character.
const WIDE: usize = 2;

/// The files a wide file drop `b` names, in order; an error for an ANSI, cut or empty drop.
pub fn dropped(b: &[u8]) -> Result<Vec<PathBuf>, String> {
    let short = || "the file drop is too short".to_string();
    let at = usize::try_from(u32_at(b, NAMES_AT).ok_or_else(short)?).map_err(|e| e.to_string())?;
    if u32_at(b, WIDE_AT).ok_or_else(short)? == 0 {
        return Err("the file drop names its files in ANSI, not wide".into());
    }
    let units: Vec<u16> = b
        .get(at..)
        .ok_or_else(short)?
        .as_chunks::<WIDE>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    let end = units
        .windows(2)
        .position(|w| w == [0, 0])
        .ok_or("the file drop has no end")?;
    let names: Vec<PathBuf> = units[..end]
        .split(|&u| u == 0)
        .map(|n| PathBuf::from(String::from_utf16_lossy(n)))
        .collect();
    if names.iter().any(|n| n.as_os_str().is_empty()) {
        return Err("the file drop names no file".into());
    }
    Ok(names)
}

/// True when `file` lies below folder `dir`, with no step up on the way.
pub fn inside(file: &Path, dir: &Path) -> bool {
    !file.components().any(|c| c == Component::ParentDir) && file.starts_with(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wide file drop naming `paths`, or an ANSI one when not `wide`.
    fn drop_of(paths: &[&str], wide: bool) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(20u32.to_le_bytes());
        b.extend([0u8; 12]);
        b.extend(u32::from(wide).to_le_bytes());
        for p in paths {
            b.extend(p.encode_utf16().chain([0]).flat_map(u16::to_le_bytes));
        }
        b.extend([0u8; 2]);
        b
    }

    #[test]
    fn a_wide_drop_names_each_file_in_order() {
        let got = dropped(&drop_of(&[r"D:\a\one.mp4", r"D:\b.mp4"], true));
        assert_eq!(
            got,
            Ok(vec![
                PathBuf::from(r"D:\a\one.mp4"),
                PathBuf::from(r"D:\b.mp4")
            ])
        );
    }

    #[test]
    fn an_ansi_short_or_empty_drop_is_refused() {
        assert!(dropped(&drop_of(&[r"D:\b.mp4"], false)).is_err(), "ansi");
        assert!(dropped(&drop_of(&[], true)).is_err(), "empty");
        assert!(dropped(&[0u8; 8]).is_err(), "short");
        let mut cut = drop_of(&[r"D:\b.mp4"], true);
        cut.truncate(24);
        assert!(dropped(&cut).is_err(), "no end");
    }

    #[test]
    fn a_file_lies_in_a_folder_only_below_it_with_no_step_up() {
        let dir = Path::new(r"D:\data");
        assert!(inside(Path::new(r"D:\data\snips\unsaved\a.mp4"), dir));
        assert!(!inside(Path::new(r"D:\data\..\a.mp4"), dir));
        assert!(!inside(Path::new(r"D:\other\a.mp4"), dir));
    }
}

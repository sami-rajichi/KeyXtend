//! A snip: the screen frozen when it starts, two clicked corners, and that region saved as a BMP file.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::capture::{self, Shot};
use crate::hold::Pt;

/// Start of every snip file name.
pub const PREFIX: &str = "snip-";
/// Snip file type.
pub const EXT: &str = "bmp";
/// The frozen screen, written only for a face that must load it from a file.
const FROZEN: &str = "frozen.bmp";

/// What a click did.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// The first corner is set.
    First(Pt),
    /// The region was saved to this file.
    Saved(PathBuf),
    /// The same point was clicked twice: nothing saved.
    Cancelled,
}

/// A snip in progress.
pub struct Snip {
    shot: Shot,
    first: Option<Pt>,
}

impl Snip {
    /// Freezes the whole screen now, if it fits in `cap` bytes.
    pub fn start(cap: usize) -> Result<Snip, String> {
        Ok(Snip::of(capture::screen(cap)?))
    }

    /// A snip of an existing copy.
    pub fn of(shot: Shot) -> Snip {
        Snip { shot, first: None }
    }

    /// The frozen screen.
    pub fn shot(&self) -> &Shot {
        &self.shot
    }

    /// Takes a click at `at`, in screen pixels; the second one saves the region into `dir`.
    pub fn click(&mut self, at: Pt, dir: &Path) -> Result<Step, String> {
        let Some(first) = self.first.take() else {
            self.first = Some(at);
            return Ok(Step::First(at));
        };
        if first == at {
            return Ok(Step::Cancelled);
        }
        let part = self
            .shot
            .crop(&capture::region(first, at))
            .ok_or(OFF_SCREEN)?;
        let file = dir.join(file_name(stamp_ms()));
        write(&file, &part.to_bmp()?)?;
        Ok(Step::Saved(file))
    }

    /// Takes a click at the mouse pointer; the note to show once the snip ends, or `None` while it goes on.
    pub fn pick(&mut self, dir: &Path) -> Option<String> {
        note(&crate::screen::cursor().and_then(|p| self.click(p, dir)))
    }

    /// Writes the frozen screen into `dir` for a face that shows it from a file; delete it with `forget_frozen`.
    pub fn frozen_file(&self, dir: &Path) -> Result<PathBuf, String> {
        let file = dir.join(FROZEN);
        write(&file, &self.shot.to_bmp()?)?;
        Ok(file)
    }
}

/// Shown when a snip was cancelled.
const CANCELLED: &str = "snip cancelled";
/// Starts the note for a saved snip, before the file name.
const SAVED: &str = "snip saved: ";
/// Starts the note for a failed snip, before the reason.
const FAILED: &str = "snip: ";
/// Why a region outside the frozen screen cannot be saved.
const OFF_SCREEN: &str = "the region is off the screen";

/// The status note for a click's result; `None` while the snip goes on.
pub fn note(step: &Result<Step, String>) -> Option<String> {
    match step {
        Ok(Step::First(_)) => None,
        Ok(Step::Saved(f)) => Some(format!(
            "{SAVED}{}",
            f.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        )),
        Ok(Step::Cancelled) => Some(CANCELLED.to_string()),
        Err(e) => Some(format!("{FAILED}{e}")),
    }
}

/// Deletes the frozen screen file from `dir`; a file that is already gone is fine.
pub fn forget_frozen(dir: &Path) -> Result<(), String> {
    let file = dir.join(FROZEN);
    match std::fs::remove_file(&file) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("{}: {e}", file.display()))
        }
        _ => Ok(()),
    }
}

/// Milliseconds since 1970, to name files.
fn stamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

/// The snip file name for time `ms`.
fn file_name(ms: u128) -> String {
    format!("{PREFIX}{ms}.{EXT}")
}

/// Writes `bytes` to `file`, making its folder first.
fn write(file: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(file, bytes).map_err(|e| format!("{}: {e}", file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A size cap far above the tiny test picture.
    const CAP: usize = 1 << 20;

    fn snip() -> Snip {
        Snip::of(Shot {
            left: 0,
            top: 0,
            width: 4,
            height: 4,
            bgra: (0..64).collect(),
        })
    }

    #[test]
    fn the_first_click_only_sets_a_corner() {
        let mut s = snip();
        let at = Pt { x: 1, y: 1 };
        assert_eq!(s.click(at, Path::new("unused")), Ok(Step::First(at)));
    }

    #[test]
    fn the_same_point_twice_cancels() {
        let mut s = snip();
        let at = Pt { x: 2, y: 2 };
        let dir = Path::new("unused");
        s.click(at, dir).expect("first");
        assert_eq!(s.click(at, dir), Ok(Step::Cancelled));
    }

    #[test]
    fn a_second_corner_saves_the_region_as_a_bmp() {
        let dir = std::env::temp_dir().join(format!("kx-snip-test-{}", std::process::id()));
        let mut s = snip();
        s.click(Pt { x: 2, y: 2 }, &dir).expect("first");
        let step = s.click(Pt { x: 1, y: 1 }, &dir).expect("second");
        let Step::Saved(file) = step else {
            panic!("not saved: {step:?}");
        };
        let back = std::fs::read(&file).map(|b| capture::from_bmp(&b, CAP));
        std::fs::remove_dir_all(&dir).expect("clean up");
        let want = s
            .shot()
            .crop(&capture::region(Pt { x: 1, y: 1 }, Pt { x: 2, y: 2 }));
        assert_eq!(
            back.expect("read").expect("bmp").bgra,
            want.expect("crop").bgra
        );
    }

    #[test]
    fn a_region_off_the_frozen_screen_is_an_error() {
        let mut s = snip();
        let dir = Path::new("unused");
        s.click(Pt { x: 10, y: 10 }, dir).expect("first");
        assert!(s.click(Pt { x: 12, y: 12 }, dir).is_err());
    }

    #[test]
    fn forgetting_a_missing_frozen_file_is_fine() {
        let dir = std::env::temp_dir().join(format!("kx-snip-none-{}", std::process::id()));
        assert_eq!(forget_frozen(&dir), Ok(()));
    }

    #[test]
    fn notes_come_only_when_the_snip_ends() {
        assert_eq!(note(&Ok(Step::First(Pt { x: 0, y: 0 }))), None);
        let saved = Ok(Step::Saved(PathBuf::from("d").join("snip-1.bmp")));
        assert_eq!(note(&saved), Some(format!("{SAVED}snip-1.bmp")));
        assert_eq!(note(&Ok(Step::Cancelled)).as_deref(), Some(CANCELLED));
        assert_eq!(note(&Err("x".into())), Some(format!("{FAILED}x")));
    }

    #[test]
    fn snip_files_are_named_by_time() {
        assert_eq!(file_name(1234), "snip-1234.bmp");
    }
}

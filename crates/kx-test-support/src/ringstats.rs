//! Frame gaps of the ring test (gate G12): how often the ring window drew, summed up when the test stops.

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use kx_module_api::{US_PER_MS, US_PER_S};
use serde::{Deserialize, Serialize};

use crate::pct::{P50, P99, percentile};

/// The ending of the file the stats are written to before they replace the old ones.
const PART: &str = "part";

/// The gaps between the ring's frames, up to a cap.
#[derive(Debug, Clone)]
pub struct Frames {
    gaps_us: Vec<u32>,
    cap: usize,
    capped: bool,
    /// When the last frame was drawn.
    last: Option<Instant>,
}

/// What the ring test measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameReport {
    /// Gaps kept: one fewer than the frames drawn, unless capped.
    pub gaps: usize,
    /// Time they span, in seconds.
    pub secs: f64,
    /// Frames per second.
    pub fps: f64,
    /// The typical gap, in ms.
    pub p50_ms: f64,
    /// 99 in 100 gaps are this or shorter, in ms.
    pub p99_ms: f64,
    /// The longest gap, in ms.
    pub max_ms: f64,
    /// The cap was reached, so later gaps were not kept.
    pub capped: bool,
}

/// Why a report could not be saved.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    /// The file or its folder could not be written.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file being saved.
        path: PathBuf,
        /// What the system said.
        source: io::Error,
    },
    /// The report could not be turned into JSON.
    #[error("the report is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl Frames {
    /// An empty record that keeps at most `cap` gaps.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            gaps_us: Vec::new(),
            cap,
            capped: false,
            last: None,
        }
    }

    /// Counts a frame drawn at `now`; the first one only starts the clock.
    pub fn tick(&mut self, now: Instant) {
        if let Some(before) = self.last.replace(now) {
            self.add(now.saturating_duration_since(before));
        }
    }

    /// Records one frame `gap` after the one before; a gap too long to count is kept as the longest countable.
    pub fn add(&mut self, gap: Duration) {
        if self.gaps_us.len() >= self.cap {
            self.capped = true;
            return;
        }
        self.gaps_us
            .push(u32::try_from(gap.as_micros()).unwrap_or(u32::MAX));
    }

    /// The record so far.
    #[must_use]
    pub fn report(&self) -> FrameReport {
        let mut ms: Vec<f64> = self
            .gaps_us
            .iter()
            .map(|&us| f64::from(us) / f64::from(US_PER_MS))
            .collect();
        ms.sort_by(f64::total_cmp);
        let total_us: f64 = self.gaps_us.iter().map(|&us| f64::from(us)).sum();
        let secs = total_us / f64::from(US_PER_S);
        let count = u32::try_from(ms.len()).unwrap_or(u32::MAX);
        let fps = if secs > 0.0 {
            f64::from(count) / secs
        } else {
            0.0
        };
        FrameReport {
            gaps: ms.len(),
            secs,
            fps,
            p50_ms: percentile(&ms, P50).unwrap_or_default(),
            p99_ms: percentile(&ms, P99).unwrap_or_default(),
            max_ms: ms.last().copied().unwrap_or_default(),
            capped: self.capped,
        }
    }
}

/// Writes `r` to `file` as JSON, making its folder if needed; the file appears whole, and a failed write leaves no part.
///
/// # Errors
///
/// When the report cannot be turned into JSON, or the folder or file cannot be written.
pub fn save(file: &Path, r: &FrameReport) -> Result<(), SaveError> {
    let fail = |source| SaveError::Io {
        path: file.to_path_buf(),
        source,
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let text = serde_json::to_string_pretty(r)?;
    let part = part_file(file);
    let done = std::fs::write(&part, text).and_then(|()| std::fs::rename(&part, file));
    if done.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    done.map_err(fail)
}

/// The file `save` writes before it replaces `file`.
#[must_use]
pub fn part_file(file: &Path) -> PathBuf {
    file.with_extension(PART)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tempdir::TempDir;

    /// One frame at 60 Hz.
    fn hz60() -> Duration {
        Duration::from_secs_f64(1.0 / 60.0)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn sixty_even_gaps_make_one_second_at_60_fps() {
        let mut f = Frames::new(1000);
        (0..60).for_each(|_| f.add(hz60()));
        let r = f.report();
        assert_eq!(r.gaps, 60);
        assert!(close(r.secs, 1.0) && close(r.fps, 60.0), "{r:?}");
        assert!(close(r.p99_ms, 16.667) && close(r.max_ms, 16.667), "{r:?}");
    }

    #[test]
    fn one_slow_frame_in_a_hundred_shows_as_the_longest_not_the_p99() {
        let mut f = Frames::new(1000);
        (0..99).for_each(|_| f.add(hz60()));
        f.add(Duration::from_millis(50));
        let r = f.report();
        assert!(
            close(r.max_ms, 50.0) && close(r.p99_ms, 16.667) && close(r.p50_ms, 16.667),
            "{r:?}"
        );
    }

    #[test]
    fn the_first_tick_starts_the_clock_and_each_later_one_adds_its_gap() {
        let mut f = Frames::new(10);
        let t0 = Instant::now();
        f.tick(t0);
        f.tick(t0 + Duration::from_millis(16));
        f.tick(t0 + Duration::from_millis(50));
        let r = f.report();
        assert_eq!(r.gaps, 2);
        assert!(close(r.max_ms, 34.0), "{r:?}");
    }

    #[test]
    fn the_cap_stops_the_record_and_says_so() {
        let mut f = Frames::new(3);
        (0..5).for_each(|_| f.add(hz60()));
        let r = f.report();
        assert_eq!((r.gaps, r.capped), (3, true));
    }

    #[test]
    fn an_empty_record_divides_by_nothing_and_a_huge_gap_is_clamped() {
        let r = Frames::new(10).report();
        assert!(
            r.gaps == 0 && close(r.fps, 0.0) && close(r.max_ms, 0.0),
            "{r:?}"
        );
        let mut f = Frames::new(10);
        f.add(Duration::MAX);
        let clamped = f64::from(u32::MAX) / f64::from(US_PER_MS);
        assert!(close(f.report().max_ms, clamped));
    }

    #[test]
    fn a_saved_report_reads_back_the_same_and_leaves_no_part_file() {
        let dir = TempDir::new("ringstats").expect("makes a folder");
        let file = dir.path().join("frames.json");
        let mut f = Frames::new(10);
        (0..4).for_each(|_| f.add(hz60()));
        save(&file, &f.report()).expect("saves");
        save(&file, &f.report()).expect("replaces");
        let text = std::fs::read_to_string(&file).expect("reads");
        let back: FrameReport = serde_json::from_str(&text).expect("parses");
        assert_eq!(back, f.report());
        assert!(!part_file(&file).exists());
    }

    #[test]
    fn a_failed_replace_leaves_no_part_file() {
        let dir = TempDir::new("ringstats-busy").expect("makes a folder");
        let file = dir.path().join("frames.json");
        std::fs::create_dir_all(file.join("child")).expect("a folder where the file goes");
        let got = save(&file, &Frames::new(1).report());
        assert!(matches!(got, Err(SaveError::Io { .. })), "{got:?}");
        assert!(!part_file(&file).exists());
    }
}

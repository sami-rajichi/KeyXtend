//! `spike.toml [hold]` and `[ring]`: the hold a still press waits for (spec §5.1), and gate G12's ring test.

use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

/// The most parts the stats path may have: one folder and the file, so kx-gates can remove both.
const STATS_PARTS: usize = 2;

/// A still left press: how long it waits, what counts as still, and when the ring shows (spec §3.2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldConfig {
    /// A still press fires after this, in ms.
    pub ms: u64,
    /// The shortest hold the owner may set, in ms.
    pub min_ms: u64,
    /// The longest hold the owner may set, in ms.
    pub max_ms: u64,
    /// Moves up to this many pixels still count as still.
    pub still_px: i32,
    /// The ring shows this long after the press, in ms.
    pub ring_after_ms: u64,
}

impl HoldConfig {
    /// Refuses a hold outside its limits, limits the wrong way round, no still room, or a ring that the shortest hold
    /// would never show.
    pub fn check(&self) -> Result<(), String> {
        if !(self.min_ms..=self.max_ms).contains(&self.ms) {
            return Err(format!(
                "hold: ms must be {} to {}",
                self.min_ms, self.max_ms
            ));
        }
        if self.still_px <= 0 || self.ring_after_ms >= self.min_ms {
            return Err("hold: still_px must be above 0 and ring_after_ms below min_ms".into());
        }
        Ok(())
    }

    /// How long the wave shrinks: the hold less the ring's delay.
    pub fn wave_ms(&self) -> u64 {
        self.ms.saturating_sub(self.ring_after_ms)
    }
}

/// Gate G12's ring test: a loop of holds at the pointer, started from the test strip.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RingTest {
    /// The ring window's title, so kx-gates can find it.
    pub title: String,
    /// The test strip's switch for the loop.
    pub label: String,
    /// The frame stats file, under the user's local app-data folder, where the face writes it and kx-gates reads it.
    pub stats: String,
    /// The most frame gaps kept, so a loop left on never grows without end.
    pub max_frames: usize,
    /// Pause after each burst before the next hold, in ms.
    pub gap_ms: u64,
}

impl RingTest {
    /// Refuses an empty title or label, a stats path that leaves its folder or goes more than one folder deep, and a cap
    /// of no frames.
    pub fn check(&self) -> Result<(), String> {
        let parts: Vec<Component> = Path::new(&self.stats).components().collect();
        let plain = parts.iter().all(|c| matches!(c, Component::Normal(_)));
        let inside = plain && (1..=STATS_PARTS).contains(&parts.len());
        let empty = self.title.is_empty() || self.label.is_empty();
        if empty || !inside || self.max_frames == 0 {
            return Err(
                "ring: title, label and a stats path of at most one folder must be set, and max_frames above 0"
                    .into(),
            );
        }
        Ok(())
    }

    /// The frame stats file.
    pub fn stats_path(&self) -> Result<PathBuf, String> {
        Ok(crate::folders::local_data()?.join(&self.stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> crate::config::SpikeConfig {
        crate::config::load().expect("spike.toml loads")
    }

    #[test]
    fn the_hold_is_the_specs_one_and_a_half_seconds_with_the_ring_after_150_ms() {
        let h = cfg().hold;
        assert_eq!((h.ms, h.min_ms, h.max_ms), (1500, 500, 3000));
        assert_eq!((h.still_px, h.ring_after_ms), (7, 150));
        assert_eq!(h.wave_ms(), 1350);
    }

    #[test]
    fn a_hold_outside_its_limits_or_a_ring_that_never_shows_is_refused() {
        let h = cfg().hold;
        let bad = [
            HoldConfig {
                ms: 400,
                ..h.clone()
            },
            HoldConfig {
                ms: 3100,
                ..h.clone()
            },
            HoldConfig {
                min_ms: 3500,
                ..h.clone()
            },
            HoldConfig {
                still_px: 0,
                ..h.clone()
            },
            HoldConfig {
                ring_after_ms: 1500,
                ..h.clone()
            },
            HoldConfig {
                ring_after_ms: 800,
                ..h.clone()
            },
        ];
        for b in bad {
            assert!(b.check().is_err(), "{b:?}");
        }
        assert_eq!(h.check(), Ok(()));
    }

    #[test]
    fn a_ring_test_without_a_title_label_inner_file_or_frames_is_refused() {
        let r = cfg().ring;
        assert_eq!(r.check(), Ok(()));
        let bad = [
            RingTest {
                title: String::new(),
                ..r.clone()
            },
            RingTest {
                label: String::new(),
                ..r.clone()
            },
            RingTest {
                stats: String::new(),
                ..r.clone()
            },
            RingTest {
                stats: "../frames.json".into(),
                ..r.clone()
            },
            RingTest {
                stats: "C:/frames.json".into(),
                ..r.clone()
            },
            RingTest {
                stats: "a/b/frames.json".into(),
                ..r.clone()
            },
            RingTest {
                max_frames: 0,
                ..r.clone()
            },
        ];
        for b in bad {
            assert!(b.check().is_err(), "{b:?}");
        }
    }
}

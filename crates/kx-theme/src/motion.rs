//! `motion.toml`: every animation's length and curve, from the mock-up's transitions and keyframes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ringmove::{RingFile, RingMotion};
use crate::theme::{bez, outside_unit};

/// Every animation the faces play, by what it moves.
pub const MOVES: [&str; 20] = [
    "press",
    "colour",
    "plate",
    "led",
    "glow",
    "dot",
    "carousel",
    "legends",
    "stop",
    "chips",
    "chips_fade",
    "caption",
    "panel",
    "shimmer",
    "burst",
    "notice",
    "menu",
    "badge",
    "outline",
    "layer",
];
/// Every animation the clipboard panel plays, by what it moves.
pub const CLIP_MOVES: [&str; 4] = ["tick", "highlight", "swap", "turn"];
/// Every animation Quick-fill plays, by what it moves.
pub const FILL_MOVES: [&str; 5] = ["highlight", "swap", "turn", "sheet", "reveal"];
/// Every state that shows for a set time before what comes next, by what it shows.
pub const SHOWS: [&str; 1] = ["chips_done"];
/// The tables in the file that hold the faces' moves, the clipboard panel's, Quick-fill's and the shown times, as
/// errors name them.
const MOVES_TABLE: &str = "moves";
const CLIP_TABLE: &str = "clip";
const FILL_TABLE: &str = "fill";
const SHOWS_TABLE: &str = "shows";

/// The file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionFile {
    curves: BTreeMap<String, [f32; 4]>,
    moves: BTreeMap<String, Move>,
    clip: BTreeMap<String, Move>,
    fill: BTreeMap<String, Move>,
    amount: Amounts,
    shows: BTreeMap<String, u32>,
    ring: RingFile,
}

/// One animation as written: its length and its curve's name.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Move {
    ms: u32,
    curve: String,
}

/// How far the animations go (the mock-up's keyframes).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Amounts {
    /// Opacity halfway through a pulse.
    pub pulse_low: f32,
    /// Brightness factor halfway through a recording cap's glow.
    pub glow_lift: f32,
    /// How far language names slide in, as a share of their width.
    pub carousel_shift: f32,
    /// How far legends rise in, as a share of their height.
    pub rise: f32,
    /// How far down a pop-in starts, in pixels.
    pub pop_rise_px: f32,
    /// A pop-in's starting scale.
    pub pop_scale: f32,
    /// The tray menu's starting scale.
    pub menu_scale: f32,
    /// A clipboard tick's starting scale.
    pub tick_scale: f32,
    /// How far the selection strip rises as it fades, in pixels.
    pub fade_rise_px: f32,
}

/// One animation ready to play: its length and its curve as Qt bezier points (two controls, then the end).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Timed {
    /// Length in ms; 0 means no animation.
    pub ms: u32,
    /// The curve for `Easing.BezierSpline`.
    pub bez: [f32; 6],
}

/// Every animation, ready to play.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Motion {
    /// Each animation by its name in `MOVES`.
    pub moves: BTreeMap<String, Timed>,
    /// The clipboard panel's, by name in `CLIP_MOVES`.
    pub clip: BTreeMap<String, Timed>,
    /// Quick-fill's, by name in `FILL_MOVES`.
    pub fill: BTreeMap<String, Timed>,
    /// How far they go.
    pub amount: Amounts,
    /// How long each state in `SHOWS` shows, in ms, the same with reduced motion.
    pub shows: BTreeMap<String, u32>,
    /// The hold ring, the same with reduced motion since its waves show the hold's time.
    pub ring: RingMotion,
}

/// The animations as timed, and the same with reduced motion.
#[derive(Debug, Clone, PartialEq)]
pub struct Motions {
    /// As the mock-up times them.
    pub full: Motion,
    /// Every length 0, the curves kept; the ring's waves and `[shows]` keep their times.
    pub reduced: Motion,
}

impl Motions {
    /// The reduced set when `reduced`, else the full one.
    #[must_use]
    pub fn pick(&self, reduced: bool) -> &Motion {
        if reduced { &self.reduced } else { &self.full }
    }
}

/// The checked animations in `text`, full and reduced.
pub(crate) fn parse(text: &str) -> Result<Motions, String> {
    let f: MotionFile = toml::from_str(text).map_err(|e| e.to_string())?;
    check_names(MOVES_TABLE, &f.moves, &MOVES)?;
    check_names(CLIP_TABLE, &f.clip, &CLIP_MOVES)?;
    check_names(FILL_TABLE, &f.fill, &FILL_MOVES)?;
    check_names(SHOWS_TABLE, &f.shows, &SHOWS)?;
    for (name, c) in &f.curves {
        check_curve(name, c)?;
    }
    f.amount.check()?;
    check_shows(&f.shows)?;
    let ring = f.ring.ready(&f.curves)?;
    let (moves, clip, quick) = (
        all_timed(MOVES_TABLE, &f.moves, &f.curves)?,
        all_timed(CLIP_TABLE, &f.clip, &f.curves)?,
        all_timed(FILL_TABLE, &f.fill, &f.curves)?,
    );
    let reduced = Motion {
        moves: still(&moves),
        clip: still(&clip),
        fill: still(&quick),
        amount: f.amount,
        shows: f.shows.clone(),
        ring,
    };
    let full = Motion {
        moves,
        clip,
        fill: quick,
        amount: f.amount,
        shows: f.shows,
        ring,
    };
    Ok(Motions { full, reduced })
}

/// Refuses a shown time of 0, as the state would never show.
fn check_shows(shows: &BTreeMap<String, u32>) -> Result<(), String> {
    match shows.iter().find(|(_, ms)| **ms == 0) {
        Some((n, _)) => Err(format!(
            "{SHOWS_TABLE}.{n} must be above 0 ms, or it never shows"
        )),
        None => Ok(()),
    }
}

/// Each of `moves`, from table `table`, with its curve from `curves`.
fn all_timed(
    table: &str,
    moves: &BTreeMap<String, Move>,
    curves: &BTreeMap<String, [f32; 4]>,
) -> Result<BTreeMap<String, Timed>, String> {
    let ready = |(name, m): (&String, &Move)| Ok((name.clone(), timed(table, name, m, curves)?));
    moves.iter().map(ready).collect()
}

/// `moves` with every length 0, the curves kept.
fn still(moves: &BTreeMap<String, Timed>) -> BTreeMap<String, Timed> {
    let none = |(n, t): (&String, &Timed)| (n.clone(), Timed { ms: 0, ..*t });
    moves.iter().map(none).collect()
}

/// Refuses an entry of table `table` missing from `names`, and one not in it.
fn check_names<V>(table: &str, moves: &BTreeMap<String, V>, names: &[&str]) -> Result<(), String> {
    if let Some(n) = names.iter().find(|n| !moves.contains_key(**n)) {
        return Err(format!("{table}.{n} is missing"));
    }
    match moves.keys().find(|n| !names.contains(&n.as_str())) {
        Some(n) => Err(format!("{table}.{n} is not one the faces use")),
        None => Ok(()),
    }
}

/// Refuses x points outside 0 to 1, since time would run backwards, and y points that are not numbers.
fn check_curve(name: &str, [x1, y1, x2, y2]: &[f32; 4]) -> Result<(), String> {
    let time = |x: &f32| (0.0..=1.0).contains(x);
    if !(time(x1) && time(x2)) {
        return Err(format!("curves.{name}: x1 and x2 must be 0 to 1"));
    }
    if !(y1.is_finite() && y2.is_finite()) {
        return Err(format!("curves.{name}: y1 and y2 must be numbers"));
    }
    Ok(())
}

/// Move `m`, named `table.name`, with its curve from `curves`.
fn timed(
    table: &str,
    name: &str,
    m: &Move,
    curves: &BTreeMap<String, [f32; 4]>,
) -> Result<Timed, String> {
    let no_curve = || format!("{table}.{name}: no curve {:?}", m.curve);
    Ok(Timed {
        ms: m.ms,
        bez: bez(curves, &m.curve).ok_or_else(no_curve)?,
    })
}

impl Amounts {
    /// Refuses a share or scale outside 0 to 1, a glow that darkens and a negative rise; NaN fails each.
    fn check(&self) -> Result<(), String> {
        let shares = [
            ("pulse_low", self.pulse_low),
            ("carousel_shift", self.carousel_shift),
            ("rise", self.rise),
            ("pop_scale", self.pop_scale),
            ("menu_scale", self.menu_scale),
            ("tick_scale", self.tick_scale),
        ];
        if let Some((n, _)) = outside_unit(&shares) {
            return Err(format!("amount.{n} must be 0 to 1"));
        }
        if !(self.glow_lift.is_finite() && self.glow_lift >= 1.0) {
            return Err("amount.glow_lift must be 1 or more".into());
        }
        let rises = [
            ("pop_rise_px", self.pop_rise_px),
            ("fade_rise_px", self.fade_rise_px),
        ];
        if let Some((n, _)) = rises.iter().find(|(_, v)| !(v.is_finite() && *v >= 0.0)) {
            return Err(format!("amount.{n} must be 0 or more"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

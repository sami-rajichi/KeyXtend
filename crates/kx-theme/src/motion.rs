//! `motion.toml`: every animation's length and curve, from the mock-up's transitions and keyframes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ringmove::{RingFile, RingMotion};
use crate::theme::{bez, outside_unit};

/// Every animation the faces play, by what it moves.
pub const MOVES: [&str; 17] = [
    "press", "colour", "plate", "led", "glow", "dot", "carousel", "legends", "stop", "pill",
    "caption", "panel", "shimmer", "burst", "notice", "menu", "badge",
];

/// The file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionFile {
    curves: BTreeMap<String, [f32; 4]>,
    moves: BTreeMap<String, Move>,
    amount: Amounts,
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
    /// How far they go.
    pub amount: Amounts,
    /// The hold ring, the same with reduced motion since its waves show the hold's time.
    pub ring: RingMotion,
}

/// The animations as timed, and the same with reduced motion.
#[derive(Debug, Clone, PartialEq)]
pub struct Motions {
    /// As the mock-up times them.
    pub full: Motion,
    /// Every length 0, the curves kept.
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
    check_names(&f.moves)?;
    for (name, c) in &f.curves {
        check_curve(name, c)?;
    }
    f.amount.check()?;
    let ring = f.ring.ready(&f.curves)?;
    let ready = |(name, m): (&String, &Move)| Ok((name.clone(), timed(name, m, &f.curves)?));
    let moves = f
        .moves
        .iter()
        .map(ready)
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let still = |(n, t): (&String, &Timed)| (n.clone(), Timed { ms: 0, ..*t });
    let reduced = Motion {
        moves: moves.iter().map(still).collect(),
        amount: f.amount,
        ring,
    };
    let full = Motion {
        moves,
        amount: f.amount,
        ring,
    };
    Ok(Motions { full, reduced })
}

/// Refuses a missing move, and one no face plays.
fn check_names(moves: &BTreeMap<String, Move>) -> Result<(), String> {
    if let Some(n) = MOVES.iter().find(|n| !moves.contains_key(**n)) {
        return Err(format!("moves.{n} is missing"));
    }
    match moves.keys().find(|n| !MOVES.contains(&n.as_str())) {
        Some(n) => Err(format!("moves.{n} is not an animation the faces play")),
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

/// Move `m` with its curve from `curves`.
fn timed(name: &str, m: &Move, curves: &BTreeMap<String, [f32; 4]>) -> Result<Timed, String> {
    let no_curve = || format!("moves.{name}: no curve {:?}", m.curve);
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
        ];
        if let Some((n, _)) = outside_unit(&shares) {
            return Err(format!("amount.{n} must be 0 to 1"));
        }
        if !(self.glow_lift.is_finite() && self.glow_lift >= 1.0) {
            return Err("amount.glow_lift must be 1 or more".into());
        }
        if !(self.pop_rise_px.is_finite() && self.pop_rise_px >= 0.0) {
            return Err("amount.pop_rise_px must be 0 or more".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

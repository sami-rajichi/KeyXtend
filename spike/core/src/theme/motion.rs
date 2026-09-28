//! `motion.toml`: every animation's length and curve, from the mock-up's transitions and keyframes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::read_settings;

/// The motion file, beside `spike.toml`.
const FILE: &str = "motion.toml";
/// Every animation the faces play, by what it moves.
pub const MOVES: [&str; 13] = [
    "press", "colour", "plate", "led", "glow", "dot", "carousel", "legends", "stop", "pill",
    "caption", "panel", "shimmer",
];

/// The file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionFile {
    curves: BTreeMap<String, [f32; 4]>,
    moves: BTreeMap<String, Move>,
    amount: Amounts,
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
    pub fn pick(&self, reduced: bool) -> &Motion {
        if reduced { &self.reduced } else { &self.full }
    }
}

/// Loads and checks `motion.toml` from beside the exe or the spike folder.
pub fn load() -> Result<Motions, String> {
    read_settings(FILE, parse)
}

/// The checked animations in `text`, full and reduced.
pub fn parse(text: &str) -> Result<Motions, String> {
    let f: MotionFile = toml::from_str(text).map_err(|e| e.to_string())?;
    check_names(&f.moves)?;
    for (name, c) in &f.curves {
        check_curve(name, c)?;
    }
    f.amount.check()?;
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
    };
    let full = Motion {
        moves,
        amount: f.amount,
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

/// Move `m` with its curve from `curves`, ending at (1, 1) as Qt wants.
fn timed(name: &str, m: &Move, curves: &BTreeMap<String, [f32; 4]>) -> Result<Timed, String> {
    let no_curve = || format!("moves.{name}: no curve {:?}", m.curve);
    let [x1, y1, x2, y2] = *curves.get(&m.curve).ok_or_else(no_curve)?;
    Ok(Timed {
        ms: m.ms,
        bez: [x1, y1, x2, y2, 1.0, 1.0],
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
        ];
        if let Some((n, _)) = shares.iter().find(|(_, v)| !(0.0..=1.0).contains(v)) {
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
mod tests {
    use super::*;

    fn text() -> String {
        std::fs::read_to_string(crate::config::settings_path(FILE)).expect("motion.toml reads")
    }

    #[test]
    fn the_mock_up_timings_and_curves_load() {
        let m = load().expect("motion.toml loads").full;
        let names: Vec<&str> = m.moves.keys().map(String::as_str).collect();
        let mut all = MOVES.to_vec();
        all.sort_unstable();
        assert_eq!(names, all);
        let ease = [0.25, 0.1, 0.25, 1.0, 1.0, 1.0];
        assert_eq!(
            m.moves["press"],
            Timed { ms: 90, bez: ease },
            ".cap transform .09s ease"
        );
        let spring = [0.3, 1.5, 0.5, 1.0, 1.0, 1.0];
        assert_eq!(
            m.moves["stop"],
            Timed {
                ms: 220,
                bez: spring
            },
            ".dp-stop"
        );
        assert_eq!(m.amount.pulse_low, 0.35, "@keyframes pulse");
    }

    #[test]
    fn reduced_motion_keeps_the_curves_but_takes_no_time() {
        let m = load().expect("motion.toml loads");
        assert!(std::ptr::eq(m.pick(true), &m.reduced));
        for (name, t) in &m.reduced.moves {
            assert_eq!(t.ms, 0, "{name}");
            assert_eq!(t.bez, m.full.moves[name].bez, "{name}");
        }
        assert_eq!(m.reduced.amount, m.full.amount);
    }

    #[test]
    fn a_move_with_an_unknown_curve_is_refused_by_name() {
        let typo = text().replacen("curve = \"spring\"", "curve = \"sprung\"", 1);
        let e = parse(&typo).expect_err("no such curve");
        assert!(e.contains("stop") && e.contains("sprung"), "{e}");
    }

    #[test]
    fn a_missing_or_unknown_move_is_refused() {
        let all = text();
        let gone = all.replacen("shimmer = {", "shine = {", 1);
        let e = parse(&gone).expect_err("shimmer is missing");
        assert!(e.contains("shimmer"), "{e}");
        let extra = all.replacen(
            "[amount]",
            "wobble = { ms = 5, curve = \"ease\" }\n\n[amount]",
            1,
        );
        let e = parse(&extra).expect_err("wobble is unknown");
        assert!(e.contains("wobble"), "{e}");
    }

    #[test]
    fn a_curve_that_runs_back_in_time_or_is_not_a_number_is_refused() {
        let all = text();
        for (bad, says) in [
            ("linear = [1.2, 0.0, 1.0, 1.0]", "x1 and x2"),
            ("linear = [0.0, 0.0, -0.1, 1.0]", "x1 and x2"),
            ("linear = [0.0, inf, 1.0, 1.0]", "y1 and y2"),
        ] {
            let e = parse(&all.replacen("linear = [0.0, 0.0, 1.0, 1.0]", bad, 1)).expect_err(bad);
            assert!(e.contains("linear") && e.contains(says), "{e}");
        }
    }

    #[test]
    fn an_amount_outside_its_range_is_refused() {
        let all = text();
        for (from, to) in [
            ("pulse_low = 0.35", "pulse_low = 1.5"),
            ("pop_scale = 0.98", "pop_scale = -1.0"),
            ("glow_lift = 1.12", "glow_lift = 0.9"),
            ("pop_rise_px = 6.0", "pop_rise_px = -1.0"),
        ] {
            let bad = all.replacen(from, to, 1);
            let e = parse(&bad).expect_err(to);
            assert!(e.contains(from.split(' ').next().unwrap_or(from)), "{e}");
        }
    }
}

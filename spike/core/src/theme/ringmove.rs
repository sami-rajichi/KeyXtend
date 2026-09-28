//! `motion.toml [ring]`: how the hold ring moves (mock-up `.ring`, `.burst`); its waves last the hold, not a set time.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::extras::RingShape;
use super::{bez, outside_unit};

/// The ring as written: its curves by name, and how far it goes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RingFile {
    wave_curve: String,
    core_curve: String,
    amount: RingAmounts,
}

/// How far the ring goes (@keyframes shrink, core, burst and fly).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RingAmounts {
    /// A wave's opacity at its start, at its turn and at its end.
    pub wave_fade: [f32; 3],
    /// The turn's share of a wave's time.
    pub wave_turn: f32,
    /// A wave's scale at its end.
    pub wave_end: f32,
    /// The second wave's share of the time; it starts once the rest has passed.
    pub wave2_share: f32,
    /// The core's starting scale.
    pub core_from: f32,
    /// The core's opacity at the end.
    pub core_to: f32,
    /// The burst ring's scale at its start and end.
    pub burst_scale: [f32; 2],
    /// The burst dots' scale at the end.
    pub dot_end: f32,
}

/// The ring ready to play: its curves as Qt bezier points, and its amounts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RingMotion {
    /// The waves' curve: once over a wave for its scale, and on each side of the turn for its opacity, as CSS
    /// eases each keyframe step.
    pub wave: [f32; 6],
    /// The core's curve.
    pub core: [f32; 6],
    /// How far it goes.
    pub amount: RingAmounts,
}

impl RingFile {
    /// The checked ring, with its curves taken from `curves`.
    pub(super) fn ready(&self, curves: &BTreeMap<String, [f32; 4]>) -> Result<RingMotion, String> {
        let pick = |field: &str, name: &String| {
            bez(curves, name).ok_or_else(|| format!("ring.{field}: no curve {name:?}"))
        };
        self.amount.check()?;
        Ok(RingMotion {
            wave: pick("wave_curve", &self.wave_curve)?,
            core: pick("core_curve", &self.core_curve)?,
            amount: self.amount,
        })
    }
}

impl RingAmounts {
    /// Refuses a share, opacity or scale outside 0 to 1, and a burst that does not grow; NaN fails each.
    fn check(&self) -> Result<(), String> {
        let [f0, f1, f2] = self.wave_fade;
        let shares = [
            ("wave_fade", f0),
            ("wave_fade", f1),
            ("wave_fade", f2),
            ("wave_turn", self.wave_turn),
            ("wave_end", self.wave_end),
            ("wave2_share", self.wave2_share),
            ("core_from", self.core_from),
            ("core_to", self.core_to),
            ("burst_scale", self.burst_scale[0]),
            ("dot_end", self.dot_end),
        ];
        if let Some((n, _)) = outside_unit(&shares) {
            return Err(format!("ring.amount.{n} must be 0 to 1"));
        }
        let [from, to] = self.burst_scale;
        if !(to.is_finite() && to > from) {
            return Err("ring.amount.burst_scale must grow".into());
        }
        Ok(())
    }
}

/// Refuses a burst ring or dots that would reach past the ring's window, `shape.toml [ring] box_px`.
pub(super) fn fits(s: &RingShape, a: &RingAmounts) -> Result<(), String> {
    // Written as "fits", so a size that is not a number fails too.
    let burst_fits = s.burst_px[0] * a.burst_scale[1] <= s.box_px;
    if !burst_fits {
        return Err(
            "shape.toml ring.burst_px, grown by motion.toml burst_scale, must fit in ring.box_px"
                .into(),
        );
    }
    let dots_fit = s.fly_px[1] + s.dot_px / 2.0 <= s.box_px / 2.0;
    if !dots_fit {
        return Err("shape.toml ring.fly_px and ring.dot_px must fit in ring.box_px".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{motion, shape};

    fn text() -> String {
        std::fs::read_to_string(crate::config::settings_path(motion::FILE))
            .expect("motion.toml reads")
    }

    #[test]
    fn the_mock_up_ring_loads_and_keeps_its_curves_with_reduced_motion() {
        let m = motion::load().expect("motion.toml loads");
        let (r, a) = (m.full.ring, m.full.ring.amount);
        assert_eq!(
            r.wave,
            [0.45, 0.0, 0.55, 1.0, 1.0, 1.0],
            ".ring.run .w1, .w2"
        );
        assert_eq!(r.core, [0.42, 0.0, 1.0, 1.0, 1.0, 1.0], "ease-in");
        assert_eq!(
            (a.wave_fade, a.wave_turn),
            ([0.15, 0.85, 1.0], 0.55),
            "@keyframes shrink"
        );
        assert_eq!((a.wave_end, a.wave2_share), (0.13, 0.8));
        assert_eq!((a.core_from, a.core_to), (0.4, 0.9), "@keyframes core");
        assert_eq!(
            (a.burst_scale, a.dot_end),
            ([0.15, 1.25], 0.3),
            "@keyframes burst, fly"
        );
        assert_eq!(m.reduced.ring, r, "the waves keep the hold time");
        let burst = m.full.moves["burst"];
        assert_eq!(
            (burst.ms, burst.bez),
            (420, [0.0, 0.0, 0.58, 1.0, 1.0, 1.0]),
            ".burst .42s ease-out"
        );
    }

    #[test]
    fn the_ring_takes_the_mock_up_sizes_and_its_burst_fits() {
        let (s, m) = (
            shape::load().expect("shape"),
            motion::load().expect("motion"),
        );
        let r = &s.ring;
        assert_eq!(
            (r.box_px, r.wave_px, r.core_share),
            (76.0, [2.5, 1.5], 0.1),
            ".ring, .w, .core"
        );
        assert_eq!(
            (r.burst_px, r.dot_px, r.dots),
            ([60.0, 2.0], 6.0, 6),
            ".b-ring, .b-dot"
        );
        assert_eq!(r.fly_px, [6.0, 30.0], "@keyframes fly");
        assert_eq!(fits(r, &m.full.ring.amount), Ok(()));
    }

    #[test]
    fn a_burst_or_dots_past_the_ring_window_are_refused() {
        let (s, m) = (
            shape::load().expect("shape"),
            motion::load().expect("motion"),
        );
        let a = m.full.ring.amount;
        let wide = RingShape {
            burst_px: [70.0, 2.0],
            ..s.ring.clone()
        };
        assert!(fits(&wide, &a).expect_err("too wide").contains("burst_px"));
        let far = RingShape {
            fly_px: [6.0, 40.0],
            ..s.ring.clone()
        };
        assert!(fits(&far, &a).expect_err("too far").contains("fly_px"));
        let nan = RingShape {
            box_px: f32::NAN,
            ..s.ring.clone()
        };
        assert!(fits(&nan, &a).is_err());
    }

    #[test]
    fn a_ring_amount_outside_its_range_or_an_unknown_curve_is_refused() {
        let all = text();
        for (from, to, says) in [
            ("wave_end = 0.13", "wave_end = 1.5", "wave_end"),
            ("dot_end = 0.3", "dot_end = nan", "dot_end"),
            (
                "burst_scale = [0.15, 1.25]",
                "burst_scale = [0.15, 0.1]",
                "burst_scale",
            ),
            (
                "burst_scale = [0.15, 1.25]",
                "burst_scale = [0.15, inf]",
                "burst_scale",
            ),
            (
                "wave_curve = \"wave\"",
                "wave_curve = \"wavy\"",
                "wave_curve",
            ),
        ] {
            let e = motion::parse(&all.replacen(from, to, 1)).expect_err(to);
            assert!(e.contains(says), "{e}");
        }
    }

    #[test]
    fn a_ring_core_share_outside_0_to_1_is_refused() {
        let file = crate::config::settings_path(shape::FILE);
        let t = std::fs::read_to_string(file).expect("shape.toml reads");
        let bad = t.replacen("core_share = 0.1", "core_share = 1.5", 1);
        assert!(
            shape::parse(&bad)
                .expect_err("1.5")
                .contains("ring.core_share")
        );
    }
}

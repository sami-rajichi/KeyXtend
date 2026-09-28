//! Keyboard size: −/+ steps, S/M/L presets, and a click-move-click resize, all within `spike.toml [size]`.

use serde::Deserialize;

/// `spike.toml [size]`: sizes are factors of the size-1 keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizeConfig {
    /// S, M and L; the keyboard starts at the first.
    pub presets: Vec<f32>,
    /// Change per − or + click.
    pub step: f32,
    /// Smallest size.
    pub min: f32,
    /// Largest size.
    pub max: f32,
    /// How often the pointer is read while moving or resizing, in ms.
    pub poll_ms: u64,
}

impl SizeConfig {
    /// Refuses a table whose limits, step or presets make no sense.
    pub fn check(&self) -> Result<(), String> {
        if !(self.step > 0.0 && self.min > 0.0 && self.min <= self.max) || self.poll_ms == 0 {
            return Err("size: step, min and poll_ms must be above 0, and min at most max".into());
        }
        let off = |p: &f32| !(self.min..=self.max).contains(p);
        if self.presets.is_empty() || self.presets.iter().any(off) {
            return Err("size: presets must sit between min and max".into());
        }
        Ok(())
    }

    /// The size the keyboard starts at: the first preset.
    pub fn start(&self) -> f32 {
        self.presets.first().copied().unwrap_or(self.min)
    }
}

/// `v` rounded to a whole number of steps, within the limits.
fn snap(v: f32, c: &SizeConfig) -> f32 {
    ((v / c.step).round() * c.step).clamp(c.min, c.max)
}

/// One step larger, stopping at the largest size.
pub fn step_up(size: f32, c: &SizeConfig) -> f32 {
    snap(size + c.step, c)
}

/// One step smaller, stopping at the smallest size.
pub fn step_down(size: f32, c: &SizeConfig) -> f32 {
    snap(size - c.step, c)
}

/// A resize in progress: the first click on the corner handle starts it, the second ends it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resize {
    start: f32,
    anchor: (f32, f32),
    plate: (f32, f32),
}

impl Resize {
    /// Starts at `size`, with the pointer at `anchor` and the plate `plate` wide and high, in the same pixels.
    pub fn begin(size: f32, anchor: (f32, f32), plate: (f32, f32)) -> Resize {
        Resize {
            start: size,
            anchor,
            plate,
        }
    }

    /// The size for pointer `p`: the axis that moved further, relative to the plate, sets the growth.
    pub fn at(&self, p: (f32, f32), c: &SizeConfig) -> f32 {
        let rx = (p.0 - self.anchor.0) / self.plate.0.max(1.0);
        let ry = (p.1 - self.anchor.1) / self.plate.1.max(1.0);
        let grow = if rx.abs() >= ry.abs() { rx } else { ry };
        snap(self.start * (1.0 + grow), c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allowed rounding error on a size.
    const EPS: f32 = 1e-4;

    fn cfg() -> SizeConfig {
        crate::config::load().expect("spike.toml loads").size
    }

    #[test]
    fn steps_move_by_one_step_and_stop_at_the_limits() {
        let c = cfg();
        assert!((step_up(1.0, &c) - 1.05).abs() < EPS);
        assert!((step_down(1.0, &c) - 0.95).abs() < EPS);
        assert!(
            (step_up(c.max, &c) - c.max).abs() < EPS,
            "+ at the top does nothing"
        );
        assert!(
            (step_down(c.min, &c) - c.min).abs() < EPS,
            "− at the bottom does nothing"
        );
        let mut s = c.min;
        for _ in 0..100 {
            s = step_up(s, &c);
        }
        assert!((s - c.max).abs() < EPS, "no drift after many steps: {s}");
    }

    #[test]
    fn presets_are_s_m_and_l() {
        assert_eq!(cfg().presets, [1.0, 1.25, 1.5]);
    }

    #[test]
    fn a_resize_follows_the_pointer_in_steps_and_stops_at_the_limits() {
        let c = cfg();
        let r = Resize::begin(1.0, (100.0, 100.0), (600.0, 200.0));
        assert!(
            (r.at((400.0, 100.0), &c) - 1.5).abs() < EPS,
            "half as wide again"
        );
        assert!(
            (r.at((100.0, 150.0), &c) - 1.25).abs() < EPS,
            "down grows too"
        );
        assert!((r.at((5000.0, 100.0), &c) - c.max).abs() < EPS);
        assert!((r.at((-5000.0, 100.0), &c) - c.min).abs() < EPS);
        assert!(
            (r.at((103.0, 100.0), &c) - 1.0).abs() < EPS,
            "snapped to a step"
        );
    }

    #[test]
    fn a_bad_size_table_is_refused() {
        let mut c = cfg();
        assert_eq!(c.check(), Ok(()));
        c.min = 2.0;
        assert!(c.check().is_err(), "min above max");
        let mut c = cfg();
        c.presets.push(3.0);
        assert!(c.check().is_err(), "a preset past the limits");
        let mut c = cfg();
        c.poll_ms = 0;
        assert!(
            c.check().is_err(),
            "the resize would never follow the pointer"
        );
    }
}

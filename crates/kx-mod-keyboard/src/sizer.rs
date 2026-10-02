//! The keyboard's size: smaller and bigger steps, the S, M and L presets and a click-move-click resize,
//! all within the `[keyboard]` limits. Each finished change is saved through the module's own settings handle.

use crate::settings::KeyboardSettings;
use kx_module_api::ports::Point;
use kx_module_api::{SettingsHandle, WriteError};
use thiserror::Error;

/// The settings key the size is saved under.
const SIZE_KEY: &str = "size";

/// The size rules from `[keyboard]`: the presets, the step and the limits, and the room on the keyboard's screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Limits {
    presets: Vec<f64>,
    step: f64,
    min: f64,
    max: f64,
    /// The largest size that fits the keyboard's screen; it wins over `min`, so the window never spills past it.
    room: f64,
}

impl Limits {
    /// The rules in `s`, with no screen known yet.
    #[must_use]
    pub fn of(s: &KeyboardSettings) -> Self {
        Self {
            presets: s.presets.clone(),
            step: s.size_step,
            min: s.size_min,
            max: s.size_max,
            room: f64::INFINITY,
        }
    }

    /// The S, M and L sizes.
    #[must_use]
    pub fn presets(&self) -> &[f64] {
        &self.presets
    }

    /// The size of preset `index`, if there is one.
    #[must_use]
    pub fn preset(&self, index: usize) -> Option<f64> {
        self.presets.get(index).copied()
    }

    /// The preset that `size` is, if any.
    #[must_use]
    pub fn preset_of(&self, size: f64) -> Option<usize> {
        self.presets.iter().position(|&p| !differs(p, size))
    }

    /// `v` inside the limits and the room; not-a-number gives the smallest size, and it never panics, unlike
    /// `f64::clamp`.
    #[must_use]
    pub fn clamp(&self, v: f64) -> f64 {
        let top = self.max.min(self.room);
        let low = self.min.min(top);
        if v.is_nan() {
            return low;
        }
        v.min(top).max(low)
    }

    /// `v` rounded to a whole number of steps, inside the limits.
    /// Dividing by the steps per unit keeps sizes such as 1.05 exact in the settings file.
    #[must_use]
    pub fn snap(&self, v: f64) -> f64 {
        let per_unit = self.step.recip();
        self.clamp((v * per_unit).round() / per_unit)
    }

    /// One step bigger than `size`, stopping at the largest size.
    #[must_use]
    pub fn up(&self, size: f64) -> f64 {
        self.snap(size + self.step)
    }

    /// One step smaller than `size`, stopping at the smallest size.
    #[must_use]
    pub fn down(&self, size: f64) -> f64 {
        self.snap(size - self.step)
    }
}

/// A resize in progress: the first click on the corner starts it, the second ends it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Resize {
    start: f64,
    anchor: Point,
    plate: (f64, f64),
}

impl Resize {
    /// Starts at `size`, with the pointer at `anchor` and the plate `plate` wide and high, in the same pixels.
    #[must_use]
    pub fn begin(size: f64, anchor: Point, plate: (f64, f64)) -> Self {
        Self {
            start: size,
            anchor,
            plate,
        }
    }

    /// The size the resize started at.
    #[must_use]
    pub fn start(&self) -> f64 {
        self.start
    }

    /// The size for pointer `p`: the axis that moved further, relative to the plate, sets the growth.
    #[must_use]
    pub fn at(&self, p: Point, limits: &Limits) -> f64 {
        let moved =
            |to: i32, from: i32, plate: f64| (f64::from(to) - f64::from(from)) / plate.max(1.0);
        let rx = moved(p.x, self.anchor.x, self.plate.0);
        let ry = moved(p.y, self.anchor.y, self.plate.1);
        let grow = if rx.abs() >= ry.abs() { rx } else { ry };
        limits.snap(self.start * (1.0 + grow))
    }
}

/// Why a size change failed.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SizeError {
    /// There is no preset at that index.
    #[error("there is no size preset {0}")]
    NoPreset(usize),
    /// The new size holds for this run, but saving it failed.
    #[error(transparent)]
    Unsaved(#[from] WriteError),
}

/// The keyboard's size, saved through the module's own settings handle after each finished change.
#[derive(Debug)]
pub struct Sizer {
    limits: Limits,
    size: f64,
    saved: f64,
    resize: Option<Resize>,
    store: SettingsHandle,
}

impl Sizer {
    /// The size saved in `settings`, which `store` saves again after each change.
    #[must_use]
    pub fn new(settings: &KeyboardSettings, store: SettingsHandle) -> Self {
        let limits = Limits::of(settings);
        Self {
            size: limits.clamp(settings.size),
            saved: settings.size,
            limits,
            resize: None,
            store,
        }
    }

    /// The size now.
    #[must_use]
    pub fn size(&self) -> f64 {
        self.size
    }

    /// The rules the size follows.
    #[must_use]
    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// The preset the size is now, if any.
    #[must_use]
    pub fn preset(&self) -> Option<usize> {
        self.limits.preset_of(self.size)
    }

    /// Caps the size at `room`, the largest that fits the keyboard's screen, saving nothing; true when it changed.
    pub fn fit(&mut self, room: f64) -> bool {
        self.limits.room = room;
        let size = self.limits.clamp(self.size);
        let changed = differs(size, self.size);
        self.size = size;
        changed
    }

    /// One step smaller; true when the size changed.
    ///
    /// # Errors
    /// `Unsaved` when the new size could not be saved; it holds for this run.
    pub fn smaller(&mut self) -> Result<bool, SizeError> {
        self.set(self.limits.down(self.size))
    }

    /// One step bigger; true when the size changed.
    ///
    /// # Errors
    /// `Unsaved` when the new size could not be saved; it holds for this run.
    pub fn bigger(&mut self) -> Result<bool, SizeError> {
        self.set(self.limits.up(self.size))
    }

    /// Preset `index`, as big as the room allows; true when the size changed.
    ///
    /// # Errors
    /// `NoPreset` for an unknown index, or `Unsaved` when the new size could not be saved.
    pub fn pick(&mut self, index: usize) -> Result<bool, SizeError> {
        let size = self
            .limits
            .preset(index)
            .ok_or(SizeError::NoPreset(index))?;
        self.set(self.limits.clamp(size))
    }

    /// Starts a resize with the pointer at `at` and the plate `plate` big, in the pointer's pixels.
    pub fn begin(&mut self, at: Point, plate: (f64, f64)) {
        self.resize = Some(Resize::begin(self.size, at, plate));
    }

    /// Follows the pointer at `at` without saving; true when the size changed.
    pub fn follow(&mut self, at: Point) -> bool {
        let Some(r) = self.resize else {
            return false;
        };
        let size = r.at(at, &self.limits);
        let changed = differs(size, self.size);
        self.size = size;
        changed
    }

    /// Ends the resize at the size it reached and saves it; true when that differs from where it began.
    ///
    /// # Errors
    /// `Unsaved` when the size could not be saved; it holds for this run.
    pub fn end(&mut self) -> Result<bool, SizeError> {
        let Some(r) = self.resize.take() else {
            return Ok(false);
        };
        self.save()?;
        Ok(differs(self.size, r.start()))
    }

    /// True while a resize runs.
    #[must_use]
    pub fn resizing(&self) -> bool {
        self.resize.is_some()
    }

    /// Sets the size and saves it when it changed, so a step that a fitted size stops saves nothing; true when it
    /// changed.
    fn set(&mut self, size: f64) -> Result<bool, SizeError> {
        let changed = differs(size, self.size);
        self.size = size;
        if changed {
            self.save()?;
        }
        Ok(changed)
    }

    /// Writes the size to the settings when it differs from the size they hold, which an unsaved write still sets.
    fn save(&mut self) -> Result<(), SizeError> {
        if differs(self.size, self.saved) {
            let wrote = self.store.set(SIZE_KEY, toml::Value::Float(self.size));
            if wrote.as_ref().err().is_none_or(WriteError::holds) {
                self.saved = self.size;
            }
            wrote?;
        }
        Ok(())
    }
}

/// True when two sizes differ by more than a rounding error.
fn differs(a: f64, b: f64) -> bool {
    (a - b).abs() > f64::EPSILON
}

#[cfg(test)]
mod tests;

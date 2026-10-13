//! The checks a recipe file must pass: every value in range, and every frequency below Nyquist.

use std::collections::HashSet;

use super::{Book, CENTS_PER_OCTAVE, Glide, Layer, Modes, NAME_SEP, Noise, Recipe};
use crate::sounds::{SoundError, wav};

/// `Err(why)` unless `ok`.
fn check(ok: bool, why: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(why.into()) }
}

/// True if `x` is finite and above zero.
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

/// True if `x` is finite and zero or more.
fn non_negative(x: f64) -> bool {
    x.is_finite() && x >= 0.0
}

impl Book {
    /// Checks every value against the others and the writer's abilities.
    pub(super) fn validate(&self) -> Result<(), SoundError> {
        self.check_settings().map_err(SoundError::Recipe)?;
        check(!self.recipes.is_empty(), "the file has no recipe").map_err(SoundError::Recipe)?;
        let mut names = HashSet::new();
        for recipe in &self.recipes {
            let name = &recipe.name;
            check(
                names.insert(name.as_str()),
                format!("{name:?} is used twice"),
            )
            .and_then(|()| recipe.check(self))
            .map_err(|why| SoundError::Recipe(format!("{name}: {why}")))?;
        }
        Ok(())
    }

    /// Checks `[format]`, `[variants]` and `[resample]`.
    fn check_settings(&self) -> Result<(), String> {
        let (format, jitter, setup) = (&self.format, &self.variants, &self.resample);
        let made = |what: &str, value: u16| format!("[format] {what} must be {value}, as written");
        check(format.rate > 0, "[format] rate must be above 0")?;
        check(format.bits == wav::BITS, made("bits", wav::BITS))?;
        check(
            format.channels == wav::CHANNELS,
            made("channels", wav::CHANNELS),
        )?;
        check(
            non_negative(-format.ceiling_db),
            "[format] ceiling_db must be 0 or less",
        )?;
        check(
            positive(-format.trim_db),
            "[format] trim_db must be below 0",
        )?;
        check(positive(format.max_ms), "[format] max_ms must be above 0")?;
        check(
            non_negative(format.fade_ms) && format.fade_ms <= format.max_ms,
            "[format] fade_ms must be from 0 to max_ms",
        )?;
        check(
            non_negative(jitter.pitch_cents) && non_negative(jitter.level_db),
            "[variants] values must be 0 or more",
        )?;
        check(
            setup.half_taps > 0 && positive(setup.rolloff) && setup.rolloff <= 1.0,
            "[resample] needs half_taps above 0 and rolloff above 0, up to 1",
        )
    }
}

impl Recipe {
    /// Checks the recipe's own values, then each layer's.
    fn check(&self, book: &Book) -> Result<(), String> {
        let plain = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == NAME_SEP;
        check(
            !self.name.is_empty() && self.name.chars().all(plain),
            format!("{:?} must hold only a-z, 0-9 and {NAME_SEP:?}", self.name),
        )?;
        check(
            positive(self.length_ms) && self.length_ms <= book.format.max_ms,
            format!("length_ms must be above 0, up to {} ms", book.format.max_ms),
        )?;
        check(non_negative(-self.gain_db), "gain_db must be 0 or less")?;
        check(self.variants != Some(0), "variants must be above 0")?;
        check(!self.layers.is_empty(), "a recipe needs a layer")?;
        // A variant may sound higher, so frequencies must leave room for the largest pitch jitter.
        let cents = self.variants.map_or(0.0, |_| book.variants.pitch_cents);
        let nyquist = f64::from(book.format.rate) / 2.0;
        let top = nyquist / (cents / CENTS_PER_OCTAVE).exp2();
        self.layers
            .iter()
            .try_for_each(|layer| layer.check(self.length_ms, top))
    }
}

impl Layer {
    /// Checks the layer's values; `top` is the highest frequency it may reach.
    fn check(&self, length_ms: f64, top: f64) -> Result<(), String> {
        let (start_ms, attack_ms, level, glide) = match self {
            Self::Noise(n) => (n.start_ms, n.attack_ms, n.level, n.glide),
            Self::Modes(m) => (m.start_ms, m.attack_ms, m.level, m.glide),
        };
        check(
            non_negative(start_ms) && start_ms < length_ms,
            "start_ms must be inside the sound",
        )?;
        check(non_negative(attack_ms), "attack_ms must be 0 or more")?;
        check(non_negative(level), "level must be 0 or more")?;
        let rise = glide.map_or(Ok(1.0), Glide::rise)?;
        match self {
            Self::Noise(n) => n.check(top / rise),
            Self::Modes(m) => m.check(top / rise),
        }
    }
}

impl Noise {
    /// Checks the filter and the decay; `top` is the highest frequency allowed.
    fn check(&self, top: f64) -> Result<(), String> {
        check(
            positive(self.hz) && self.hz < top,
            format!("hz must be above 0 and below {top:.0} Hz"),
        )?;
        check(positive(self.q), "q must be above 0")?;
        check(positive(self.decay_ms), "decay_ms must be above 0")
    }
}

impl Modes {
    /// Checks every partial; `top` is the highest frequency allowed.
    fn check(&self, top: f64) -> Result<(), String> {
        check(!self.partials.is_empty(), "modes need partials")?;
        self.partials.iter().try_for_each(|p| {
            check(
                positive(p.hz) && p.hz < top,
                format!("partial hz must be above 0 and below {top:.0} Hz"),
            )?;
            check(positive(p.decay_ms), "partial decay_ms must be above 0")?;
            check(non_negative(p.level), "partial level must be 0 or more")
        })
    }
}

impl Glide {
    /// The highest frequency factor the glide reaches, once its values are checked.
    fn rise(self) -> Result<f64, String> {
        check(
            positive(self.ratio) && positive(self.ms),
            "glide ratio and ms must be above 0",
        )?;
        Ok(self.ratio.max(1.0))
    }
}

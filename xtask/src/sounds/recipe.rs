//! The recipe book: the shipping format from the sounds file, and the recipe file's variant jitter, resampler settings
//! and sound recipes.

mod check;

use kx_theme::sounds::{Format, Sounds};
use serde::Deserialize;
use serde::de::IgnoredAny;

use super::SoundError;

/// Cents in one octave.
pub(crate) const CENTS_PER_OCTAVE: f64 = 1200.0;
/// Joins a recipe name and a variant number; also the only mark a name may hold besides letters and digits.
pub(crate) const NAME_SEP: char = '-';
/// Why a recipe file may not hold `[format]`.
const FORMAT_MOVED: &str =
    "[format] lives in the file [workspace.metadata.sounds] names as `format`; remove it here";

/// The shipping format and the whole recipe file.
#[derive(Debug)]
pub(crate) struct Book {
    /// The shipping format every sound is written in.
    pub(crate) format: Format,
    /// How much the variants of one recipe differ.
    pub(crate) variants: Jitter,
    /// The importer's resampler.
    pub(crate) resample: Resample,
    /// The sound recipes.
    pub(crate) recipes: Vec<Recipe>,
}

/// The recipe file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    /// The old place of the format, read only to refuse it.
    format: Option<IgnoredAny>,
    /// How much the variants of one recipe differ.
    variants: Jitter,
    /// The importer's resampler.
    resample: Resample,
    /// The sound recipes.
    #[serde(rename = "recipe")]
    recipes: Vec<Recipe>,
}

/// How much the variants of one recipe differ.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Jitter {
    /// The largest pitch change, up or down, in cents.
    pub(crate) pitch_cents: f64,
    /// The largest level drop, in dB.
    pub(crate) level_db: f64,
}

/// The importer's windowed-sinc resampler.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resample {
    /// Sinc zero crossings on each side of a sample.
    pub(crate) half_taps: u32,
    /// The cutoff, as a share of the lower of the two Nyquist frequencies.
    pub(crate) rolloff: f64,
}

/// One sound recipe.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recipe {
    /// The file name, without extension.
    pub(crate) name: String,
    /// The sound's length, in ms.
    pub(crate) length_ms: f64,
    /// The peak level under the ceiling, in dB (zero or less).
    pub(crate) gain_db: f64,
    /// How many jittered variants to write; none writes one plain file.
    pub(crate) variants: Option<u32>,
    /// The layers, mixed together.
    #[serde(rename = "layer")]
    pub(crate) layers: Vec<Layer>,
}

/// One layer of a recipe.
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum Layer {
    /// A filtered noise burst.
    Noise(Noise),
    /// Damped sine partials.
    Modes(Modes),
}

/// A noise burst through a biquad filter, with a rise and an exponential decay.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Noise {
    /// The filter's shape.
    pub(crate) filter: Filter,
    /// The filter's corner or centre, in Hz.
    pub(crate) hz: f64,
    /// The filter's Q.
    pub(crate) q: f64,
    /// When the layer starts, in ms.
    pub(crate) start_ms: f64,
    /// The rise time, in ms.
    pub(crate) attack_ms: f64,
    /// The time to fall 60 dB after the rise, in ms.
    pub(crate) decay_ms: f64,
    /// The layer's level.
    pub(crate) level: f64,
    /// An optional sweep of the filter's frequency.
    pub(crate) glide: Option<Glide>,
}

/// Damped sine partials: the modal resonances of a struck body.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Modes {
    /// When the layer starts, in ms.
    pub(crate) start_ms: f64,
    /// The rise time, in ms.
    pub(crate) attack_ms: f64,
    /// The layer's level.
    pub(crate) level: f64,
    /// An optional pitch glide of every partial.
    pub(crate) glide: Option<Glide>,
    /// The partials.
    pub(crate) partials: Vec<Partial>,
}

/// One damped sine.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Partial {
    /// Its frequency, in Hz.
    pub(crate) hz: f64,
    /// The time to fall 60 dB after the rise, in ms.
    pub(crate) decay_ms: f64,
    /// Its level.
    pub(crate) level: f64,
}

/// A frequency glide: the ratio reached, evenly in pitch, after `ms`.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Glide {
    /// The end frequency over the start frequency.
    pub(crate) ratio: f64,
    /// The glide's length, in ms.
    pub(crate) ms: f64,
}

/// A biquad filter shape (RBJ cookbook).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Filter {
    /// Passes below the corner.
    Lowpass,
    /// Passes above the corner.
    Highpass,
    /// Passes around the centre, with a 0 dB peak.
    Bandpass,
}

impl Recipe {
    /// The file stems this recipe writes: its name, or `name-1` to `name-n` with variants.
    pub(crate) fn stems(&self) -> Vec<String> {
        let name = &self.name;
        match self.variants {
            None => vec![name.clone()],
            Some(n) => (1..=n).map(|i| format!("{name}{NAME_SEP}{i}")).collect(),
        }
    }
}

/// Parses and checks a recipe file, `recipes`, with the shipping format from the sounds file, `sounds`.
pub(crate) fn parse(recipes: &str, sounds: &str) -> Result<Book, SoundError> {
    let format = Sounds::parse(sounds).map_err(SoundError::Format)?.format;
    let file: RecipeFile =
        toml::from_str(recipes).map_err(|err| SoundError::Recipe(err.to_string()))?;
    if file.format.is_some() {
        return Err(SoundError::Recipe(FORMAT_MOVED.into()));
    }
    let book = Book {
        format,
        variants: file.variants,
        resample: file.resample,
        recipes: file.recipes,
    };
    book.validate()?;
    Ok(book)
}

#[cfg(test)]
mod tests;

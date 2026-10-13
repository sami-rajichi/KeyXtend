//! The synth: filtered noise bursts and damped sine partials, mixed in f64 from a seeded random stream.

use std::f64::consts::{PI, TAU};

use super::recipe::{CENTS_PER_OCTAVE, Filter, Glide, Jitter, Layer, Modes, Noise, Recipe};
use super::units::{db_gain, ms_at, samples};

/// The level, in dB, that a decay time counts down to (T60).
const DECAY_DB: f64 = -60.0;
/// The step of `SplitMix64`: the golden ratio in 64-bit fixed point.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
/// The two mixing multipliers of `SplitMix64`.
const MIX: [u64; 2] = [0xBF58_476D_1CE4_E5B9, 0x94D0_49BB_1331_11EB];
/// The three mixing shifts of `SplitMix64`.
const SHIFTS: [u32; 3] = [30, 27, 31];
/// FNV-1a's 64-bit offset basis.
const FNV_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
/// FNV-1a's 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
/// The shift that leaves a random word with as many bits as an f64 mantissa holds.
const MANTISSA_SHIFT: u32 = u64::BITS - (f64::MANTISSA_DIGITS - 1);

/// One rendering of a recipe: its pitch ratio, level change and random stream.
pub(crate) struct Voice {
    /// The pitch ratio of this variant.
    pitch: f64,
    /// The level change of this variant, in dB (zero or less).
    pub(crate) level_db: f64,
    /// The random stream, seeded from the file's stem.
    rng: Rng,
}

impl Voice {
    /// The voice of the file `stem`, jittered when `jitter` is given.
    pub(crate) fn new(stem: &str, jitter: Option<&Jitter>) -> Self {
        let mut rng = Rng::new(fnv1a(stem));
        let (pitch, level_db) = jitter.map_or((1.0, 0.0), |j| {
            let cents = j.pitch_cents * rng.signed();
            ((cents / CENTS_PER_OCTAVE).exp2(), -j.level_db * rng.unit())
        });
        Self {
            pitch,
            level_db,
            rng,
        }
    }
}

/// Mixes every layer of `recipe` at `rate`, before any level change.
pub(crate) fn render(recipe: &Recipe, rate: f64, voice: &mut Voice) -> Vec<f64> {
    let mut mix = vec![0.0; samples(recipe.length_ms, rate)];
    for layer in &recipe.layers {
        match layer {
            Layer::Noise(noise) => add_noise(&mut mix, noise, rate, voice),
            Layer::Modes(modes) => add_modes(&mut mix, modes, rate, voice.pitch),
        }
    }
    mix
}

/// Adds a filtered noise burst to `mix`.
fn add_noise(mix: &mut [f64], layer: &Noise, rate: f64, voice: &mut Voice) {
    let mut filter = Biquad::default();
    let start = samples(layer.start_ms, rate);
    for (i, out) in mix.iter_mut().skip(start).enumerate() {
        let ms = ms_at(i, rate);
        let hz = layer.hz * voice.pitch * bend(layer.glide, ms);
        filter.tune(layer.filter, hz, layer.q, rate);
        let gain = layer.level * envelope(ms, layer.attack_ms, layer.decay_ms);
        *out += gain * filter.run(voice.rng.signed());
    }
}

/// Adds damped sine partials to `mix`, each starting at phase zero.
fn add_modes(mix: &mut [f64], layer: &Modes, rate: f64, pitch: f64) {
    let start = samples(layer.start_ms, rate);
    for partial in &layer.partials {
        let mut phase = 0.0_f64;
        for (i, out) in mix.iter_mut().skip(start).enumerate() {
            let ms = ms_at(i, rate);
            let decay = envelope(ms, layer.attack_ms, partial.decay_ms);
            *out += layer.level * partial.level * decay * phase.sin();
            let hz = partial.hz * pitch * bend(layer.glide, ms);
            phase = (phase + TAU * hz / rate) % TAU;
        }
    }
}

/// The gain `ms` after onset: a raised-cosine rise over `attack`, then a fall of 60 dB per `decay`.
fn envelope(ms: f64, attack: f64, decay: f64) -> f64 {
    if ms < attack {
        f64::midpoint(1.0, -(PI * ms / attack).cos())
    } else {
        db_gain(DECAY_DB * (ms - attack) / decay)
    }
}

/// The frequency factor of `glide` at `ms` after onset: it moves evenly in pitch, then holds.
fn bend(glide: Option<Glide>, ms: f64) -> f64 {
    glide.map_or(1.0, |g| g.ratio.powf((ms / g.ms).min(1.0)))
}

/// The FNV-1a hash of `text`, which seeds a file's random stream.
fn fnv1a(text: &str) -> u64 {
    text.bytes()
        .fold(FNV_BASIS, |h, b| (h ^ u64::from(b)).wrapping_mul(FNV_PRIME))
}

/// A `SplitMix64` random stream: small, fast and the same on every machine.
struct Rng(u64);

impl Rng {
    /// A stream seeded with `seed`.
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next random word.
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(GOLDEN);
        let z = (self.0 ^ (self.0 >> SHIFTS[0])).wrapping_mul(MIX[0]);
        let z = (z ^ (z >> SHIFTS[1])).wrapping_mul(MIX[1]);
        z ^ (z >> SHIFTS[2])
    }

    /// A uniform draw from 0 up to 1, built from the bits of a float between 1 and 2.
    fn unit(&mut self) -> f64 {
        f64::from_bits(1.0_f64.to_bits() | (self.next() >> MANTISSA_SHIFT)) - 1.0
    }

    /// A uniform draw from -1 up to 1.
    fn signed(&mut self) -> f64 {
        2.0 * self.unit() - 1.0
    }
}

/// A biquad filter (RBJ cookbook), direct form I.
#[derive(Default)]
struct Biquad {
    /// Feed-forward terms, over a0.
    b: [f64; 3],
    /// Feedback terms a1 and a2, over a0.
    a: [f64; 2],
    /// The last two inputs.
    x: [f64; 2],
    /// The last two outputs.
    y: [f64; 2],
}

impl Biquad {
    /// Sets the filter's shape, frequency and Q, keeping its state.
    fn tune(&mut self, filter: Filter, hz: f64, q: f64, rate: f64) {
        let (sin, cos) = (TAU * hz / rate).sin_cos();
        let alpha = sin / (2.0 * q);
        let (low, high) = (1.0 - cos, 1.0 + cos);
        let b = match filter {
            Filter::Lowpass => [low / 2.0, low, low / 2.0],
            Filter::Highpass => [high / 2.0, -high, high / 2.0],
            Filter::Bandpass => [alpha, 0.0, -alpha],
        };
        let a0 = 1.0 + alpha;
        self.b = b.map(|v| v / a0);
        self.a = [-2.0 * cos / a0, (1.0 - alpha) / a0];
    }

    /// Filters one sample.
    fn run(&mut self, x: f64) -> f64 {
        let [b0, b1, b2] = self.b;
        let [a1, a2] = self.a;
        let y = b0 * x + b1 * self.x[0] + b2 * self.x[1] - a1 * self.y[0] - a2 * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sounds::fixture::BOOK;
    use crate::sounds::recipe::parse;

    #[test]
    fn the_same_stem_gives_the_same_mix_and_another_stem_another() {
        let book = parse(BOOK).unwrap();
        let tap = &book.recipes[0];
        let mix = |stem| render(tap, 48_000.0, &mut Voice::new(stem, Some(&book.variants)));
        assert_eq!(mix("tap-1"), mix("tap-1"));
        assert_ne!(mix("tap-1"), mix("tap-2"));
        assert_eq!(mix("tap-1").len(), 2_880);
    }

    #[test]
    fn a_jittered_voice_stays_within_the_jitter() {
        let book = parse(BOOK).unwrap();
        for i in 0..50 {
            let voice = Voice::new(&format!("v-{i}"), Some(&book.variants));
            assert!((-1.5..=0.0).contains(&voice.level_db));
            assert!((voice.pitch.log2() * 1_200.0).abs() <= 40.0);
        }
        let plain = Voice::new("v", None);
        assert!((plain.pitch - 1.0).abs() < f64::EPSILON && plain.level_db.abs() < f64::EPSILON);
    }

    #[test]
    fn the_envelope_rises_then_falls_60_db_per_decay() {
        assert!(envelope(0.0, 2.0, 10.0).abs() < 1e-12);
        assert!((envelope(1.0, 2.0, 10.0) - 0.5).abs() < 1e-12);
        assert!((envelope(2.0, 2.0, 10.0) - 1.0).abs() < 1e-12);
        assert!((envelope(12.0, 2.0, 10.0) - 1e-3).abs() < 1e-12);
    }

    #[test]
    fn the_random_stream_is_uniform_in_range() {
        let mut rng = Rng::new(7);
        let draws: Vec<f64> = (0..10_000).map(|_| rng.signed()).collect();
        assert!(draws.iter().all(|x| (-1.0..1.0).contains(x)));
        let mean = draws.iter().sum::<f64>() / 10_000.0;
        assert!(mean.abs() < 0.03, "mean {mean}");
    }

    #[test]
    fn a_lowpass_passes_a_level_and_a_highpass_blocks_it() {
        for (filter, expect) in [
            (Filter::Lowpass, 1.0),
            (Filter::Highpass, 0.0),
            (Filter::Bandpass, 0.0),
        ] {
            let mut biquad = Biquad::default();
            biquad.tune(filter, 1_000.0, 0.7, 48_000.0);
            let last = (0..4_800).map(|_| biquad.run(1.0)).last().unwrap();
            assert!((last - expect).abs() < 1e-6, "{filter:?} gave {last}");
        }
    }
}

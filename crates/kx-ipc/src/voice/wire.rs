//! The voice frames' fields and their caps, on the shared field machinery in `crate::wire`.

use crate::limits::{
    MAX_DEVICES, MAX_LANGS, MAX_NAME_BYTES, MAX_PATH_BYTES, MAX_PERCENT, MAX_SOUND_BYTES,
    MAX_SOUND_PIECES, MAX_SPEECH_PIECES, MAX_TAG_BYTES, MAX_TEXT_BYTES, SAMPLE_BYTES,
};
use crate::wire::{FOUR, ONE, Spec, TWO};
use std::ops::RangeInclusive;

/// A file path: UTF-8 without a NUL.
pub(crate) const PATH: Spec = Spec::new(TWO, 1, MAX_PATH_BYTES);
/// A short ASCII name: a language code, a runner or a card's id.
pub(crate) const TAG: Spec = Spec::new(ONE, 1, MAX_TAG_BYTES);
/// A device name or driver version, possibly empty.
pub(crate) const NAME: Spec = Spec::new(ONE, 0, MAX_NAME_BYTES);
/// A piece's text, possibly empty.
pub(crate) const TEXT: Spec = Spec::new(TWO, 0, MAX_TEXT_BYTES);
/// A sound piece's bytes; an empty last piece ends an empty recording.
pub(crate) const SOUND: Spec = Spec::new(FOUR, 0, MAX_SOUND_BYTES);
/// A speech piece's bytes: one sample at least, as a cloud run sends no empty piece.
pub(crate) const SPEECH: Spec = Spec::new(FOUR, SAMPLE_BYTES, MAX_SOUND_BYTES);
/// The count of devices: the processor at least.
pub(crate) const DEVICES: Spec = Spec::new(ONE, 1, MAX_DEVICES);
/// The count of a file's language codes.
pub(crate) const LANGS: Spec = Spec::new(ONE, 0, MAX_LANGS);
/// The silence stop's byte when it is off; its range starts above it.
pub(crate) const SILENCE_OFF: u8 = 0;
/// A warm-up's progress.
pub(crate) const PERCENT: RangeInclusive<u8> = 0..=MAX_PERCENT;

/// Where a model runs: a device kind, and for a card its id and memory need.
const TARGET_MOST: usize = ONE + TAG.most() + FOUR;
/// A model: its path, language, runner, target and threads.
pub(crate) const MODEL_LEAST: usize = PATH.least() + 2 * TAG.least() + ONE + ONE;
pub(crate) const MODEL_MOST: usize = PATH.most() + 2 * TAG.most() + TARGET_MOST + ONE;
/// A setup's numbers after its model: pad, piece, record limit, level rate and silence stop.
const NUMBERS: usize = TWO + ONE + TWO + ONE + ONE;
/// A setup: whether it names a model, the model, the numbers, then the detector's path.
pub(crate) const SETUP_LEAST: usize = ONE + NUMBERS + PATH.least();
pub(crate) const SETUP_MOST: usize = ONE + MODEL_MOST + NUMBERS + PATH.most();
/// A card's check: the model, then the clip's path.
pub(crate) const CHECK_LEAST: usize = MODEL_LEAST + PATH.least();
pub(crate) const CHECK_MOST: usize = MODEL_MOST + PATH.most();
/// A sound piece: its index, whether it is the last, then its bytes.
pub(crate) const SOUND_LEAST: usize = TWO + ONE + SOUND.least();
pub(crate) const SOUND_MOST: usize = TWO + ONE + SOUND.most();
/// A speech piece, laid out as a sound piece.
pub(crate) const SPEECH_LEAST: usize = TWO + ONE + SPEECH.least();
pub(crate) const SPEECH_MOST: usize = TWO + ONE + SPEECH.most();

/// A run of sound pieces: each one's bytes, and the count its index stays below.
#[derive(Copy, Clone)]
pub(crate) struct Pieces {
    pub spec: Spec,
    pub most: usize,
}

/// The held recording's pieces, handed in or kept.
pub(crate) const HELD: Pieces = Pieces {
    spec: SOUND,
    most: MAX_SOUND_PIECES,
};
/// A listen-only Stop's speech pieces.
pub(crate) const SPOKEN: Pieces = Pieces {
    spec: SPEECH,
    most: MAX_SPEECH_PIECES,
};
/// A piece's text after its index.
pub(crate) const PIECE_LEAST: usize = TWO + TEXT.least();
pub(crate) const PIECE_MOST: usize = TWO + TEXT.most();
/// A run's end: its pieces, then sound, wait, load and memory figures, then whether a card served.
pub(crate) const DONE_LEN: usize = TWO + 4 * FOUR + ONE;
/// A device: its kind, id, name, total and free memory, then its driver.
const DEVICE_LEAST: usize = ONE + TAG.least() + NAME.least() + 2 * FOUR + NAME.least();
const DEVICE_MOST: usize = ONE + TAG.most() + NAME.most() + 2 * FOUR + NAME.most();
/// A device list: its count, then each device.
pub(crate) const DEVICES_LEAST: usize = DEVICES.width + DEVICES.min * DEVICE_LEAST;
pub(crate) const DEVICES_MOST: usize = DEVICES.width + DEVICES.max * DEVICE_MOST;
/// A file's facts: its runner, then its language codes.
pub(crate) const FACTS_LEAST: usize = TAG.least() + LANGS.width;
pub(crate) const FACTS_MOST: usize = TAG.most() + LANGS.width + LANGS.max * TAG.most();
/// A card's check result: the card's and the processor's times, then their texts.
pub(crate) const CHECKED_LEAST: usize = 2 * FOUR + 2 * TEXT.least();
pub(crate) const CHECKED_MOST: usize = 2 * FOUR + 2 * TEXT.most();

// Every voice payload fits the header's `u32` length; these are the largest.
const _: () = assert!(CHECK_MOST <= u32::MAX as usize);
const _: () = assert!(SETUP_MOST <= u32::MAX as usize);
const _: () = assert!(SOUND_MOST <= u32::MAX as usize);

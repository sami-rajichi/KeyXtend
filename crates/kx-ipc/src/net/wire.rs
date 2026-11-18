//! The net frames' fields and their caps, on the shared field machinery in `crate::wire`.

pub(crate) use crate::fetch::wire::URL;
use crate::limits::{
    MAX_FIXED, MAX_KEY_BYTES, MAX_NAME_BYTES, MAX_NET_TEXT_BYTES, MAX_SOUND_BYTES,
    MAX_SPEECH_PIECES,
};
pub(crate) use crate::voice::wire::TAG;
use crate::wire::{FOUR, ONE, Spec, TWO};

/// A form value: visible ASCII and spaces, possibly empty.
pub(crate) const VALUE: Spec = Spec::new(ONE, 0, MAX_NAME_BYTES);
/// The count of a form's fixed fields.
pub(crate) const FIXED: Spec = Spec::new(ONE, 0, MAX_FIXED);
/// The user's key: visible ASCII.
pub(crate) const KEY: Spec = Spec::new(TWO, 1, MAX_KEY_BYTES);
/// The count of a run's pieces, each sent as its size.
pub(crate) const SIZES: Spec = Spec::new(TWO, 1, MAX_SPEECH_PIECES);
/// A part of a piece's sound: whole samples.
pub(crate) const SOUND: Spec = Spec::new(FOUR, 1, MAX_SOUND_BYTES);
/// A piece's text, possibly empty.
pub(crate) const TEXT: Spec = Spec::new(FOUR, 0, MAX_NET_TEXT_BYTES);

/// A form: its language field's name, the list flag, its fixed fields' names and values, then the paywall flag.
const FORM_MOST: usize =
    TAG.most() + ONE + FIXED.width + FIXED.max * (TAG.most() + VALUE.most()) + ONE;
/// A shape: its code, then a form.
const SHAPE_LEAST: usize = ONE;
const SHAPE_MOST: usize = ONE + FORM_MOST;
/// A key's header: its code, then a name.
const HEADER_LEAST: usize = ONE;
const HEADER_MOST: usize = ONE + TAG.most();
/// A call, which a test sends alone: its shape, address, header, key and wait.
pub(crate) const TEST_LEAST: usize = SHAPE_LEAST + URL.least() + HEADER_LEAST + KEY.least() + TWO;
pub(crate) const TEST_MOST: usize = SHAPE_MOST + URL.most() + HEADER_MOST + KEY.most() + TWO;
/// A row's limits: megabytes, seconds, requests a minute and at once.
const LIMITS_LEN: usize = FOUR + TWO + TWO + ONE;
/// A run: whether it names a model, the model, the language, the limits, the tries, then each piece's size.
const RUN_LEAST: usize = ONE + TAG.least() + LIMITS_LEN + ONE + SIZES.width + SIZES.min * FOUR;
const RUN_MOST: usize = ONE + 2 * TAG.most() + LIMITS_LEN + ONE + SIZES.width + SIZES.max * FOUR;
/// A send: the call, then the run.
pub(crate) const SEND_LEAST: usize = TEST_LEAST + RUN_LEAST;
pub(crate) const SEND_MOST: usize = TEST_MOST + RUN_MOST;
/// A part of a piece's sound: the piece's index, then the bytes.
pub(crate) const SOUND_LEAST: usize = TWO + SOUND.least();
pub(crate) const SOUND_MOST: usize = TWO + SOUND.most();
/// A piece's text: its index, then the text.
pub(crate) const TEXT_LEAST: usize = TWO + TEXT.least();
pub(crate) const TEXT_MOST: usize = TWO + TEXT.most();
/// A fault: its code, then for the allowance whether its renewal is told, and the seconds.
pub(crate) const FAULT_LEAST: usize = ONE;
pub(crate) const FAULT_MOST: usize = ONE + ONE + FOUR;

// Every net payload fits the header's `u32` length; these are the largest.
const _: () = assert!(SEND_MOST <= u32::MAX as usize);
const _: () = assert!(TEXT_MOST <= u32::MAX as usize);

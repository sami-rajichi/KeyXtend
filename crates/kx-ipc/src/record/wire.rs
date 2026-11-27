//! The record frames' fields and their caps, on the shared field machinery in `crate::wire`.

use crate::wire::{FOUR, ONE, PATH, TWO};

/// A part's four edges.
const AREA: usize = 4 * FOUR;
/// A start's fields before its video's path: the part, the rate, both switches, the longest length and the floor.
const START_HEAD: usize = AREA + ONE + 2 * ONE + TWO + FOUR;
/// A start: its fields, then its video's path.
pub(crate) const START_LEAST: usize = START_HEAD + PATH.least();
pub(crate) const START_MOST: usize = START_HEAD + PATH.most();
/// A copy: the video's path, the speed, then the copy's path.
pub(crate) const SPEED_LEAST: usize = 2 * PATH.least() + ONE;
pub(crate) const SPEED_MOST: usize = 2 * PATH.most() + ONE;
/// A time: its ms.
pub(crate) const TIME_LEN: usize = FOUR;
/// An end: the video's length in ms, then why it ended.
pub(crate) const END_LEN: usize = FOUR + ONE;
/// A switch's ack: which switch, then its new state.
pub(crate) const SWITCHED_LEN: usize = 2 * ONE;
/// A lost sound: which switch.
pub(crate) const LOST_LEN: usize = ONE;

// The largest record ask fits the header's `u32` length.
const _: () = assert!(SPEED_MOST <= u32::MAX as usize);

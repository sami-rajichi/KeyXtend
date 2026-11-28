//! Snip's shared values (ADR-0056): a raw pixel's bytes, the frozen copy's cap and the most an `Encode` carries, the
//! `[snip]` settings' ranges a recording is asked with, the longest video, the speeds, the longest proposed name and
//! the file kinds' extensions.

use super::{MIB, MS_PER_S};
use std::ops::RangeInclusive;

/// Bytes in one raw screen pixel: blue, green, red and one unused byte.
pub const BGRX_BYTES: usize = 4;
/// The `shot_cap_mb` setting's range: the largest frozen copy of every screen, in MiB of `MIB` bytes.
pub const SNIP_SHOT_MB: RangeInclusive<u32> = 64..=1024;
/// The most raw pixel bytes one `Encode` carries: a picture is cut from a frozen copy, which `SNIP_SHOT_MB` caps.
pub const MAX_ENCODE_BYTES: usize = *SNIP_SHOT_MB.end() as usize * MIB;
/// The `fps` setting's choices: frames a second a video records at.
pub const SNIP_FPS: [u8; 2] = [15, 30];
/// How many times faster a re-timed copy plays, its frames dropped.
pub const SNIP_SPEEDS: [u8; 3] = [1, 2, 4];
/// The `max_min` setting's range: the longest video, in minutes.
pub const SNIP_MAX_MIN: RangeInclusive<u16> = 1..=120;
/// Seconds in a minute, the unit of `max_min`.
const MIN_S: u32 = 60;
/// The longest video in ms, the top of `max_min`; no time or length the `record` job tells passes it.
pub const MAX_VIDEO_MS: u32 = *SNIP_MAX_MIN.end() as u32 * MIN_S * MS_PER_S;
/// The `min_free_mb` setting's range: the room a recording leaves on its drive, in MiB of `MIB` bytes.
pub const SNIP_MIN_FREE_MB: RangeInclusive<u32> = 512..=20_480;
/// The `record_mb` setting's range: the `record` job's memory cap, in MiB of `MIB` bytes.
pub const SNIP_RECORD_MB: RangeInclusive<u32> = 256..=2048;
/// UTF-16 units Windows allows in one file name.
pub(crate) const NAME_UNITS: usize = 255;
/// The most UTF-8 bytes one UTF-16 unit stands for: three in the basic plane, four for a pair of two.
const UNIT_BYTES: usize = 3;
/// The longest name a Save dialog is proposed, in UTF-8 bytes: `NAME_UNITS` units of at most `UNIT_BYTES`.
pub const MAX_FILE_NAME_BYTES: usize = NAME_UNITS * UNIT_BYTES;
/// A picture's extension, lower case.
pub const PNG_EXT: &str = "png";
/// A video's extension, lower case.
pub const MP4_EXT: &str = "mp4";

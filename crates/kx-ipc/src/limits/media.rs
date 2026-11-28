//! The media job's frame caps (ADR-0034 decision 5, ADR-0056 decision 2): a draw, a DIB, pixels, an `Encode` and its
//! PNG, each from one source, and the bounded reads under them.

use super::snip::MAX_ENCODE_BYTES;
use super::{KIB, MIB};

/// Bytes in one RGBA8 pixel.
pub const RGBA_BYTES: usize = 4;
/// The longest side, in pixels, that a draw may ask for and a reply may hold.
pub const MAX_SIDE: u32 = 4096;
/// The largest image a draw may carry: the largest clipboard entry the settings allow, 64 MiB.
pub const MAX_IMAGE_BYTES: usize = 64 * MIB;
/// The largest pixels a reply may carry: `MAX_SIDE` by `MAX_SIDE` in RGBA8, 64 MiB.
pub const MAX_RGBA_BYTES: usize = MAX_SIDE as usize * MAX_SIDE as usize * RGBA_BYTES;

/// A draw's fields before its image: the codec and the longest side.
pub(crate) const DRAW_HEAD: usize = size_of::<u8>() + size_of::<u32>();
/// A DIB ask's field before its PNG: the most bytes the DIB may hold.
pub(crate) const MAKE_HEAD: usize = size_of::<u32>();
/// An `Encode`'s fields before its pixels: the width, the height and the bytes a row.
pub(crate) const ENCODE_HEAD: usize = 3 * size_of::<u32>();
/// The fewest image bytes a draw carries: an empty image is refused.
pub(crate) const MIN_IMAGE_BYTES: usize = 1;
/// A pixel reply's fields before its pixels: the width and the height.
pub(crate) const PIXELS_HEAD: usize = 2 * size_of::<u32>();
/// A failure reply's only field: the reason.
pub(crate) const FAILED_LEN: usize = size_of::<u8>();
/// The largest draw payload: its fields and the largest image.
pub(crate) const MAX_DRAW_LEN: usize = DRAW_HEAD + MAX_IMAGE_BYTES;
/// The largest pixel reply payload, and so the most `Reply::read` takes: its fields and the largest pixels.
pub(crate) const MAX_PIXELS_LEN: usize = PIXELS_HEAD + MAX_RGBA_BYTES;
/// The largest `Encode` payload: its fields and the pixels of the largest frozen copy.
pub(crate) const MAX_ENCODE_LEN: usize = ENCODE_HEAD + MAX_ENCODE_BYTES;

/// How far a PNG may outgrow its raw pixels, as a share: 1/64, above stored deflate blocks and 1 KiB chunks' frames.
const PNG_GROWTH: usize = 64;
/// What a PNG may add whatever its size: its signature, header and end, and any small chunk an encoder writes.
const PNG_FIXED: usize = 64 * KIB;

/// The most bytes the PNG of `raw` pixel bytes may take: the pixels, a 1/`PNG_GROWTH` share and `PNG_FIXED`.
#[must_use]
pub const fn png_most(raw: usize) -> usize {
    raw.saturating_add(raw / PNG_GROWTH)
        .saturating_add(PNG_FIXED)
}

/// The largest PNG an `Encode` reply may carry.
pub const MAX_PNG_BYTES: usize = png_most(MAX_ENCODE_BYTES);
/// The largest media ask payload: a draw's or an `Encode`'s, whichever is larger.
pub const MAX_ASK_LEN: usize = larger(MAX_DRAW_LEN, MAX_ENCODE_LEN);
/// The largest media reply payload: a pixel reply's or an `Encode`'s PNG, whichever is larger.
pub const MAX_REPLY_LEN: usize = larger(MAX_PIXELS_LEN, MAX_PNG_BYTES);

/// The first buffer a payload read takes; it then doubles only as bytes arrive, up to the declared length.
pub(crate) const READ_CHUNK: usize = 64 * KIB;
/// The most times its length a payload read holds while its buffer grows: the old and new buffers together.
pub const READ_TIMES: usize = 2;
/// What the worker's PNG decoder may allocate for its rows and chunks: 16 MiB, rows up to 4 million pixels wide.
pub const DECODE_BYTES: usize = 16 * MIB;

/// The larger of `a` and `b`, in a constant.
const fn larger(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

// Every media payload fits the header's `u32` field.
const _: () = assert!(MAX_ASK_LEN <= u32::MAX as usize);
const _: () = assert!(MAX_REPLY_LEN <= u32::MAX as usize);
// A DIB ask fits the largest ask, and a DIB reply the bound `Reply::read` holds to.
const _: () = assert!(MAKE_HEAD + MAX_IMAGE_BYTES <= MAX_ASK_LEN);
const _: () = assert!(MAX_IMAGE_BYTES <= MAX_PIXELS_LEN);

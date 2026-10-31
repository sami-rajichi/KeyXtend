//! The protocol's magic, version and size limits, in one place.

/// The bytes every frame starts with.
pub(crate) const MAGIC: [u8; 4] = *b"KXIP";
/// The protocol version; a frame with any other is refused.
pub const VERSION: u8 = 1;
/// The fixed header: magic, version, kind, request id and payload length.
pub(crate) const HEADER_LEN: usize = MAGIC.len() + 2 * size_of::<u8>() + 2 * size_of::<u32>();

/// Bytes in one RGBA8 pixel.
pub const RGBA_BYTES: usize = 4;
/// The longest side, in pixels, that a draw may ask for and a reply may hold.
pub const MAX_SIDE: u32 = 4096;
/// The largest image a draw may carry: the largest clipboard entry the settings allow, 64 MiB.
pub const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
/// The largest pixels a reply may carry: `MAX_SIDE` by `MAX_SIDE` in RGBA8, 64 MiB.
pub const MAX_RGBA_BYTES: usize = MAX_SIDE as usize * MAX_SIDE as usize * RGBA_BYTES;

/// A draw's fields before its image: the codec and the longest side.
pub(crate) const DRAW_HEAD: usize = size_of::<u8>() + size_of::<u32>();
/// The fewest image bytes a draw carries: an empty image is refused.
pub(crate) const MIN_IMAGE_BYTES: usize = 1;
/// A pixel reply's fields before its pixels: the width and the height.
pub(crate) const PIXELS_HEAD: usize = 2 * size_of::<u32>();
/// A failure reply's only field: the reason.
pub(crate) const FAILED_LEN: usize = size_of::<u8>();
/// The largest payload the host sends: a draw's fields and the largest image.
pub const MAX_ASK_LEN: usize = DRAW_HEAD + MAX_IMAGE_BYTES;
/// The largest payload the worker sends: a pixel reply's fields and the largest pixels.
pub const MAX_REPLY_LEN: usize = PIXELS_HEAD + MAX_RGBA_BYTES;

/// The first buffer a payload read takes; it then doubles only as bytes arrive, up to the declared length.
pub(crate) const READ_CHUNK: usize = 64 * 1024;
/// The most times its length a payload read holds while its buffer grows: the old and new buffers together.
pub const READ_TIMES: usize = 2;
/// What the worker's PNG decoder may allocate for its rows and chunks: 16 MiB, rows up to 4 million pixels wide.
pub const DECODE_BYTES: usize = 16 * 1024 * 1024;

// Every payload length fits the header's `u32` field.
const _: () = assert!(MAX_ASK_LEN <= u32::MAX as usize);
const _: () = assert!(MAX_REPLY_LEN <= u32::MAX as usize);

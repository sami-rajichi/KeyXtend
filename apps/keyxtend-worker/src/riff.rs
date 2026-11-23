//! The RIFF/WAVE layout the voice job reads and the speech job writes, named once: chunk ids, sizes and the plain PCM
//! format.

/// The outer chunk's id.
pub const RIFF_ID: &[u8] = b"RIFF";
/// The form a WAV file names after the outer chunk's size.
pub const WAVE_ID: &[u8] = b"WAVE";
/// The format chunk's id.
pub const FMT_ID: &[u8] = b"fmt ";
/// The samples chunk's id.
pub const DATA_ID: &[u8] = b"data";
/// Where the first chunk starts: after `RIFF`, the size and `WAVE`.
pub const FIRST_CHUNK: usize = 12;
/// Bytes of a chunk's id and size.
pub const CHUNK_HEAD: usize = 8;
/// The plain format chunk's length.
pub const FMT_LEAST: usize = 16;
/// The plain PCM format tag.
pub const PCM: u16 = 1;
/// The width of a PCM sample the jobs take.
pub const PCM_BITS: u16 = 16;
/// Bits in a byte.
pub const BYTE_BITS: u16 = 8;

//! Base64 (RFC 4648, padded) as a request writes a piece inline: written chunk by chunk through `Display`, so the JSON
//! writer puts it straight into the request's wiped body and no second, unwiped copy of the sound is made.

use std::fmt;
use zeroize::Zeroizing;

/// The standard alphabet, one character for each six bits.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// What pads a last group short of its bytes.
const PAD: u8 = b'=';
/// Bytes in a group, and the characters that write it.
const GROUP_IN: usize = 3;
const GROUP_OUT: usize = 4;
/// Bits in a byte and in a character, and the mask of a character's bits.
const BYTE_BITS: usize = 8;
const SEXTET_BITS: usize = 6;
const SEXTET: u32 = 0x3F;
/// Groups written at a time, so only the last chunk's last group pads.
const CHUNK_GROUPS: usize = 1024;

/// Bytes written as Base64.
pub struct Base64<'a>(pub &'a [u8]);

impl fmt::Display for Base64<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = Zeroizing::new([0u8; CHUNK_GROUPS * GROUP_OUT]);
        for chunk in self.0.chunks(CHUNK_GROUPS * GROUP_IN) {
            for (group, to) in chunk.chunks(GROUP_IN).zip(out.chunks_mut(GROUP_OUT)) {
                group_into(group, to);
            }
            let n = chunk.len().div_ceil(GROUP_IN) * GROUP_OUT;
            f.write_str(std::str::from_utf8(&out[..n]).map_err(|_| fmt::Error)?)?;
        }
        Ok(())
    }
}

/// Writes `group`, one to three bytes, as four characters into `to`, padded after a short group's last character.
fn group_into(group: &[u8], to: &mut [u8]) {
    let word = (0..GROUP_IN).fold(0u32, |word, i| {
        (word << BYTE_BITS) | u32::from(group.get(i).copied().unwrap_or(0))
    });
    for (i, c) in to.iter_mut().enumerate() {
        let shift = (GROUP_OUT - 1 - i) * SEXTET_BITS;
        let at = usize::try_from((word >> shift) & SEXTET).unwrap_or(0);
        *c = if i <= group.len() { ALPHABET[at] } else { PAD };
    }
}

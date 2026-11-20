//! Base64 (RFC 4648, padded) by the `base64` crate (ADR-0047), written chunk by chunk through `Display` from a wiped
//! buffer, so the JSON writer puts it straight into the request's wiped body and no unwiped copy of the sound is made.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use std::fmt;
use zeroize::Zeroizing;

/// Bytes in a group, and the characters that write it.
const GROUP_IN: usize = 3;
const GROUP_OUT: usize = 4;
/// Groups written at a time, so only the last chunk's last group pads.
const CHUNK_GROUPS: usize = 1024;

/// Bytes written as Base64.
pub struct Base64<'a>(pub &'a [u8]);

impl fmt::Display for Base64<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = Zeroizing::new([0u8; CHUNK_GROUPS * GROUP_OUT]);
        for chunk in self.0.chunks(CHUNK_GROUPS * GROUP_IN) {
            let n = STANDARD
                .encode_slice(chunk, &mut out[..])
                .map_err(|_| fmt::Error)?;
            f.write_str(std::str::from_utf8(&out[..n]).map_err(|_| fmt::Error)?)?;
        }
        Ok(())
    }
}

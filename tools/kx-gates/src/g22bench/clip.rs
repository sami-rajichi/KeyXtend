//! A check clip's length, read from its WAV header: the data chunk's size over the format's byte rate.
#![cfg(windows)]

use std::path::Path;

use kx_module_api::MS_PER_S;

/// A RIFF file's mark, the kind a WAV file names, and the two chunks the length needs.
const RIFF: &[u8] = b"RIFF";
const WAVE: &[u8] = b"WAVE";
const FMT: &[u8] = b"fmt ";
const DATA: &[u8] = b"data";
/// Bytes of a chunk's id, and of a size or a rate.
const ID_LEN: usize = 4;
const WORD_LEN: usize = 4;
/// Where the first chunk starts: after the mark, the file's size and its kind.
const FIRST: usize = 12;
/// Where the byte rate sits in the format chunk's body.
const BYTE_RATE_AT: usize = 8;

/// The length of the WAV sound in `bytes`, in ms; `None` for anything but a whole WAV file with a byte rate.
pub(super) fn length_ms(bytes: &[u8]) -> Option<u32> {
    if bytes.get(..ID_LEN)? != RIFF || bytes.get(FIRST - ID_LEN..FIRST)? != WAVE {
        return None;
    }
    let (mut rate, mut at) = (None, FIRST);
    while let Some((id, body)) = chunk(bytes, at) {
        if id == FMT {
            rate = word(body, BYTE_RATE_AT).filter(|r| *r > 0);
        } else if id == DATA {
            let len = u64::try_from(body.len()).ok()?;
            return u32::try_from(len * u64::from(MS_PER_S) / u64::from(rate?)).ok();
        }
        // RIFF pads an odd body to an even length.
        let size = body.len() + body.len() % 2;
        at = at.saturating_add(ID_LEN + WORD_LEN + size);
    }
    None
}

/// The clip at `path`'s length, in ms.
pub(super) fn read_ms(path: &Path) -> Result<u32, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    length_ms(&bytes).ok_or_else(|| format!("{} is no WAV file", path.display()))
}

/// The chunk at `at`: its id and its body, which must fit in `bytes`.
fn chunk(bytes: &[u8], at: usize) -> Option<(&[u8], &[u8])> {
    let id = bytes.get(at..at.checked_add(ID_LEN)?)?;
    let size = usize::try_from(word(bytes, at + ID_LEN)?).ok()?;
    let start = at + ID_LEN + WORD_LEN;
    Some((id, bytes.get(start..start.checked_add(size)?)?))
}

/// The little-endian number at `at` in `bytes`.
fn word(bytes: &[u8], at: usize) -> Option<u32> {
    let four = bytes.get(at..at.checked_add(WORD_LEN)?)?;
    Some(u32::from_le_bytes(four.try_into().ok()?))
}

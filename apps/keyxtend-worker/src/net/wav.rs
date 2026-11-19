//! A piece wrapped as a WAV file for a service: the plain 44-byte header of 16 kHz mono 16-bit sound, in the layout
//! the voice job reads (`crate::riff`), in a wiped buffer that takes its whole room at once, so no growth leaves an
//! unwiped copy of the sound.

use crate::riff::{
    BYTE_BITS, CHUNK_HEAD, DATA_ID, FIRST_CHUNK, FMT_ID, FMT_LEAST, PCM, PCM_BITS, RIFF_ID, WAVE_ID,
};
use kx_ipc::{NetFault, SAMPLE_BYTES, SOUND_HZ};
use zeroize::Zeroizing;

/// The header's bytes before the samples: the outer chunk, the format chunk and the samples chunk's head.
pub const HEAD: usize = FIRST_CHUNK + CHUNK_HEAD + FMT_LEAST + CHUNK_HEAD;
/// A piece's media type, as every shape names it.
pub const TYPE: &str = "audio/wav";
/// The hand-over sound's channels: mono.
const CHANNELS: u16 = 1;

// The hand-over sample is the PCM width the header names.
const _: () = assert!(SAMPLE_BYTES * BYTE_BITS as usize == PCM_BITS as usize);

/// A wiped buffer holding the header for `samples` bytes of hand-over sound, with room for them after it.
///
/// # Errors
/// `Range` for more bytes than a WAV header holds, `Gone` when the room cannot be had.
pub fn start(samples: usize) -> Result<Zeroizing<Vec<u8>>, NetFault> {
    let word = |n: usize| u32::try_from(n).map_err(|_| NetFault::Range);
    let riff = word(
        samples
            .checked_add(HEAD - CHUNK_HEAD)
            .ok_or(NetFault::Range)?,
    )?;
    let (rate, block) = (word(SOUND_HZ)?, word(SAMPLE_BYTES)? * u32::from(CHANNELS));
    let block16 = u16::try_from(block).map_err(|_| NetFault::Range)?;
    let mut wav = Zeroizing::new(Vec::new());
    wav.try_reserve_exact(HEAD + samples)
        .map_err(|_| NetFault::Gone)?;
    let parts: [&[u8]; 13] = [
        RIFF_ID,
        &riff.to_le_bytes(),
        WAVE_ID,
        FMT_ID,
        &word(FMT_LEAST)?.to_le_bytes(),
        &PCM.to_le_bytes(),
        &CHANNELS.to_le_bytes(),
        &rate.to_le_bytes(),
        &(rate * block).to_le_bytes(),
        &block16.to_le_bytes(),
        &PCM_BITS.to_le_bytes(),
        DATA_ID,
        &word(samples)?.to_le_bytes(),
    ];
    for p in parts {
        wav.extend_from_slice(p);
    }
    Ok(wav)
}

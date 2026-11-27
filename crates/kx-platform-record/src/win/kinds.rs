//! The media types a video writes and takes: H.264 video from BGRA or NV12 frames, and AAC sound from 16-bit stereo
//! PCM at the track's rate.

use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE,
    MF_MT_AUDIO_BLOCK_ALIGNMENT, MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND,
    MF_MT_AVG_BITRATE, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE,
    MF_MT_PIXEL_ASPECT_RATIO, MF_MT_SUBTYPE, MFAudioFormat_AAC, MFAudioFormat_PCM,
    MFCreateMediaType, MFMediaType_Audio, MFMediaType_Video, MFVideoFormat_H264,
    MFVideoInterlace_Progressive,
};
use windows::core::GUID;

use crate::record::{CHANNELS, RATE, Shape};

/// Bits in one PCM sample.
pub(super) const BITS: u32 = 16;
/// Media Foundation packs a pair into the high and low halves of 64 bits.
const HALF: u32 = 32;

/// `high` and `low` packed as Media Foundation packs a pair.
pub(super) fn pair(high: u32, low: u32) -> u64 {
    (u64::from(high) << HALF) | u64::from(low)
}

/// The two halves of a packed pair.
pub(super) fn halves(packed: u64) -> (u32, u32) {
    let low = u32::try_from(packed & u64::from(u32::MAX)).unwrap_or(0);
    (u32::try_from(packed >> HALF).unwrap_or(0), low)
}

/// The video's type: H.264 at `bits_s` when given, else the frames of `kind` it is made from.
#[allow(unsafe_code, reason = "Fills a media type of ours.")]
pub(super) fn video(
    shape: &Shape,
    kind: GUID,
    bits_s: Option<u32>,
) -> windows::core::Result<IMFMediaType> {
    let progressive = MFVideoInterlace_Progressive.0.cast_unsigned();
    let sub = if bits_s.is_some() {
        MFVideoFormat_H264
    } else {
        kind
    };
    // SAFETY: a new media type, set from plain values.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &raw const sub)?;
        t.SetUINT32(&MF_MT_INTERLACE_MODE, progressive)?;
        t.SetUINT64(&MF_MT_FRAME_SIZE, pair(shape.width, shape.height))?;
        t.SetUINT64(&MF_MT_FRAME_RATE, pair(shape.fps, 1))?;
        t.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, pair(1, 1))?;
        if let Some(bits) = bits_s {
            t.SetUINT32(&MF_MT_AVG_BITRATE, bits)?;
        }
        Ok(t)
    }
}

/// The sound's type: AAC at `aac_bytes_s` when given, else the 16-bit stereo PCM it is made from.
#[allow(unsafe_code, reason = "Fills a media type of ours.")]
pub(super) fn audio(aac_bytes_s: Option<u32>) -> windows::core::Result<IMFMediaType> {
    let block = u32::from(CHANNELS) * BITS / u8::BITS;
    let sub = if aac_bytes_s.is_some() {
        MFAudioFormat_AAC
    } else {
        MFAudioFormat_PCM
    };
    // SAFETY: a new media type, set from plain values.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)?;
        t.SetGUID(&MF_MT_SUBTYPE, &raw const sub)?;
        t.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, BITS)?;
        t.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, RATE)?;
        t.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, u32::from(CHANNELS))?;
        t.SetUINT32(
            &MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
            aac_bytes_s.unwrap_or(RATE * block),
        )?;
        if aac_bytes_s.is_none() {
            t.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, block)?;
        }
        Ok(t)
    }
}

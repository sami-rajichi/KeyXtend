//! A video read for a re-timed copy with Media Foundation's Source Reader: its frames decoded to NV12 in memory and its
//! sound to 16-bit stereo PCM at the track's rate, in the order the file holds them.

use std::path::Path;

use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFSample, IMFSourceReader, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE,
    MF_MT_SUBTYPE, MF_PD_DURATION, MF_SOURCE_READER_ALL_STREAMS, MF_SOURCE_READER_ANY_STREAM,
    MF_SOURCE_READER_CONSTANTS, MF_SOURCE_READER_FLAG, MF_SOURCE_READER_MEDIASOURCE,
    MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED, MF_SOURCE_READERF_ENDOFSTREAM,
    MF_SOURCE_READERF_ERROR, MFCreateMediaType, MFCreateSourceReaderFromURL, MFMediaType_Audio,
    MFMediaType_Video, MFVideoFormat_NV12,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToUInt64;
use windows::core::{GUID, HSTRING};

use super::kinds::{audio, halves};
use super::refused;
use super::writer::STEPS_S;
use crate::record::{Piece, RecordFault, Reel, Shape};

/// Media Foundation's 100 ns steps in a millisecond.
const STEPS_MS: u64 = 10_000;

/// A decoded frame, with the frame size its buffer is laid out at, which may hold rows past the picture.
pub struct Decoded {
    /// The frame.
    pub(super) sample: IMFSample,
    /// Its buffer's frame size.
    pub(super) size: (u32, u32),
}

/// A video being read.
pub(super) struct WinReel {
    reader: IMFSourceReader,
    shape: Shape,
    length_ms: u32,
    /// The decoder's frame size, which it may change after the first frame.
    size: (u32, u32),
    /// The video stream's own number in the file, as each read names it.
    video: u32,
    /// The streams not ended yet: the video's, and the sound's when the file has one.
    left: Vec<u32>,
}

/// A stream number the reader takes.
fn index(which: MF_SOURCE_READER_CONSTANTS) -> u32 {
    which.0.cast_unsigned()
}

impl WinReel {
    /// Opens `from`, its video decoded to NV12 and its sound to the track's PCM.
    #[allow(unsafe_code, reason = "Reads a video of ours.")]
    pub(super) fn open(from: &Path) -> Result<Self, RecordFault> {
        let url = HSTRING::from(from.as_os_str());
        // SAFETY: a live path; no attributes.
        let reader = unsafe { MFCreateSourceReaderFromURL(&url, None) }.map_err(refused)?;
        let video = first(&reader, &MFMediaType_Video).ok_or(RecordFault::Failed)?;
        let sound = first(&reader, &MFMediaType_Audio);
        let shape = native(&reader, video).map_err(refused)?;
        let (nv12, pcm) = (nv12().map_err(refused)?, audio(None).map_err(refused)?);
        // SAFETY: a live reader and media types of ours; a sound it cannot decode is left out, so the video is read
        // alone.
        let heard = unsafe {
            reader
                .SetStreamSelection(index(MF_SOURCE_READER_ALL_STREAMS), false)
                .map_err(refused)?;
            reader.SetStreamSelection(video, true).map_err(refused)?;
            reader
                .SetCurrentMediaType(video, None, &nv12)
                .map_err(refused)?;
            sound.filter(|&s| {
                reader.SetCurrentMediaType(s, None, &pcm).is_ok()
                    && reader.SetStreamSelection(s, true).is_ok()
            })
        };
        let mut reel = Self {
            length_ms: length_ms(&reader).map_err(refused)?,
            reader,
            shape,
            size: (shape.width, shape.height),
            video,
            left: std::iter::once(video).chain(heard).collect(),
        };
        reel.size = reel.decoded().map_err(refused)?;
        Ok(reel)
    }

    /// The decoder's current frame size.
    #[allow(unsafe_code, reason = "Reads the reader's current type.")]
    fn decoded(&self) -> windows::core::Result<(u32, u32)> {
        // SAFETY: a live reader and the type it hands back.
        unsafe {
            let t = self.reader.GetCurrentMediaType(self.video)?;
            Ok(halves(t.GetUINT64(&MF_MT_FRAME_SIZE)?))
        }
    }

    /// Reads the next sample of any stream left: its stream, and the sample when one came.
    #[allow(unsafe_code, reason = "Reads a sample of ours.")]
    fn read(&mut self) -> windows::core::Result<(u32, Option<IMFSample>)> {
        let (mut stream, mut flags, mut sample) = (0_u32, 0_u32, None);
        // SAFETY: a live reader and locals for it to fill.
        unsafe {
            self.reader.ReadSample(
                index(MF_SOURCE_READER_ANY_STREAM),
                0,
                Some(&raw mut stream),
                Some(&raw mut flags),
                None,
                Some(&raw mut sample),
            )?;
        }
        let has = |flag: MF_SOURCE_READER_FLAG| flags & flag.0.cast_unsigned() != 0;
        if has(MF_SOURCE_READERF_ERROR) {
            return Err(windows::core::Error::empty());
        }
        if has(MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED) {
            self.size = self.decoded()?;
        }
        if has(MF_SOURCE_READERF_ENDOFSTREAM) {
            self.left.retain(|s| *s != stream);
        }
        Ok((stream, sample))
    }
}

/// The file's own number of its first stream of `major` type, if it has one.
#[allow(unsafe_code, reason = "Reads the file's own stream types.")]
fn first(reader: &IMFSourceReader, major: &GUID) -> Option<u32> {
    let mut n = 0;
    // SAFETY: a live reader; a number past the last stream is refused, which ends the search, and each type it hands
    // back is read for a plain value.
    while let Ok(t) = unsafe { reader.GetNativeMediaType(n, 0) } {
        // SAFETY: as above.
        if unsafe { t.GetMajorType() }.is_ok_and(|k| k == *major) {
            return Some(n);
        }
        n += 1;
    }
    None
}

impl Reel<Decoded> for WinReel {
    fn shape(&self) -> Shape {
        self.shape
    }

    fn length_ms(&self) -> u32 {
        self.length_ms
    }

    fn next(&mut self) -> Result<Option<Piece<Decoded>>, RecordFault> {
        while !self.left.is_empty() {
            let (stream, sample) = self.read().map_err(refused)?;
            let Some(sample) = sample else { continue };
            if stream == self.video {
                let (size, fps) = (self.size, self.shape.fps);
                let slot = slot(&sample, fps).map_err(refused)?;
                return Ok(Some(Piece::Frame(slot, Decoded { sample, size })));
            }
            return pcm(&sample).map(|p| Some(Piece::Sound(p))).map_err(refused);
        }
        Ok(None)
    }
}

/// The slot at `fps` that `sample` plays at, from its own time, rounded to the nearest.
#[allow(unsafe_code, reason = "Reads a sample's time.")]
fn slot(sample: &IMFSample, fps: u32) -> windows::core::Result<u64> {
    // SAFETY: a live sample; its time is a plain value.
    let at = unsafe { sample.GetSampleTime()? };
    let at = u64::try_from(at).unwrap_or(0);
    Ok(at
        .saturating_mul(u64::from(fps))
        .saturating_add(STEPS_S / 2)
        / STEPS_S)
}

/// The video's own frame size and rate, as the file holds them.
#[allow(unsafe_code, reason = "Reads the file's own video type.")]
fn native(reader: &IMFSourceReader, video: u32) -> windows::core::Result<Shape> {
    // SAFETY: a live reader and the type it hands back.
    let t = unsafe { reader.GetNativeMediaType(video, 0)? };
    // SAFETY: a live type; both are plain values.
    let (size, rate) = unsafe {
        (
            t.GetUINT64(&MF_MT_FRAME_SIZE)?,
            t.GetUINT64(&MF_MT_FRAME_RATE)?,
        )
    };
    let ((width, height), (num, den)) = (halves(size), halves(rate));
    let fps = (num + den.max(1) / 2) / den.max(1);
    Ok(Shape { width, height, fps })
}

/// The NV12 frames the decoder gives.
#[allow(unsafe_code, reason = "Fills a media type of ours.")]
fn nv12() -> windows::core::Result<IMFMediaType> {
    // SAFETY: a new media type, set from plain values.
    unsafe {
        let t = MFCreateMediaType()?;
        t.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        t.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
        Ok(t)
    }
}

/// The file's length in ms.
#[allow(unsafe_code, reason = "Reads the file's length.")]
fn length_ms(reader: &IMFSourceReader) -> windows::core::Result<u32> {
    let whole = index(MF_SOURCE_READER_MEDIASOURCE);
    // SAFETY: a live reader; the value it hands back is read as a plain number.
    let steps = unsafe {
        let length = reader.GetPresentationAttribute(whole, &MF_PD_DURATION)?;
        PropVariantToUInt64(&raw const length)?
    };
    Ok(u32::try_from(steps / STEPS_MS).unwrap_or(u32::MAX))
}

/// The 16-bit samples a decoded sound sample holds.
#[allow(unsafe_code, reason = "Reads a decoded buffer once.")]
fn pcm(sample: &IMFSample) -> windows::core::Result<Vec<i16>> {
    // SAFETY: a live sample's buffer, locked once and unlocked once; its bytes are copied out between.
    unsafe {
        let buffer = sample.ConvertToContiguousBuffer()?;
        let (mut data, mut len) = (std::ptr::null_mut(), 0_u32);
        buffer.Lock(&raw mut data, None, Some(&raw mut len))?;
        let bytes = std::slice::from_raw_parts(data, usize::try_from(len).unwrap_or(0));
        let out = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes(*b))
            .collect();
        buffer.Unlock()?;
        Ok(out)
    }
}

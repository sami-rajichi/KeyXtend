//! A recording read back with Media Foundation's Source Reader, as a player reads it: its length, each stream's format,
//! the video's frames, and how many pixels at the centre of a decoded frame are near a colour.
#![allow(
    unsafe_code,
    reason = "The probes read their own recording through Media Foundation."
)]

use std::path::Path;

use windows::Win32::Media::MediaFoundation::{
    IMFMediaType, IMFSample, IMFSourceReader, MF_MT_AUDIO_BITS_PER_SAMPLE,
    MF_MT_AUDIO_NUM_CHANNELS, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
    MF_PD_DURATION, MF_SOURCE_READER_ALL_STREAMS, MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
    MF_SOURCE_READER_FIRST_AUDIO_STREAM, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
    MF_SOURCE_READER_MEDIASOURCE, MF_SOURCE_READERF_ENDOFSTREAM, MF_VERSION, MFAudioFormat_PCM,
    MFCreateAttributes, MFCreateMediaType, MFCreateSourceReaderFromURL, MFMediaType_Audio,
    MFMediaType_Video, MFSTARTUP_FULL, MFShutdown, MFStartup, MFVideoFormat_RGB32,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToUInt64;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::core::{GUID, HSTRING};

use super::screen::{PIXEL, Rgb};

/// Media Foundation counts time in 100 ns steps.
const STEPS_MS: u64 = 10_000;
/// Media Foundation packs a pair into the high and low halves of 64 bits.
const HALF: u32 = 32;
/// A true flag, as an attribute holds it.
const TRUE: u32 = 1;
/// How far a decoded pixel may stray from a colour in each channel, after H.264's loss.
const NEAR: u8 = 48;
/// The decoded sound's sample size, in bits and in bytes.
const BITS: u32 = 16;
const SAMPLE: usize = 2;
/// A whole, in percent.
const PCT: u64 = 100;

/// What the file holds.
#[derive(Debug)]
pub(super) struct Facts {
    pub(super) length_ms: u64,
    pub(super) video: GUID,
    pub(super) rate: (u32, u32),
    pub(super) size: (u32, u32),
    pub(super) frames: u32,
    pub(super) audio: GUID,
}

/// COM and Media Foundation on this thread, both ended when dropped.
struct Mf;

impl Mf {
    fn start() -> Self {
        // SAFETY: plain starts, each ended once when dropped.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok().unwrap();
            MFStartup(MF_VERSION, MFSTARTUP_FULL).unwrap();
        }
        Self
    }
}

impl Drop for Mf {
    fn drop(&mut self) {
        // SAFETY: both started above, on this thread.
        unsafe {
            let _ = MFShutdown();
            CoUninitialize();
        }
    }
}

/// A stream number the reader takes.
fn index(which: windows::Win32::Media::MediaFoundation::MF_SOURCE_READER_CONSTANTS) -> u32 {
    which.0.cast_unsigned()
}

/// The two halves of a packed pair.
fn halves(packed: u64) -> (u32, u32) {
    let low = u32::try_from(packed & u64::from(u32::MAX)).unwrap();
    (u32::try_from(packed >> HALF).unwrap(), low)
}

/// A reader of `path`, which decodes video to BGRA when `decode`.
fn reader(path: &Path, decode: bool) -> IMFSourceReader {
    let url = HSTRING::from(path.as_os_str());
    let mut attrs = None;
    // SAFETY: a local for Windows to fill, then live attributes and a path that outlive each call.
    unsafe {
        MFCreateAttributes(&raw mut attrs, 1).unwrap();
        let attrs = attrs.unwrap();
        attrs
            .SetUINT32(
                &MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
                u32::from(decode) * TRUE,
            )
            .unwrap();
        MFCreateSourceReaderFromURL(&url, &attrs).unwrap()
    }
}

/// The file's length, its streams' formats and the video's frame count.
pub(super) fn read(path: &Path) -> Facts {
    let _mf = Mf::start();
    let reader = reader(path, false);
    let (video, audio) = (
        index(MF_SOURCE_READER_FIRST_VIDEO_STREAM),
        index(MF_SOURCE_READER_FIRST_AUDIO_STREAM),
    );
    // SAFETY: a live reader; each answer is a plain value or a media type we hold.
    unsafe {
        let whole = index(MF_SOURCE_READER_MEDIASOURCE);
        let length = reader
            .GetPresentationAttribute(whole, &MF_PD_DURATION)
            .unwrap();
        let length_ms = PropVariantToUInt64(&raw const length).unwrap() / STEPS_MS;
        let shown = reader.GetNativeMediaType(video, 0).unwrap();
        let heard = reader.GetNativeMediaType(audio, 0).unwrap();
        Facts {
            length_ms,
            video: shown.GetGUID(&MF_MT_SUBTYPE).unwrap(),
            rate: halves(shown.GetUINT64(&MF_MT_FRAME_RATE).unwrap()),
            size: halves(shown.GetUINT64(&MF_MT_FRAME_SIZE).unwrap()),
            frames: samples(&reader, video, u32::MAX).0,
            audio: heard.GetGUID(&MF_MT_SUBTYPE).unwrap(),
        }
    }
}

/// Reads `stream` alone up to sample `upto`; the count read and the last sample.
fn samples(reader: &IMFSourceReader, stream: u32, upto: u32) -> (u32, Option<IMFSample>) {
    each(reader, stream, upto, |_| {})
}

/// Reads `stream` alone up to sample `upto`, handing each sample to `seen`; the count read and the last sample.
fn each(
    reader: &IMFSourceReader,
    stream: u32,
    upto: u32,
    mut seen: impl FnMut(&IMFSample),
) -> (u32, Option<IMFSample>) {
    let (mut count, mut last) = (0, None);
    // SAFETY: a live reader and locals for it to fill.
    unsafe {
        reader
            .SetStreamSelection(index(MF_SOURCE_READER_ALL_STREAMS), false)
            .unwrap();
        reader.SetStreamSelection(stream, true).unwrap();
        while count < upto {
            let (mut flags, mut sample) = (0_u32, None);
            reader
                .ReadSample(
                    stream,
                    0,
                    None,
                    Some(&raw mut flags),
                    None,
                    Some(&raw mut sample),
                )
                .unwrap();
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0.cast_unsigned() != 0 {
                break;
            }
            if let Some(got) = &sample {
                seen(got);
                count += 1;
                last = sample;
            }
        }
    }
    (count, last)
}

/// The sound track decoded to 16-bit samples, as counts alone: never the sound itself.
#[derive(Debug, Default)]
pub(super) struct Track {
    /// Its sound frames, one sample per channel each.
    pub(super) frames: u64,
    /// The share of its samples that are not silent, in percent.
    pub(super) loud_pct: u64,
}

/// The sound track of `path`, decoded and counted.
pub(super) fn sound(path: &Path) -> Track {
    let _mf = Mf::start();
    let reader = reader(path, false);
    let audio = index(MF_SOURCE_READER_FIRST_AUDIO_STREAM);
    let (mut samples, mut loud) = (0_u64, 0_u64);
    // SAFETY: a live reader, a new media type of plain values, and each sample's buffer locked once and unlocked once.
    let channels = unsafe {
        let pcm: IMFMediaType = MFCreateMediaType().unwrap();
        pcm.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio).unwrap();
        pcm.SetGUID(&MF_MT_SUBTYPE, &MFAudioFormat_PCM).unwrap();
        pcm.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, BITS).unwrap();
        reader.SetCurrentMediaType(audio, None, &pcm).unwrap();
        let current = reader.GetCurrentMediaType(audio).unwrap();
        each(&reader, audio, u32::MAX, |sample| {
            let buffer = sample.ConvertToContiguousBuffer().unwrap();
            let (mut data, mut len) = (std::ptr::null_mut(), 0_u32);
            buffer
                .Lock(&raw mut data, None, Some(&raw mut len))
                .unwrap();
            let bytes = std::slice::from_raw_parts(data, usize::try_from(len).unwrap());
            for s in bytes.as_chunks::<SAMPLE>().0 {
                samples += 1;
                loud += u64::from(i16::from_le_bytes(*s) != 0);
            }
            buffer.Unlock().unwrap();
        });
        current.GetUINT32(&MF_MT_AUDIO_NUM_CHANNELS).unwrap()
    };
    Track {
        frames: samples / u64::from(channels.max(1)),
        loud_pct: loud * PCT / samples.max(1),
    }
}

/// How many pixels of the centre half of frame `at`, decoded, are near each of `colours`, and how many it holds.
pub(super) fn centre(path: &Path, at: u32, colours: &[Rgb]) -> (Vec<usize>, usize) {
    let _mf = Mf::start();
    let reader = reader(path, true);
    let video = index(MF_SOURCE_READER_FIRST_VIDEO_STREAM);
    // SAFETY: a live reader, a new media type of plain values, and a sample's buffer locked once and unlocked once.
    unsafe {
        let bgra: IMFMediaType = MFCreateMediaType().unwrap();
        bgra.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video).unwrap();
        bgra.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32).unwrap();
        reader.SetCurrentMediaType(video, None, &bgra).unwrap();
        let (w, h) = halves(
            reader
                .GetCurrentMediaType(video)
                .unwrap()
                .GetUINT64(&MF_MT_FRAME_SIZE)
                .unwrap(),
        );
        let sample = samples(&reader, video, at).1.unwrap();
        let buffer = sample.ConvertToContiguousBuffer().unwrap();
        let (mut data, mut len) = (std::ptr::null_mut(), 0_u32);
        buffer
            .Lock(&raw mut data, None, Some(&raw mut len))
            .unwrap();
        let bytes = std::slice::from_raw_parts(data, usize::try_from(len).unwrap());
        let found = near(bytes, (w, h), colours);
        buffer.Unlock().unwrap();
        found
    }
}

/// Counts the pixels of the centre half of a `size` frame in `bytes` near each colour; the centre is the same whichever
/// way its rows run.
fn near(bytes: &[u8], size: (u32, u32), colours: &[Rgb]) -> (Vec<usize>, usize) {
    let (w, h) = (
        usize::try_from(size.0).unwrap(),
        usize::try_from(size.1).unwrap(),
    );
    let stride = bytes.len() / h.max(1);
    let (mut found, mut total) = (vec![0; colours.len()], 0);
    for row in bytes.chunks_exact(stride).skip(h / 4).take(h / 2) {
        for pixel in row[w / 4 * PIXEL..w * 3 / 4 * PIXEL].as_chunks::<PIXEL>().0 {
            total += 1;
            for (n, colour) in found.iter_mut().zip(colours) {
                *n += usize::from(colour.near(*pixel, NEAR));
            }
        }
    }
    (found, total)
}

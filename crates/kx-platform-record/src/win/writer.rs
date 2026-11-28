//! Media Foundation's Sink Writer: an MP4 file with an H.264 video stream and an AAC sound stream, each sample timed
//! by its slot or its first sound frame. The system's hardware encoder may serve it.

use std::path::Path;

use windows::Win32::Media::MediaFoundation::{
    IMFDXGIDeviceManager, IMFMediaBuffer, IMFMediaType, IMFSinkWriter,
    MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, MF_SINK_WRITER_D3D_MANAGER, MFCreateAttributes,
    MFCreateMemoryBuffer, MFCreateSample, MFCreateSinkWriterFromURL,
};
use windows::core::{GUID, HSTRING};

use super::kinds::{audio, video};
use super::refused;
use crate::record::{CHANNELS, RATE, RecordFault, Shape};
use crate::table::Table;

/// Media Foundation counts time in 100 ns steps.
pub(super) const STEPS_S: u64 = 10_000_000;
/// A true flag, as an attribute holds it.
const TRUE: u32 = 1;
/// The writer's settings: the hardware switch and the device manager.
const SETTINGS: u32 = 2;

/// An MP4 file being written, with its video and sound streams.
pub(super) struct Writer {
    sink: IMFSinkWriter,
    video: u32,
    audio: u32,
    fps: u64,
}

impl Writer {
    /// Starts writing `path` as `shape` says, its video made from frames of `kind`, read through `manager` when given.
    #[allow(unsafe_code, reason = "Starts a writer of ours.")]
    pub(super) fn open(
        path: &Path,
        shape: &Shape,
        kind: GUID,
        manager: Option<&IMFDXGIDeviceManager>,
        table: &Table,
    ) -> Result<Self, RecordFault> {
        let writer = writer(path, manager).map_err(refused)?;
        let made = || -> windows::core::Result<(u32, u32)> {
            let coded = video(shape, kind, Some(table.video.bits_s))?;
            let v = stream(&writer, &coded, &video(shape, kind, None)?)?;
            let a = stream(
                &writer,
                &audio(Some(table.sound.aac_bytes_s))?,
                &audio(None)?,
            )?;
            // SAFETY: a live writer whose streams are set.
            unsafe { writer.BeginWriting()? };
            Ok((v, a))
        };
        let (video, audio) = made().map_err(refused)?;
        Ok(Self {
            sink: writer,
            video,
            audio,
            fps: u64::from(shape.fps.max(1)),
        })
    }

    /// Writes `buffer` as frame `slot`.
    pub(super) fn frame(&self, slot: u64, buffer: &IMFMediaBuffer) -> Result<(), RecordFault> {
        let (at, end) = (steps(slot, self.fps), steps(slot + 1, self.fps));
        self.send(self.video, buffer, at, end - at)
    }

    /// Writes stereo `pcm` that begins at sound frame `first`.
    pub(super) fn sound(&self, first: u64, pcm: &[i16]) -> Result<(), RecordFault> {
        if pcm.is_empty() {
            return Ok(());
        }
        let bytes: Vec<u8> = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
        let buffer = memory(&bytes).map_err(refused)?;
        let frames = u64::try_from(pcm.len() / usize::from(CHANNELS)).unwrap_or_default();
        let rate = u64::from(RATE);
        let (at, end) = (steps(first, rate), steps(first + frames, rate));
        self.send(self.audio, &buffer, at, end - at)
    }

    /// Ends the file, whole and playable.
    #[allow(unsafe_code, reason = "Ends a writer of ours.")]
    pub(super) fn finish(&self) -> Result<(), RecordFault> {
        // SAFETY: a live writer, ended once.
        unsafe { self.sink.Finalize() }.map_err(refused)
    }

    /// Writes `buffer` to `stream` as one sample from `at`, `len` long, both in 100 ns steps.
    #[allow(unsafe_code, reason = "Writes a sample of ours.")]
    fn send(
        &self,
        stream: u32,
        buffer: &IMFMediaBuffer,
        at: u64,
        len: u64,
    ) -> Result<(), RecordFault> {
        let write = || -> windows::core::Result<()> {
            // SAFETY: a new sample holding a live buffer; the writer keeps it as long as it needs.
            unsafe {
                let sample = MFCreateSample()?;
                sample.AddBuffer(buffer)?;
                sample.SetSampleTime(i64::try_from(at).unwrap_or(i64::MAX))?;
                sample.SetSampleDuration(i64::try_from(len).unwrap_or(i64::MAX))?;
                self.sink.WriteSample(stream, &sample)
            }
        };
        write().map_err(refused)
    }
}

/// A buffer in memory holding a copy of `bytes`; empty bytes are refused, as the writer never sends any.
#[allow(unsafe_code, reason = "Fills a buffer of ours.")]
pub(super) fn memory(bytes: &[u8]) -> windows::core::Result<IMFMediaBuffer> {
    let size = u32::try_from(bytes.len())
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(windows::core::Error::empty)?;
    // SAFETY: a new buffer of `size` bytes, above 0, so its lock gives memory to copy exactly that many into.
    unsafe {
        let buffer = MFCreateMemoryBuffer(size)?;
        let mut to = std::ptr::null_mut();
        buffer.Lock(&raw mut to, None, None)?;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), to, bytes.len());
        buffer.Unlock()?;
        buffer.SetCurrentLength(size)?;
        Ok(buffer)
    }
}

/// `n` units of `per` a second, in 100 ns steps.
fn steps(n: u64, per: u64) -> u64 {
    n.saturating_mul(STEPS_S) / per.max(1)
}

/// A Sink Writer for `path` that may use the hardware encoder and reads frames through `manager` when given.
#[allow(unsafe_code, reason = "Makes a writer of ours.")]
fn writer(
    path: &Path,
    manager: Option<&IMFDXGIDeviceManager>,
) -> windows::core::Result<IMFSinkWriter> {
    let url = HSTRING::from(path.as_os_str());
    let mut attrs = None;
    // SAFETY: a local for Windows to fill, then live attributes, manager and path that outlive each call.
    unsafe {
        MFCreateAttributes(&raw mut attrs, SETTINGS)?;
        let attrs = attrs.ok_or_else(windows::core::Error::empty)?;
        attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, TRUE)?;
        if let Some(manager) = manager {
            attrs.SetUnknown(&MF_SINK_WRITER_D3D_MANAGER, manager)?;
        }
        MFCreateSinkWriterFromURL(&url, None, &attrs)
    }
}

/// Adds a stream that writes `out` from `input`, and hands back its number.
#[allow(unsafe_code, reason = "Adds a stream to a writer of ours.")]
fn stream(
    writer: &IMFSinkWriter,
    out: &IMFMediaType,
    input: &IMFMediaType,
) -> windows::core::Result<u32> {
    // SAFETY: a live writer and two live media types.
    unsafe {
        let index = writer.AddStream(out)?;
        writer.SetInputMediaType(index, input, None)?;
        Ok(index)
    }
}

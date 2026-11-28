//! The microphone and what the PC plays (WASAPI loopback: an input stream on the default output device) through
//! `cpal`, each chunk made stereo at the track's rate and queued with its capture time from the stream's timestamps.
//! A gap the stream goes on through (cpal's xrun) is counted; any other error fails it.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device, ErrorKind, FromSample, InputCallbackInfo, SampleFormat, SizedSample, Stream,
    StreamConfig, SupportedStreamConfig,
};

use crate::record::stereo::Stereo;
use crate::record::{CHANNELS, Ear, Packet, RATE, RecordFault, Switch};
use crate::table::Table;

#[cfg(test)]
mod tests;

/// Milliseconds in a second.
const MS_S: usize = 1_000;

/// How a device's default format is read.
type Config = fn(&Device) -> Result<SupportedStreamConfig, cpal::Error>;

/// Packets waiting, with the frames they hold, at most `keep`.
#[derive(Default)]
struct Queue {
    packets: VecDeque<Packet>,
    frames: usize,
}

/// What a stream shares with its callbacks: its packets, the gaps it went on through and whether it failed.
#[derive(Default)]
struct Shared {
    queue: Mutex<Queue>,
    gaps: AtomicU32,
    failed: AtomicBool,
}

/// An open source, heard until dropped.
struct WinEar {
    shared: Arc<Shared>,
    _stream: Stream,
}

/// Opens `which` with `table`'s downmix and queue.
///
/// # Errors
/// `NoMic` or `NoSound` with no device, `MicRefused` or `NoSound` when its stream will not start (edge 11).
pub(super) fn open(which: Switch, table: &Table) -> Result<Box<dyn Ear>, RecordFault> {
    let host = cpal::default_host();
    let (device, config, missing, refused): (_, Config, _, _) = match which {
        Switch::Mic => (
            host.default_input_device(),
            DeviceTrait::default_input_config,
            RecordFault::NoMic,
            RecordFault::MicRefused,
        ),
        Switch::Sound => (
            host.default_output_device(),
            DeviceTrait::default_output_config,
            RecordFault::NoSound,
            RecordFault::NoSound,
        ),
    };
    let device = device.ok_or(missing)?;
    let shared = Arc::new(Shared::default());
    let stream = start(&device, config, table, &shared).ok_or(refused)?;
    Ok(Box::new(WinEar {
        shared,
        _stream: stream,
    }))
}

impl Ear for WinEar {
    fn heard(&mut self) -> Vec<Packet> {
        let mut queue = lock(&self.shared.queue);
        queue.frames = 0;
        queue.packets.drain(..).collect()
    }

    fn failed(&self) -> bool {
        self.shared.failed.load(Ordering::Relaxed)
    }
}

impl Shared {
    /// Counts a gap (cpal's xrun), which the stream goes on through; any other error is a failure.
    fn note(&self, error: &cpal::Error) {
        if error.kind() == ErrorKind::Xrun {
            self.gaps.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    /// Queues `packet`, dropping the oldest ones past `keep` frames.
    fn put(&self, packet: Packet, keep: usize) {
        let mut queue = lock(&self.queue);
        queue.frames += packet.samples.len() / usize::from(CHANNELS);
        queue.packets.push_back(packet);
        while queue.frames > keep {
            let Some(old) = queue.packets.pop_front() else {
                break;
            };
            queue.frames -= old.samples.len() / usize::from(CHANNELS);
        }
    }
}

/// The queue, even after a callback panicked holding it.
fn lock(queue: &Mutex<Queue>) -> MutexGuard<'_, Queue> {
    queue.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Starts `device` as an input in the format `config` reads; `None` when it would not.
fn start(device: &Device, config: Config, table: &Table, shared: &Arc<Shared>) -> Option<Stream> {
    let supported = config(device).ok()?;
    let cfg = supported.config();
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(device, cfg, table, shared),
        SampleFormat::I16 => build::<i16>(device, cfg, table, shared),
        SampleFormat::I32 => build::<i32>(device, cfg, table, shared),
        SampleFormat::U16 => build::<u16>(device, cfg, table, shared),
        _ => None,
    }?;
    stream.play().ok()?;
    Some(stream)
}

/// An input stream that makes each chunk stereo at the track's rate and queues it with when it was captured.
fn build<T>(
    device: &Device,
    cfg: StreamConfig,
    table: &Table,
    shared: &Arc<Shared>,
) -> Option<Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let keep = usize::try_from(table.sound.queue_ms).ok()? * usize::try_from(RATE).ok()? / MS_S;
    let mut stereo = Stereo::new(cfg.channels, cfg.sample_rate, table);
    let (data_shared, error_shared) = (Arc::clone(shared), Arc::clone(shared));
    let mut input = Vec::new();
    let data = move |data: &[T], info: &InputCallbackInfo| {
        let stamp = info.timestamp();
        let age = stamp.callback.duration_since(stamp.capture);
        let at = Instant::now().checked_sub(age).unwrap_or_else(Instant::now);
        input.clear();
        input.extend(data.iter().map(|s| s.to_sample::<f32>()));
        let samples = stereo.push(&input);
        data_shared.put(Packet { at, samples }, keep);
    };
    let error = move |e: cpal::Error| error_shared.note(&e);
    device.build_input_stream(cfg, data, error, None).ok()
}

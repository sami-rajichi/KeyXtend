//! Gate builds only: a WAV file played in place of the microphone, at its own pace, then silence until the run closes,
//! as a quiet microphone would. It is never staged, so the product never takes a sound file (P11 task 22 checks it).

use super::Feed;
use super::mic::{Format, Opened, Source};
use super::wav::{self, Sound};
use super::wipe::Buf;
use kx_ipc::VoiceFault;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// The sound one chunk holds, in ms, as a microphone's callback would.
const CHUNK_MS: u64 = 20;
/// Milliseconds in a second.
const MS_PER_S: u64 = 1000;
/// The name of the thread that plays the file.
const THREAD: &str = "voice-played";

/// The file's sound, played at each opening.
pub struct Played(Arc<Sound>);

impl Played {
    /// The source playing the WAV file at `path`.
    ///
    /// # Errors
    /// As `wav::read`.
    pub fn read(path: &Path) -> Result<Self, VoiceFault> {
        wav::read(path).map(|sound| Self(Arc::new(sound)))
    }
}

/// Stops the playing thread when dropped, as closing a microphone does.
struct Playing(Arc<AtomicBool>);

impl Drop for Playing {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl Source for Played {
    fn open(&mut self, feed: Feed) -> Result<Opened, VoiceFault> {
        let (sound, stop) = (Arc::clone(&self.0), Arc::new(AtomicBool::new(false)));
        let done = Arc::clone(&stop);
        let wait = |chunk: Duration| thread::sleep(chunk);
        thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || play(&sound, &feed, &done, wait))
            .map_err(|_| VoiceFault::Internal)?;
        let format = self.0.format;
        Ok(Opened {
            format,
            live: Box::new(Playing(stop)),
        })
    }
}

/// Hands `sound` to `feed` a chunk at a time, waiting each chunk's length, then silence, until `stop`.
fn play(sound: &Sound, feed: &Feed, stop: &AtomicBool, wait: impl Fn(Duration)) {
    let size = chunk(sound.format);
    let (mut parts, silence) = (sound.samples.chunks(size), vec![0.0_f32; size]);
    while !stop.load(Ordering::SeqCst) {
        let part = parts.next().unwrap_or(&silence);
        match Buf::collect(part.iter().copied()) {
            Ok(buf) => feed.sound(buf),
            Err(fault) => return feed.lost(fault),
        }
        wait(Duration::from_millis(CHUNK_MS));
    }
}

/// Interleaved samples in one chunk of `format`, at least one frame.
fn chunk(format: Format) -> usize {
    let frames = u64::from(format.rate) * CHUNK_MS / MS_PER_S;
    let samples = frames.max(1) * u64::from(format.channels.max(1));
    usize::try_from(samples).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::Event;
    use std::sync::mpsc;

    #[test]
    fn a_chunk_is_twenty_ms_of_whole_frames() {
        let stereo = Format {
            rate: 48_000,
            channels: 2,
        };
        assert_eq!(chunk(stereo), 1920);
        let slow = Format {
            rate: 10,
            channels: 1,
        };
        assert_eq!(chunk(slow), 1);
    }

    #[test]
    fn the_file_plays_in_order_then_silence_until_stopped() {
        let format = Format {
            rate: 100,
            channels: 1,
        };
        let samples = Buf::collect((0..5_u8).map(|k| f32::from(k) / 10.0)).unwrap();
        let sound = Sound { format, samples };
        let (tx, rx) = mpsc::channel();
        let feed = Feed::new(tx, 1);
        let stop = AtomicBool::new(false);
        let waits = std::cell::Cell::new(0);
        play(&sound, &feed, &stop, |_| {
            waits.set(waits.get() + 1);
            if waits.get() == 4 {
                stop.store(true, Ordering::SeqCst);
            }
        });
        let got: Vec<Vec<f32>> = rx
            .try_iter()
            .filter_map(|e| match e {
                Event::Sound { chunk, .. } => Some(chunk.to_vec()),
                _ => None,
            })
            .collect();
        assert_eq!(
            got,
            [vec![0.0, 0.1], vec![0.2, 0.3], vec![0.4], vec![0.0, 0.0]]
        );
    }
}

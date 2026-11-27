//! The media job's largest encode, as Snip makes it: a large frozen copy's pixels go through `WorkerPictures` to a
//! worker under its job, and the PNG comes back through the host's bounded read, its peak memory and time told so Snip
//! can size the job's cap. Noise makes the PNG its largest, past the usual reply bound, and a screen-like pattern shows
//! the usual time.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use kx_ipc::{Ask, BGRX_BYTES, MAX_RGBA_BYTES, MIB, Reply, png_most};
use kx_module_api::ports::{EncodeAsk, PictureConfig, Pictures, Shot};
use kx_platform::pictures::WorkerPictures;
use kx_platform::worker::{Heard, MediaFrames, Worker};
use kx_test_support::noise::noise;

use super::{CFG, Launch, Piped, SpawnError, Spawner, Still, WinChild, WinSpawner, hold, spawner};

/// The probe's own picture side, 8192 by 8192 raw pixels; size it to `kx-mod-snip`'s `shot_cap_mb` default.
const SIDE: u32 = 8192;
/// A cap far above the need, so the peak told is the job's own.
const MEDIA_MB: u32 = 2048;
/// The longest one encode may take here, in ms.
const ENCODE_MS: u32 = 120_000;
/// Bands of a screen-like pattern: flat runs a few pixels wide, as windows and text make.
const BAND: u32 = 16;

/// What one encode told: the PNG's length, the time and the job's peak memory in MB.
struct Told {
    png_len: usize,
    took: Duration,
    peak_mb: Option<u32>,
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "encodes 256 MiB of pixels twice in the worker; opt-in"
)]
fn the_largest_default_encode_goes_through_the_host_and_tells_its_peak_memory() {
    let _live = hold();
    let raw_mb = (SIDE as usize).pow(2) * BGRX_BYTES / MIB;
    let cases = [
        ("noise", noisy as fn(u32, u32) -> u32, true),
        ("screen", screen, false),
    ];
    for (name, colour, past_usual) in cases {
        let Told {
            png_len,
            took,
            peak_mb,
        } = encode(colour);
        let png_mb = png_len / MIB;
        eprintln!(
            "encode {name}: {raw_mb} MiB raw, PNG {png_mb} MiB in {took:?}, peak {peak_mb:?} MB"
        );
        assert!(peak_mb.is_some(), "the job tells its peak");
        let past = png_len > MAX_RGBA_BYTES;
        assert_eq!(past, past_usual, "{name}: past the usual reply bound");
    }
}

/// Encodes a `SIDE` square of `colour` through a fresh `WorkerPictures` session and says what it told.
fn encode(colour: fn(u32, u32) -> u32) -> Told {
    let bgrx: Vec<u8> = (0..SIDE)
        .flat_map(|y| (0..SIDE).map(move |x| colour(x, y).to_le_bytes()))
        .flatten()
        .collect();
    let most = png_most(bgrx.len());
    let shot = Shot::new(SIDE, SIDE, bgrx).unwrap();
    let peak = Arc::new(AtomicU32::new(0));
    let spawner = Peaked {
        inner: spawner(),
        peak: Arc::clone(&peak),
    };
    let mut run = WorkerPictures::new(spawner, Arc::new(Still)).open(config());
    let start = Instant::now();
    let png = run.encode(EncodeAsk { shot }).expect("a PNG").into_inner();
    let took = start.elapsed();
    assert!(png.len() <= most);
    let peak_mb = Some(peak.load(Ordering::Relaxed)).filter(|&mb| mb > 0);
    Told {
        png_len: png.len(),
        took,
        peak_mb,
    }
}

/// The session's config: one encode's time and memory, and the whole picture's pixels.
fn config() -> PictureConfig {
    PictureConfig {
        deadline_ms: ENCODE_MS,
        memory_mb: MEDIA_MB,
        max_pixels: u64::from(SIDE).pow(2),
        ..CFG
    }
}

/// The real spawner, each worker it starts noting its job's peak memory in `peak`.
struct Peaked {
    inner: WinSpawner,
    peak: Arc<AtomicU32>,
}

impl Spawner<MediaFrames> for Peaked {
    type Worker = Watched;

    fn spawn(&self, launch: &Launch) -> Result<Watched, SpawnError> {
        let inner = Spawner::<MediaFrames>::spawn(&self.inner, launch)?;
        let peak = Arc::clone(&self.peak);
        Ok(Watched { inner, peak })
    }
}

/// A real media worker that notes its job's peak memory after each reply.
struct Watched {
    inner: Piped<WinChild, MediaFrames>,
    peak: Arc<AtomicU32>,
}

impl Worker<MediaFrames> for Watched {
    fn send(&mut self, ask: Ask<'static>) {
        self.inner.send(ask);
    }

    fn reply(&mut self, within: Duration) -> Heard<Reply> {
        let heard = self.inner.reply(within);
        if let Some(mb) = self.inner.peak_mb() {
            self.peak.fetch_max(mb, Ordering::Relaxed);
        }
        heard
    }

    fn end(&mut self) {
        self.inner.end();
    }

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.inner.code(within)
    }

    fn peak_mb(&mut self) -> Option<u32> {
        self.inner.peak_mb()
    }
}

/// The shared noise as one raw pixel, its unused byte 0.
fn noisy(across: u32, down: u32) -> u32 {
    let [blue, green, red] = noise(across, down);
    u32::from_le_bytes([blue, green, red, 0])
}

/// Flat bands with a thin edge every `BAND` pixels, as windows and text make.
fn screen(x: u32, y: u32) -> u32 {
    let edge = x.is_multiple_of(BAND) || y.is_multiple_of(BAND);
    if edge {
        0x0020_2020
    } else {
        0x00F0_F0F0 ^ ((x / BAND + y / BAND) & 0x3F)
    }
}

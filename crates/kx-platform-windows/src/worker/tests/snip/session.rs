//! The worker's `record` job as the host drives it: asks written on its stdin, notes read on a thread of their own, and
//! the test's windows kept changing between checks while a note is awaited.

use std::borrow::Cow;
use std::io::PipeWriter;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use kx_ipc::{Area, RecordAsk, RecordEnd, RecordNote};

use super::{Launch, Probing, Ran, WAIT, WinSpawner};

/// The worker's memory cap: the plan's `[snip] record_mb` default.
pub(super) const RECORD_MB: u32 = 512;
/// The longest a note may take: a run's end waits its cushion and closes the file, and a copy decodes the whole video.
const NOTE_WAIT: Duration = Duration::from_secs(30);
/// The plan's `[snip] fps` default, and the longest length and the floor each run asks: the settings' own range ends.
pub(super) const FPS: u8 = 30;
const MAX_MIN: u16 = 1;
const FLOOR_MB: u32 = 512;
/// Milliseconds in a second.
pub(super) const MS_S: u32 = 1_000;

/// A run's notes: its times, its length, and the wall time from its first time to the one it stopped at.
#[derive(Debug)]
pub(super) struct Run {
    pub(super) times: Vec<u32>,
    pub(super) length_ms: u32,
    pub(super) wall: Duration,
}

/// A worker on the record job.
pub(super) struct Session {
    probing: Probing,
    input: PipeWriter,
    notes: Receiver<RecordNote>,
}

impl Session {
    /// Starts a worker on the record job, as the host would.
    pub(super) fn start(spawner: &WinSpawner) -> Self {
        let mut probing = Probing::launch(spawner, &Launch::record(RECORD_MB));
        let (input, mut output) = probing.pipes();
        let (tx, notes) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(note) = RecordNote::read(&mut output) {
                if tx.send(note).is_err() {
                    return;
                }
            }
        });
        Self {
            probing,
            input,
            notes,
        }
    }

    /// Writes `ask`.
    pub(super) fn send(&mut self, ask: &RecordAsk<'_>) {
        ask.write(&mut self.input).unwrap();
    }

    /// The next note, calling `tick` between checks.
    pub(super) fn next(&mut self, tick: &mut dyn FnMut()) -> RecordNote {
        let end = Instant::now() + NOTE_WAIT;
        loop {
            match self.notes.try_recv() {
                Ok(note) => return note,
                Err(TryRecvError::Disconnected) => panic!("the worker ended"),
                Err(TryRecvError::Empty) => {}
            }
            assert!(Instant::now() < end, "no note came");
            tick();
        }
    }

    /// Starts run `id` of `area` into `file` with both sounds on.
    pub(super) fn start_run(&mut self, id: u32, area: Area, file: &Path) {
        self.send(&RecordAsk::Start {
            id,
            area,
            fps: FPS,
            mic: true,
            sound: true,
            max_min: MAX_MIN,
            min_free_mb: FLOOR_MB,
            file: Cow::Owned(file.to_string_lossy().into_owned()),
        });
    }

    /// Records run `id` of `area` into `file` until its time reaches `seconds`, then stops it.
    pub(super) fn record(
        &mut self,
        id: u32,
        (area, file): (Area, &Path),
        seconds: u32,
        tick: &mut dyn FnMut(),
    ) -> Run {
        self.start_run(id, area, file);
        let (mut times, mut first, mut wall) = (Vec::new(), None, Duration::ZERO);
        loop {
            match self.next(tick) {
                RecordNote::Time { ms, .. } => {
                    let now = Instant::now();
                    let first = *first.get_or_insert(now);
                    times.push(ms);
                    if ms == seconds * MS_S {
                        wall = now - first;
                        self.send(&RecordAsk::Stop { id });
                    }
                }
                RecordNote::End {
                    length_ms,
                    end: RecordEnd::Stopped,
                    ..
                } => {
                    return Run {
                        times,
                        length_ms,
                        wall,
                    };
                }
                other => panic!("run {id}: {other:?}"),
            }
        }
    }

    /// Writes copy `id` of `from` into `to`, `times` faster, and hands back its length.
    pub(super) fn speed(&mut self, id: u32, from: &Path, times: u8, to: &Path) -> u32 {
        let path = |p: &Path| Cow::Owned(p.to_string_lossy().into_owned());
        self.send(&RecordAsk::Speed {
            id,
            from: path(from),
            speed: times,
            to: path(to),
        });
        match self.next(&mut super::screen::frame) {
            RecordNote::End {
                length_ms,
                end: RecordEnd::Stopped,
                ..
            } => length_ms,
            other => panic!("copy {id}: {other:?}"),
        }
    }

    /// Says goodbye and waits for the worker's end.
    pub(super) fn bye(mut self) -> Ran {
        self.send(&RecordAsk::Bye);
        self.probing.finish(WAIT)
    }
}

//! A scripted studio for tests: a clock that jumps to each wait's end, a screen that gives a frame stamped with when it
//! was made, sound sources whose samples follow a script, and a file sink that logs what it was given. Its clock may be
//! gated, so a test on another thread lets the run on slot by slot, and it posts timed orders, so nothing sleeps.

mod ear;
mod film;
mod reel;

use std::any::Any;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

pub use ear::{PACKET, Voice};
pub use reel::Tape;

use crate::record::{Ear, Film, Order, Plan, RecordFault, Reel, Shape, Sink, Studio, Switch};
use film::frames;

/// A working script's microphone level.
pub const MIC: f32 = 0.25;
/// A working script's PC sound level.
pub const SOUND: f32 = 0.125;
/// A working script's room, far above any floor, in MiB.
pub const ROOM_MB: u64 = 1 << 20;

/// What the fakes do, set by the test before the run.
#[derive(Clone)]
pub struct Script {
    /// What `ready` answers: `Ok`, or `NoVideo` as on a system without Media Foundation.
    pub ready: Result<(), RecordFault>,
    /// What opening the capture and the file answers.
    pub film: Result<(), RecordFault>,
    /// The drive's room in MiB, from each time on, latest first wins; `None` reads as unknown.
    pub room: Vec<(Duration, Option<u64>)>,
    /// The microphone.
    pub mic: Voice,
    /// The PC's sound.
    pub sound: Voice,
    /// The screen gives no frame before this.
    pub frames_from: Duration,
    /// The frame slot whose write fails, as a broken encoder would.
    pub sink_fails_at: Option<u64>,
    /// The frame slot whose write panics, as a bug inside would.
    pub sink_panics_at: Option<u64>,
    /// How long each frame's write takes on the clock, as a slow encoder's would.
    pub frame_cost: Duration,
    /// A frame slot whose write takes this long more on the clock, as a PC asleep mid-run.
    pub stall: Option<(u64, Duration)>,
    /// The video is made a folder, which no file delete removes, as a file the system still holds.
    pub held: bool,
    /// The video a copy reads.
    pub tape: Tape,
}

impl Script {
    /// Everything works: both sources steady at `MIC` and `SOUND`, `ROOM_MB` free, a frame at every grab, and `tape`
    /// for a copy to read.
    #[must_use]
    pub fn working(tape: Tape) -> Self {
        Self {
            ready: Ok(()),
            film: Ok(()),
            room: vec![(Duration::ZERO, Some(ROOM_MB))],
            mic: Voice::steady(MIC),
            sound: Voice::steady(SOUND),
            frames_from: Duration::ZERO,
            sink_fails_at: None,
            sink_panics_at: None,
            frame_cost: Duration::ZERO,
            stall: None,
            held: false,
            tape,
        }
    }
}

/// What the fakes were given, for the test to check.
#[derive(Clone, Debug, Default)]
pub struct Log {
    /// Each frame written: its slot, and when it was made.
    pub frames: Vec<(u64, Duration)>,
    /// Each frame a copy kept: its slot in the copy, and its number in the video read.
    pub kept: Vec<(u64, u64)>,
    /// The sound written, stereo frames from 0 on.
    pub sound: Vec<i16>,
    /// Sound written anywhere but where the last write ended.
    pub jumps: u32,
    /// The most samples one sound write held.
    pub widest: usize,
    /// The file was ended whole.
    pub finished: bool,
}

impl Log {
    /// Logs stereo `pcm` written from sound frame `first` on.
    fn wrote(&mut self, first: u64, pcm: &[i16]) {
        self.jumps += u32::from(first != frames(&self.sound));
        self.widest = self.widest.max(pcm.len());
        self.sound.extend_from_slice(pcm);
    }
}

/// The fakes' shared state: the script, the clock, the timed orders and the log.
pub struct World {
    /// The script.
    pub script: Script,
    /// The clock's zero.
    pub base: Instant,
    /// The time since `base`.
    pub elapsed: Duration,
    /// Orders posted once the clock reaches their time.
    pub orders: Vec<(Duration, Order)>,
    tx: Sender<Order>,
    /// What was written.
    pub log: Log,
}

/// The scripted studio; clones share one world and one gate.
#[derive(Clone)]
pub struct FakeStudio {
    world: Arc<Mutex<World>>,
    gate: Arc<Gate>,
}

/// Lets a gated clock's waits through, a given number at a time.
#[derive(Default)]
struct Gate {
    left: Mutex<Option<u64>>,
    turn: Condvar,
}

impl FakeStudio {
    /// A studio playing `script`, posting each of `orders` at its time on the receiver it hands back.
    #[must_use]
    pub fn new(script: Script, orders: Vec<(Duration, Order)>) -> (Self, Receiver<Order>) {
        let (tx, rx) = mpsc::channel();
        let world = World {
            script,
            base: Instant::now(),
            elapsed: Duration::ZERO,
            orders,
            tx,
            log: Log::default(),
        };
        let studio = Self {
            world: Arc::new(Mutex::new(world)),
            gate: Arc::default(),
        };
        (studio, rx)
    }

    /// The world, to read the log or change the script between slots.
    pub fn world(&self) -> MutexGuard<'_, World> {
        self.world.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Gates the clock: from now on each wait needs a pass `grant` gives.
    pub fn hold(&self) {
        *self
            .gate
            .left
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(0);
    }

    /// Lets `waits` more waits through a gated clock.
    pub fn grant(&self, waits: u64) {
        let mut left = self
            .gate
            .left
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *left = Some(left.unwrap_or_default().saturating_add(waits));
        self.gate.turn.notify_all();
    }

    /// Opens the gate for good.
    pub fn free(&self) {
        *self
            .gate
            .left
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
        self.gate.turn.notify_all();
    }

    /// Waits for a pass when gated.
    fn pass(&self) {
        let mut left = self
            .gate
            .left
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        while *left == Some(0) {
            left = self
                .gate
                .turn
                .wait(left)
                .unwrap_or_else(PoisonError::into_inner);
        }
        if let Some(n) = left.as_mut() {
            *n -= 1;
        }
    }
}

impl World {
    /// Moves the clock to `at` and posts the orders due by then.
    fn reach(&mut self, at: Instant) {
        self.elapsed = self.elapsed.max(at.saturating_duration_since(self.base));
        let now = self.elapsed;
        let (due, later) = self.orders.drain(..).partition(|(t, _)| *t <= now);
        self.orders = later;
        for (_, order) in due {
            let _ = self.tx.send(order);
        }
    }

    /// The drive's room now.
    fn room(&self) -> Option<u64> {
        let from = self.script.room.iter().filter(|(t, _)| *t <= self.elapsed);
        from.max_by_key(|(t, _)| *t).and_then(|(_, mb)| *mb)
    }
}

impl Studio for FakeStudio {
    type Frame = Duration;
    type Still = u64;

    fn ready(&self) -> Result<Box<dyn Any>, RecordFault> {
        self.world()
            .script
            .ready
            .map(|()| Box::new(()) as Box<dyn Any>)
    }

    fn room_mb(&self, _: &Path) -> Option<u64> {
        self.world().room()
    }

    fn ear(&self, which: Switch) -> Result<Box<dyn Ear>, RecordFault> {
        ear::open(self.clone(), which)
    }

    fn film(&self, plan: &Plan) -> Result<Film<Duration>, RecordFault> {
        film::open(self, plan)
    }

    fn reel(&self, from: &Path) -> Result<Box<dyn Reel<u64>>, RecordFault> {
        reel::open(self, from)
    }

    fn cut(&self, to: &Path, shape: Shape) -> Result<Box<dyn Sink<u64>>, RecordFault> {
        reel::cut(self, to, shape)
    }
}

//! A stream under load: a flooding worker waits once `IN_FLIGHT` of its frames wait for the stream, and a note the
//! reader took before its wait ended is told though the stream's thread, or the reader's hand-over, came after it.

use super::*;
use crate::worker::fake::Frame;
use kx_ipc::VoiceAsk;
use kx_module_api::Mono;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The notes the flooding worker sends.
const FLOOD: usize = 32;

/// What the flooding worker and the test share.
#[derive(Default)]
struct Gauge {
    /// Notes the stream finished telling.
    told: AtomicUsize,
    /// The most frames the stream's reader took beyond those told.
    worst: AtomicUsize,
}

/// A worker whose notes flood through a channel that holds none, as `Piped`'s does; it says each frame taken.
struct Gush {
    gauge: Arc<Gauge>,
    said: Sender<usize>,
}

impl Worker<VoiceFrames> for Gush {
    fn send(&mut self, _: VoiceAsk<'static>) {}

    fn reply(&mut self, _: Duration) -> crate::worker::Heard<VoiceNote> {
        crate::worker::Heard::Late
    }

    fn end(&mut self) {}

    fn code(&mut self, _: Duration) -> Option<u32> {
        None
    }
}

impl Split<VoiceFrames> for Gush {
    fn notes(&mut self) -> Receiver<Frame<VoiceFrames>> {
        let (tap, notes) = mpsc::sync_channel(0);
        let (gauge, said) = (Arc::clone(&self.gauge), self.said.clone());
        thread::spawn(move || {
            for n in 1..=FLOOD {
                if tap.send(Ok(level(FIRST))).is_err() {
                    return;
                }
                let ahead = n.saturating_sub(gauge.told.load(Ordering::SeqCst));
                gauge.worst.fetch_max(ahead, Ordering::SeqCst);
                let _ = said.send(n);
            }
        });
        notes
    }
}

/// Starts its one `Gush`; no other.
struct Gushes(Mutex<Option<Gush>>);

impl Spawner<VoiceFrames> for Gushes {
    type Worker = Gush;

    fn spawn(&self, _: &Launch) -> Result<Gush, SpawnError> {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.take().ok_or(SpawnError::Missing)
    }
}

/// A clock that says each time the stream's reader has read it, as it stamps a frame; the time is read first, so a
/// test that moves the clock once told moves it past the stamp.
struct Spied {
    hands: Hands,
    stamps: Sender<()>,
}

impl Clock for Spied {
    fn now(&self) -> Mono {
        let now = self.hands.now();
        if thread::current().name().is_some_and(|n| n.ends_with(NOTES)) {
            let _ = self.stamps.send(());
        }
        now
    }
}

/// A clock that holds the stream's reader in its second stamp, the time read, until `free` says go.
struct Held {
    hands: Hands,
    stamps: Sender<()>,
    reads: AtomicUsize,
    free: Mutex<Receiver<()>>,
}

impl Clock for Held {
    fn now(&self) -> Mono {
        let now = self.hands.now();
        if thread::current().name().is_some_and(|n| n.ends_with(NOTES)) {
            let _ = self.stamps.send(());
            if self.reads.fetch_add(1, Ordering::SeqCst) == 1 {
                let _ = self
                    .free
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .recv();
            }
        }
        now
    }
}

#[test]
fn a_flooding_worker_waits_once_its_frames_in_flight_wait_for_the_stream() {
    let (gauge, (said, taken)) = (Arc::<Gauge>::default(), mpsc::channel());
    let ((entered, enters), (go, gate)) = (mpsc::channel(), mpsc::channel::<()>());
    let counted = Arc::clone(&gauge);
    let tell = move |told: Told<VoiceNote>| {
        let _ = entered.send(told);
        let _ = gate.recv();
        counted.told.fetch_add(1, Ordering::SeqCst);
    };
    let gush = Gush {
        gauge: Arc::clone(&gauge),
        said,
    };
    let (spawner, clock) = (
        Arc::new(Gushes(Mutex::new(Some(gush)))),
        Arc::new(Hands::default()),
    );
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    assert_eq!(stream.begin(start, Wait::Off), Ok(FIRST));
    let mut pulled = 0;
    for n in 0..FLOOD {
        let told = enters.recv_timeout(LONG).expect("told in time");
        assert_eq!(told, Told::Note(level(FIRST)), "note {n}");
        while pulled < (n + IN_FLIGHT).min(FLOOD) {
            pulled = taken.recv_timeout(LONG).expect("the reader fills its room");
        }
        go.send(()).unwrap();
    }
    let worst = gauge.worst.load(Ordering::SeqCst);
    assert_eq!(
        worst, IN_FLIGHT,
        "the reader filled its room and never more"
    );
}

#[test]
fn a_note_taken_before_its_wait_ended_is_told_though_the_thread_was_held_past_it() {
    let (script, hands) = (Script::<VoiceFrames>::default(), Hands::default());
    let (stamps, stamped) = mpsc::channel();
    let clock = Arc::new(Spied {
        hands: hands.clone(),
        stamps,
    });
    let ((said, heard), (go, gate)) = (mpsc::channel(), mpsc::channel::<()>());
    let tell = move |told: Told<VoiceNote>| {
        let held = told == Told::Note(open(FIRST));
        drop(said.send(told));
        if held {
            let _ = gate.recv();
        }
    };
    let spawner = Arc::new(script.clone());
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    script.act(VoiceAct::Say(vec![open, level]));
    assert_eq!(stream.begin(start, Wait::Each(WAIT)), Ok(FIRST));
    let first = heard.recv_timeout(LONG).expect("held in its tell");
    assert_eq!(first, Told::Note(open(FIRST)));
    for _ in 0..2 {
        stamped.recv_timeout(LONG).expect("each frame stamped");
    }
    hands.advance(WAIT * 2);
    go.send(()).unwrap();
    let next = heard.recv_timeout(LONG).expect("told in time");
    assert_eq!(next, Told::Note(level(FIRST)), "not late");
}

#[test]
fn a_note_stamped_in_time_is_waited_for_though_its_reader_hands_it_over_after_the_wait() {
    let (script, hands) = (Script::<VoiceFrames>::default(), Hands::default());
    let ((stamps, stamped), (free, freed)) = (mpsc::channel(), mpsc::channel());
    let clock = Arc::new(Held {
        hands: hands.clone(),
        stamps,
        reads: AtomicUsize::new(0),
        free: Mutex::new(freed),
    });
    let ((said, heard), (go, gate)) = (mpsc::channel(), mpsc::channel::<()>());
    let tell = move |told: Told<VoiceNote>| {
        let held = told == Told::Note(open(FIRST));
        drop(said.send(told));
        if held {
            let _ = gate.recv();
        }
    };
    let spawner = Arc::new(script.clone());
    let stream = Stream::open(THREAD, spawner, clock, TIMES, launch(), tell);
    script.act(VoiceAct::Say(vec![open, level]));
    assert_eq!(stream.begin(start, Wait::Each(WAIT)), Ok(FIRST));
    let first = heard.recv_timeout(LONG).expect("held in its tell");
    assert_eq!(first, Told::Note(open(FIRST)));
    for _ in 0..2 {
        stamped.recv_timeout(LONG).expect("each frame stamped");
    }
    hands.advance(WAIT * 2);
    go.send(()).unwrap();
    // The stream's thread passes its wait while the reader still holds the stamped frame.
    stream.poke();
    free.send(()).unwrap();
    let next = heard.recv_timeout(LONG).expect("told in time");
    assert_eq!(next, Told::Note(level(FIRST)), "waited for, not late");
}

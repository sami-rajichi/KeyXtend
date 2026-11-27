//! A worker that closed its output but will not exit is waited for one deadline in all, its end included.

use super::*;

/// A child whose every wait for its exit times out, the clock moved on by the time waited.
struct Stuck {
    clock: Hands,
    waits: Sender<Duration>,
}

impl Child for Stuck {
    fn end(&mut self) {}

    fn code(&mut self, within: Duration) -> Option<u32> {
        self.clock.advance(within);
        let _ = self.waits.send(within);
        None
    }
}

/// Starts one piped worker whose output ends at once; no other.
struct Closing(Mutex<Option<Stuck>>);

impl Spawner<VoiceFrames> for Closing {
    type Worker = Piped<Stuck, VoiceFrames>;

    fn spawn(&self, _: &Launch) -> Result<Self::Worker, SpawnError> {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner).take();
        let stuck = held.ok_or(SpawnError::Missing)?;
        Piped::new(io::sink(), Cursor::new(Vec::new()), stuck)
    }
}

#[test]
fn a_closed_worker_that_will_not_exit_is_waited_for_one_deadline_in_all() {
    let clock = Hands::default();
    let (waits, waited) = mpsc::channel();
    let stuck = Stuck {
        clock: clock.clone(),
        waits,
    };
    let (stream, heard) = streamed(Closing(Mutex::new(Some(stuck))), &clock);
    stream.begin(list, Wait::Each(WAIT)).unwrap();
    assert_eq!(next(&heard, 1), [Told::Lost(Lost::Gone)]);
    let deadline = Duration::from_millis(TIMES.deadline_ms.into());
    let all: Vec<Duration> = waited.try_iter().collect();
    assert_eq!(
        all,
        [deadline, Duration::ZERO],
        "the end waits only what is left"
    );
}

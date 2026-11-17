//! Port errors become the flow's fault notes: a worker death, a refused open or call, a port that panics, a job that
//! cannot start, and the session going away. None of them panics the actor.

use std::any::Any;
use std::panic;
use std::sync::{Arc, Once};
use std::time::Duration;

use kx_module_api::SecretBytes;
use kx_module_api::ports::{
    BarHint, CheckAsk, CheckDone, Device, FactsAsk, JobLimits, ModelFacts, SessionNote, Speech,
    SpeechAsk, SpeechError, SpeechNote, SpeechRun, VoiceBar, VoiceLit,
};
use kx_platform_fake::{FakeSpeech, SpeechCall};

use super::rig::Rig;
use super::taps::{held, release, waiting};

/// The payload of a panic the test plans; the hook prints nothing for it.
struct Planned;

/// Hushes planned panics for this test binary, once.
fn hush() {
    static HUSH: Once = Once::new();
    HUSH.call_once(|| {
        let usual = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if !<dyn Any + Send>::is::<Planned>(info.payload()) {
                usual(info);
            }
        }));
    });
}

/// The fake speech service, whose runs panic at Stop and again as they drop, as a broken platform might.
struct Broken(FakeSpeech);

impl Speech for Broken {
    fn open(
        &self,
        ask: SpeechAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<Box<dyn SpeechRun>, SpeechError> {
        let run = self.0.open(ask, tell)?;
        Ok(Box::new(Snaps(run)))
    }

    fn devices(&self, job: &JobLimits) -> Result<Vec<Device>, SpeechError> {
        self.0.devices(job)
    }

    fn facts(&self, ask: &FactsAsk) -> Result<ModelFacts, SpeechError> {
        self.0.facts(ask)
    }

    fn check(
        &self,
        ask: CheckAsk,
        tell: Box<dyn Fn(SpeechNote) + Send>,
    ) -> Result<CheckDone, SpeechError> {
        self.0.check(ask, tell)
    }

    fn clips(&self) -> Vec<String> {
        self.0.clips()
    }
}

/// A fake run that panics at Stop and as it drops.
struct Snaps(Box<dyn SpeechRun>);

impl SpeechRun for Snaps {
    fn stop(&mut self) -> Result<(), SpeechError> {
        panic::panic_any(Planned)
    }

    fn cancel(&mut self) -> Result<(), SpeechError> {
        self.0.cancel()
    }

    fn again(&mut self, ask: SpeechAsk) -> Result<(), SpeechError> {
        self.0.again(ask)
    }

    fn keep(&mut self) -> Result<Vec<SecretBytes>, SpeechError> {
        self.0.keep()
    }

    fn close(&mut self) {
        self.0.close();
    }
}

impl Drop for Snaps {
    fn drop(&mut self) {
        panic::panic_any(Planned)
    }
}

/// The note the bar shows, if any.
fn note(rig: &Rig) -> Option<BarHint> {
    match rig.bar() {
        VoiceBar::Note(note) => Some(note.hint),
        _ => None,
    }
}

#[test]
fn a_worker_death_tells_its_note_closes_the_run_and_puts_the_light_out() {
    let mut rig = Rig::new();
    rig.listen();
    assert!(rig.fake.speech.die());
    rig.pump();
    assert_eq!(note(&rig), Some(BarHint::Fault));
    assert_eq!(
        rig.lits(),
        [VoiceLit::Off, VoiceLit::Prep, VoiceLit::Rec, VoiceLit::Off]
    );
    assert!(rig.fake.speech.runs()[0].closed);
}

#[test]
fn a_refused_open_tells_its_note_and_never_turns_red() {
    let mut rig = Rig::new();
    let retry_in = Duration::from_secs(1);
    rig.fake
        .speech
        .fail(SpeechCall::Open, SpeechError::Down { retry_in });
    rig.key();
    assert_eq!(note(&rig), Some(BarHint::Fault));
    let lits = [VoiceLit::Off, VoiceLit::Prep, VoiceLit::Off];
    assert_eq!(rig.lits(), lits, "amber, never red");
    assert!(rig.fake.speech.runs().is_empty());
}

#[test]
fn a_failed_stop_tells_its_fault() {
    let mut rig = Rig::new();
    rig.listen();
    rig.fake.speech.fail(SpeechCall::Stop, SpeechError::Gone);
    rig.key();
    assert_eq!(note(&rig), Some(BarHint::Fault));
    assert!(rig.fake.speech.runs()[0].closed);
}

#[test]
fn a_port_that_panics_in_a_call_tells_a_fault_and_frees_the_run() {
    hush();
    let mut rig = Rig::new();
    rig.actor.ports.speech = Arc::new(Broken(rig.fake.speech.clone()));
    rig.listen();
    rig.key();
    assert_eq!(note(&rig), Some(BarHint::Fault));
    assert_eq!(rig.lits().last(), Some(&VoiceLit::Off));
    assert!(
        rig.fake.speech.runs()[0].closed,
        "the run closed as it dropped"
    );
}

#[test]
fn a_job_that_cannot_start_tells_a_fault() {
    let mut rig = Rig::new();
    rig.actor.ports.spawn = |_| false;
    rig.key();
    assert_eq!(note(&rig), Some(BarHint::Fault));
    assert!(rig.fake.speech.runs().is_empty());
}

#[test]
fn calls_wait_for_the_open_and_go_in_order() {
    let mut rig = Rig::new();
    rig.actor.ports.spawn = held;
    rig.key();
    rig.key();
    assert!(rig.fake.speech.runs().is_empty(), "the open still runs");
    while waiting() {
        release();
        rig.pump();
    }
    let run = &rig.fake.speech.runs()[0];
    assert!(
        run.cancels == 1 && run.closed,
        "nothing was heard, so Stop cancels and closes"
    );
}

#[test]
fn a_lock_stops_the_dictation_and_wipes_the_recording() {
    let mut rig = Rig::new();
    rig.listen();
    assert!(rig.fake.session.tell(SessionNote::Locked));
    rig.pump();
    let run = &rig.fake.speech.runs()[0];
    assert!(run.closed && !run.held);
    assert_eq!(
        (rig.bar(), rig.lits().last()),
        (VoiceBar::Hidden, Some(&VoiceLit::Off))
    );
}

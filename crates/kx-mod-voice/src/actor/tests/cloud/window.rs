//! The window the stop flag guards (ADR-0047): a Cancel that lands while the job sits between the key read and `send`
//! sends nothing, and a service run that arrives after its upload was dropped is cancelled at once.

use kx_module_api::ports::{BarButton, SpeechNote, VoiceClick};
use kx_platform_fake::{CloudAnswer, CloudState};

use super::super::taps::{held, release, waiting};
use super::{Rig, done, groq};

/// Groq on with its key; at Stop the run hands its speech over, and the job that reads the key waits to be released.
fn waits() -> Rig {
    let mut rig = Rig::build(groq);
    rig.fake.cloud.script(vec![CloudAnswer::text("hello")]);
    rig.talk();
    rig.key();
    rig.actor.ports.spawn = held;
    rig.tell(SpeechNote::Done(done(0)));
    to_send(&mut rig);
    rig
}

/// Ends the held jobs and hands their answers to the actor: the hand-over first, then the job that reads the key waits.
pub(super) fn to_send(rig: &mut Rig) {
    release();
    rig.pump();
    assert!(
        waiting() && rig.fake.cloud.calls().runs.is_empty(),
        "the job waits for send"
    );
}

#[test]
fn a_cancel_while_the_job_waits_for_send_sends_nothing() {
    let mut rig = waits();
    rig.act(VoiceClick::Bar(BarButton::Cancel));
    release();
    rig.pump();
    assert!(
        rig.fake.cloud.calls().runs.is_empty(),
        "the stopped job sends nothing"
    );
    assert!(rig.asks().is_empty() && rig.actor.net.idle());
}

#[test]
fn a_run_that_arrives_after_its_upload_was_dropped_is_cancelled_at_once() {
    let mut rig = waits();
    release();
    rig.act(VoiceClick::Bar(BarButton::Cancel));
    let run = rig.fake.cloud.calls().runs[0].clone();
    assert_eq!((run.state, run.cancelled()), (CloudState::Dropped, 1));
    assert!(rig.asks().is_empty() && rig.actor.net.idle());
}

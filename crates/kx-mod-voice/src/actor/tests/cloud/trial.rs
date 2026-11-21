//! Compare's cloud models through the actor (edge 12): the recording goes only to the ticked services, through the
//! same port and with the key read in the job; an unticked ready service gets nothing; Cancel or leaving drops the
//! runs, before `send` too; and a fault shows on that model alone.

use kx_module_api::ports::{
    CompareSheet, PanelButton, RunStep, SpeechNote, SpeechStage, VoiceClick, VoiceSheet, VoiceTab,
};
use kx_platform_fake::SpeechCall;

use super::window::to_send;
use super::*;
use crate::actor::Msg;
use crate::actor::tests::taps::{held, release};
use crate::panel::rig::ROWS;

/// Custom with its typed address and model, as a second ready service.
const CUSTOM: &str = r#"custom = { on = 2026-11-01, terms = 1, address = "https://speech.example.com/v1/audio", model = "whisper-1" }"#;

/// Groq and Custom on with their keys, on Compare's page with the models at `ticks` ticked and the recording stopped.
fn ticked(ticks: &[u8]) -> Rig {
    let mut rig = Rig::build(|fake, host| {
        host.put("cloud", true.into());
        let both = format!("groq = {{ on = 2026-11-01, terms = 1 }}\n{CUSTOM}");
        host.put("services", table(&both));
        set_key(fake, &key());
        let keys = Keys::new(Arc::new(fake.shelf.clone()), false);
        keys.set("custom", secret(&key())).unwrap();
    });
    rig.send(Msg::Open);
    rig.act(VoiceClick::Rows(ROWS));
    rig.act(VoiceClick::Open(VoiceTab::Compare));
    for at in ticks {
        rig.act(VoiceClick::Row(*at));
    }
    rig.fake
        .speech
        .on(SpeechCall::Open, vec![SpeechNote::Stage(SpeechStage::Open)]);
    rig.fake.speech.set_speech(vec![vec![0; PIECE]; 2]);
    rig.act(VoiceClick::Compare(Redacted::new("hello world".into())));
    rig.tell(SpeechNote::Heard);
    rig.key();
    rig
}

/// The run hands its recording over.
fn hand(rig: &mut Rig) {
    rig.tell(SpeechNote::Done(done(0)));
}

/// As `ticked`, then handed over.
fn compared(ticks: &[u8]) -> Rig {
    let mut rig = ticked(ticks);
    hand(&mut rig);
    rig
}

/// The runs opened per service host: Groq's, then Custom's.
fn per_host(rig: &Rig) -> (usize, usize) {
    let runs = rig.fake.cloud.calls().runs;
    let on = |host: &str| runs.iter().filter(|r| r.url.host() == host).count();
    (on("api.groq.com"), on("speech.example.com"))
}

/// Each model's step on the running page.
fn steps(rig: &Rig) -> Vec<RunStep> {
    let panel = rig.views().last().and_then(|v| v.panel.clone()).unwrap();
    match panel.sheet {
        VoiceSheet::Compare(CompareSheet::Running { rows, .. }) => {
            rows.iter().map(|r| r.step).collect()
        }
        other => panic!("not running: {other:?}"),
    }
}

#[test]
fn only_the_ticked_services_get_the_recording_and_an_unticked_ready_one_gets_nothing() {
    let rig = compared(&[0, 1]);
    assert_eq!(per_host(&rig), (2, 0), "both Groq models, never Custom");
    let none = compared(&[]);
    assert_eq!(per_host(&none), (0, 0), "no tick, nothing sent");
}

#[test]
fn cancel_or_leaving_compare_drops_the_runs() {
    let mut cancelled = compared(&[0]);
    let mut left = compared(&[0]);
    assert_eq!(
        cancelled.fake.cloud.calls().runs[0].state,
        CloudState::Sending
    );
    cancelled.act(VoiceClick::Button(PanelButton::Cancel));
    left.act(VoiceClick::Button(PanelButton::Back));
    for rig in [&cancelled, &left] {
        assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Dropped);
    }
}

#[test]
fn a_cloud_fault_shows_on_that_model_alone() {
    let mut rig = compared(&[0, 1]);
    assert!(
        rig.fake
            .cloud
            .answer(0, vec![CloudAnswer::Fault(CloudFault::Credit)])
    );
    rig.pump();
    let steps = steps(&rig);
    assert_eq!(
        steps[steps.len() - 2..],
        [RunStep::Failed, RunStep::Writing]
    );
}

#[test]
fn a_key_removed_before_the_hand_over_sends_nothing_and_that_model_fails() {
    let mut rig = ticked(&[0]);
    Keys::new(Arc::new(rig.fake.shelf.clone()), false)
        .remove("groq")
        .unwrap();
    hand(&mut rig);
    assert!(rig.fake.cloud.calls().runs.is_empty(), "no key, no request");
    assert_eq!(steps(&rig).last(), Some(&RunStep::Failed));
}

#[test]
fn a_cancel_while_a_models_job_waits_for_send_sends_nothing_and_a_late_run_is_cancelled() {
    let mut waits = ticked(&[0]);
    waits.actor.ports.spawn = held;
    hand(&mut waits);
    to_send(&mut waits);
    waits.act(VoiceClick::Button(PanelButton::Cancel));
    release();
    waits.pump();
    assert!(
        waits.fake.cloud.calls().runs.is_empty(),
        "the stopped job sends nothing"
    );
    let mut late = ticked(&[0]);
    late.actor.ports.spawn = held;
    hand(&mut late);
    to_send(&mut late);
    release();
    late.act(VoiceClick::Button(PanelButton::Cancel));
    assert_eq!(late.fake.cloud.calls().runs[0].state, CloudState::Dropped);
}

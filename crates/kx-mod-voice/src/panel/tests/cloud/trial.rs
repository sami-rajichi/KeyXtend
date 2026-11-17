//! Compare's cloud models on the panel (edge 12): the ready ones to tick with the send warning, Start taking only the
//! ticked ones, none while cloud is off or the run is hidden, and a cloud result named by its service that Use picks.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, CloudNote, CompareCloud, CompareSheet, PanelButton, ServiceName, SpeechDone,
    SpeechNote, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::{apply, put};
use crate::compare::{Act, Remote};
use crate::settings::PICKS;

/// Groq on and keyed, cloud on, on the Compare page.
fn on_compare() -> Rig {
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    put(
        &mut rig,
        "services",
        "groq = { on = 2026-11-01, terms = 1 }",
    );
    rig.keyed.insert("groq".into());
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    rig
}

/// The ready page's cloud models and send warning.
fn ready(rig: &mut Rig) -> (Vec<CompareCloud>, Vec<ServiceName>) {
    match rig.view().unwrap().sheet {
        VoiceSheet::Compare(CompareSheet::Ready { cloud, send, .. }) => (cloud, send),
        other => panic!("not ready: {other:?}"),
    }
}

/// Start with a set text; the run's id.
fn start(rig: &mut Rig) -> u32 {
    let out = rig.click(&VoiceClick::Compare(Redacted::new("hello world".into())));
    match out[..] {
        [Do::Compare(Act::Open { take, .. })] => take,
        _ => panic!("no start: {out:?}"),
    }
}

/// The recording's end, `wait_ms` after Stop.
fn done(wait_ms: u32) -> SpeechNote {
    SpeechNote::Done(SpeechDone {
        pieces: 1,
        sound_ms: 4_000,
        wait_ms,
        load_ms: 0,
        memory_mb: 0,
        card: false,
    })
}

/// Groq's Whisper Large v3 Turbo as Compare sends to it.
fn turbo() -> Remote {
    Remote {
        name: Redacted::new("Whisper Large v3 Turbo".into()),
        shown: ServiceName::Named("Groq".into()),
        service: "groq".into(),
        model: "whisper-large-v3-turbo".into(),
    }
}

#[test]
fn start_takes_only_the_ticked_ready_models_and_ticks_alone_send_nothing() {
    let mut rig = on_compare();
    assert!(
        rig.click(&VoiceClick::Row(0)).is_empty(),
        "a tick sends nothing"
    );
    assert_eq!(ready(&mut rig).1, [ServiceName::Named("Groq".into())]);
    start(&mut rig);
    let plan = rig.panel.compare.plan().unwrap();
    assert_eq!(
        plan.clouds,
        [turbo()],
        "the unticked Whisper Large v3 is not sent to"
    );
}

#[test]
fn cloud_off_or_a_hidden_run_lists_and_plans_no_cloud_model() {
    for hidden in [false, true] {
        let mut rig = on_compare();
        (rig.settings.cloud, rig.hidden) = (hidden, hidden);
        assert!(ready(&mut rig).0.is_empty(), "hidden {hidden}");
        rig.click(&VoiceClick::Row(0));
        let set = Redacted::new("hello world".into());
        let out = rig.click(&VoiceClick::Compare(set));
        assert!(out.is_empty(), "no local model and no cloud one, so no run");
        assert!(rig.panel.compare.plan().is_none());
    }
}

#[test]
fn a_cloud_result_is_named_by_its_service_and_use_picks_it() {
    let mut rig = on_compare();
    rig.click(&VoiceClick::Row(0));
    let take = start(&mut rig);
    rig.with(|p, _| p.compare_note(take, SpeechNote::Heard));
    rig.click(&VoiceClick::Bar(BarButton::Stop));
    let sent = rig.with(|p, _| p.compare_note(take, done(0)));
    assert!(matches!(&sent[..], [Do::Compare(Act::Send { to, .. })] if *to == [turbo()]));
    let text = Redacted::new(kx_module_api::Zeroizing::new("hello world".into()));
    rig.with(|p, _| p.compare_net(take, 0, CloudNote::Piece { index: 0, text }, 0));
    rig.with(|p, _| p.compare_net(take, 0, CloudNote::Done, 900));
    let VoiceSheet::Compare(CompareSheet::Results { rows, .. }) = rig.view().unwrap().sheet else {
        panic!("no results");
    };
    assert_eq!(
        (rows[0].cloud.clone(), rows[0].wait_ms),
        (Some(ServiceName::Named("Groq".into())), 900)
    );
    rig.click(&VoiceClick::Row(0));
    assert!(rig.view().unwrap().acts.contains(&PanelButton::Use));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    assert!(matches!(&out[..], [Do::Set(PICKS, _)]));
    apply(&mut rig, &out);
    assert_eq!(rig.settings.picks["en"], "groq:whisper-large-v3-turbo");
}

#[test]
fn compare_lists_the_ready_cloud_models_and_names_each_ticked_service_once_before_it_sends() {
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    put(
        &mut rig,
        "services",
        "groq = { on = 2026-11-01, terms = 1 }",
    );
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    let ready = |rig: &mut Rig| match rig.view().unwrap().sheet {
        VoiceSheet::Compare(CompareSheet::Ready { cloud, send, .. }) => (cloud, send),
        other => panic!("not ready: {other:?}"),
    };
    assert!(
        ready(&mut rig).0.is_empty(),
        "a service with no key is not offered"
    );
    rig.keyed.insert("groq".into());
    let (cloud, send) = ready(&mut rig);
    assert_eq!((cloud.len(), send.len()), (2, 0));
    rig.click(&VoiceClick::Row(0));
    rig.click(&VoiceClick::Row(1));
    let (cloud, send) = ready(&mut rig);
    assert!(cloud.iter().all(|c| c.tick));
    assert_eq!(send, [ServiceName::Named("Groq".into())]);
    rig.click(&VoiceClick::Row(0));
    rig.click(&VoiceClick::Row(1));
    assert!(ready(&mut rig).1.is_empty(), "no tick, no warning");
}

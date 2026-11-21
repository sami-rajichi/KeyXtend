//! Compare's cloud models on the panel (edge 12): the ready ones to tick with the send warning, Start taking only the
//! ticked ones, none while cloud is off or the run is hidden, a header naming the first one still writing, and a cloud
//! result named by its service that Use picks, and a failed one's reason, the bar's own.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    BarButton, CloudFault, CloudNote, CompareCloud, CompareSheet, PanelButton, ServiceName,
    SpeechNote, VoiceClick, VoiceSheet, VoiceTab,
};
use std::time::Duration;

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::{At, apply, put};
use crate::compare::tests::done;
use crate::compare::{Act, Remote};
use crate::flow::{net_why, renewal};
use crate::services::tests::shipped::{entry, first, model, pick};
use crate::settings::PICKS;

/// The first offered row on and keyed, cloud on, on the Compare page.
fn on_compare() -> Rig {
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    put(&mut rig, "services", &entry(first(), ""));
    rig.keyed.insert(first().id.clone());
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

/// The first offered row's first model as Compare sends to it.
fn turbo() -> Remote {
    let (row, listed) = (first(), model(first(), 0));
    Remote {
        name: Redacted::new(listed.name.clone()),
        shown: row.shown(),
        service: row.id.clone(),
        model: Redacted::new(listed.id.clone()),
    }
}

#[test]
fn start_takes_only_the_ticked_ready_models_and_ticks_alone_send_nothing() {
    let mut rig = on_compare();
    assert!(
        rig.click(&VoiceClick::Row(0)).is_empty(),
        "a tick sends nothing"
    );
    assert_eq!(ready(&mut rig).1, [first().shown()]);
    start(&mut rig);
    let plan = rig.panel.compare.plan().unwrap();
    assert_eq!(
        plan.clouds,
        [turbo()],
        "the unticked second model is not sent to"
    );
}

#[test]
fn while_compare_records_its_picks_and_warning_show_the_plan_and_a_click_ticks_nothing() {
    let mut rig = on_compare();
    rig.click(&VoiceClick::Row(0));
    let take = start(&mut rig);
    let planned = CompareCloud {
        name: turbo().name,
        service: turbo().shown,
        tick: true,
    };
    let want = (vec![planned], vec![first().shown()]);
    assert_eq!(ready(&mut rig), want, "while the microphone opens");
    rig.with(|p, _| p.compare_note(take, SpeechNote::Heard));
    assert!(rig.click(&VoiceClick::Row(0)).is_empty());
    assert!(rig.panel.ticks.is_empty(), "the plan is fixed at Start");
    assert_eq!(ready(&mut rig), want, "while the user reads");
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
    let sent = rig.with(|p, _| p.compare_note(take, done(0, 0)));
    assert!(matches!(&sent[..], [Do::Compare(Act::Send { to, .. })] if *to == [turbo()]));
    let text = Redacted::new(kx_module_api::Zeroizing::new("hello world".into()));
    rig.with(|p, _| p.compare_net(take, 0, CloudNote::Piece { index: 0, text }, 0, None));
    rig.with(|p, _| p.compare_net(take, 0, CloudNote::Done, 900, None));
    let VoiceSheet::Compare(CompareSheet::Results { rows, .. }) = rig.view().unwrap().sheet else {
        panic!("no results");
    };
    assert_eq!(
        (rows[0].cloud.clone(), rows[0].wait_ms),
        (Some(first().shown()), 900)
    );
    rig.click(&VoiceClick::Row(0));
    assert!(rig.view().unwrap().acts.contains(&PanelButton::Use));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    assert!(matches!(&out[..], [Do::Set(PICKS, _)]));
    apply(&mut rig, &out);
    assert_eq!(rig.settings.picks["en"], pick(first(), 0));
}

#[test]
fn compare_lists_the_ready_cloud_models_and_names_each_ticked_service_once_before_it_sends() {
    let mut rig = on_compare();
    rig.keyed.clear();
    let ready = |rig: &mut Rig| match rig.view().unwrap().sheet {
        VoiceSheet::Compare(CompareSheet::Ready { cloud, send, .. }) => (cloud, send),
        other => panic!("not ready: {other:?}"),
    };
    assert!(
        ready(&mut rig).0.is_empty(),
        "a service with no key is not offered"
    );
    rig.keyed.insert(first().id.clone());
    let (cloud, send) = ready(&mut rig);
    assert_eq!((cloud.len(), send.len()), (first().models.len(), 0));
    rig.click(&VoiceClick::Row(0));
    rig.click(&VoiceClick::Row(1));
    let (cloud, send) = ready(&mut rig);
    assert!(cloud.iter().all(|c| c.tick));
    assert_eq!(send, [first().shown()]);
    rig.click(&VoiceClick::Row(0));
    rig.click(&VoiceClick::Row(1));
    assert!(ready(&mut rig).1.is_empty(), "no tick, no warning");
}

/// The running page's header: the model it names, from 1, of how many, and its name.
fn header(rig: &mut Rig) -> (u16, u16, String) {
    match rig.view().unwrap().sheet {
        VoiceSheet::Compare(CompareSheet::Running { at, of, name, .. }) => {
            (at, of, name.expose().clone())
        }
        other => panic!("not running: {other:?}"),
    }
}

#[test]
fn the_running_header_names_the_first_cloud_model_still_writing() {
    let mut rig = on_compare();
    rig.click(&VoiceClick::Row(0));
    rig.click(&VoiceClick::Row(1));
    let take = start(&mut rig);
    rig.with(|p, _| p.compare_note(take, SpeechNote::Heard));
    rig.click(&VoiceClick::Bar(BarButton::Stop));
    rig.with(|p, _| p.compare_note(take, done(0, 0)));
    assert_eq!(header(&mut rig), (1, 2, model(first(), 0).name.clone()));
    rig.with(|p, _| p.compare_net(take, 0, CloudNote::Done, 700, None));
    assert_eq!(header(&mut rig), (2, 2, model(first(), 1).name.clone()));
}

#[test]
fn a_typed_model_id_never_prints_in_the_ticks_the_plan_or_the_send() {
    let typed = ["kx", "typed", "model", "77"].join("-");
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    let custom = format!(
        r#"custom = {{ on = 2026-11-01, terms = 1, address = "https://speech.example.com/v1/audio", model = "{typed}" }}"#
    );
    put(&mut rig, "services", &custom);
    rig.keyed.insert("custom".into());
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    rig.click(&VoiceClick::Row(0));
    let take = start(&mut rig);
    assert_eq!(rig.panel.compare.plan().unwrap().clouds.len(), 1);
    rig.with(|p, _| p.compare_note(take, SpeechNote::Heard));
    rig.click(&VoiceClick::Bar(BarButton::Stop));
    let sent = rig.with(|p, _| p.compare_note(take, done(0, 0)));
    let shown = format!("{:?}{sent:?}", rig.panel);
    assert!(!shown.contains(&typed), "{shown}");
}

/// Every cloud fault, each allowance with and without its renewal.
const FAULTS: [CloudFault; 15] = [
    CloudFault::Range("model"),
    CloudFault::Key,
    CloudFault::Credit,
    CloudFault::Account,
    CloudFault::Allowance { renews_in: None },
    CloudFault::Allowance {
        renews_in: Some(Duration::from_mins(5)),
    },
    CloudFault::Rate,
    CloudFault::TooBig,
    CloudFault::Language,
    CloudFault::Region,
    CloudFault::Empty,
    CloudFault::Strange,
    CloudFault::Network,
    CloudFault::Timeout,
    CloudFault::Gone,
];

#[test]
fn a_failed_cloud_models_reason_on_compare_is_the_bars_for_every_fault() {
    for fault in FAULTS {
        let mut rig = on_compare();
        rig.click(&VoiceClick::Row(0));
        rig.click(&VoiceClick::Row(1));
        let take = start(&mut rig);
        rig.with(|p, _| p.compare_note(take, SpeechNote::Heard));
        rig.click(&VoiceClick::Bar(BarButton::Stop));
        rig.with(|p, _| p.compare_note(take, done(0, 0)));
        let renews = rig.with(|_, f| renewal(&At(f.now), Some(&fault)));
        rig.with(|p, _| p.compare_net(take, 0, CloudNote::Fault(fault), 0, renews));
        let view = rig.view().unwrap();
        let VoiceSheet::Compare(CompareSheet::Running { rows, .. }) = view.sheet else {
            panic!("not running: {fault:?}");
        };
        assert_eq!(
            rows[0].why,
            Some(net_why(fault, &view.lang, renews)),
            "{fault:?}"
        );
    }
}

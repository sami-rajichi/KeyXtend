//! Open service through the actor (final review m5): the key note's Open service opens the panel on the key page with
//! the key's dots read, and the account note's opens the card, as the Off note's does at Start for a service Off.

use kx_module_api::ports::{CloudButton, VoiceSheet, VoiceTab};

use super::*;
use crate::actor::Msg;

/// The panel's sheet as the last view draws it; `None` while the panel is closed.
fn sheet(rig: &Rig) -> Option<VoiceSheet> {
    rig.views().last()?.panel.clone().map(|p| p.sheet)
}

#[test]
fn open_service_on_the_key_note_opens_the_key_page_with_its_dots() {
    let mut rig = Rig::build(groq);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::Fault(CloudFault::Key)]);
    rig.dictate();
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Key));
    assert_eq!(sheet(&rig), None, "the panel is closed");
    rig.act(VoiceClick::Bar(BarButton::Service));
    let Some(VoiceSheet::Service(page)) = sheet(&rig) else {
        panic!("the key page shows");
    };
    let dots = u32::try_from(key().len()).unwrap();
    assert_eq!(*page.key.expose(), dots, "the key's dots are read");
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Key), "the note stays");
}

#[test]
fn open_service_on_the_account_note_opens_the_card() {
    let mut rig = Rig::build(groq);
    let fault = CloudAnswer::Fault(CloudFault::Account);
    rig.fake.cloud.script(vec![fault]);
    rig.dictate();
    rig.act(VoiceClick::Bar(BarButton::Service));
    assert!(matches!(sheet(&rig), Some(VoiceSheet::Card(_))));
}

#[test]
fn a_service_switched_off_keeps_its_pick_so_start_says_off_and_its_confirm_brings_it_back() {
    let mut rig = Rig::build(groq);
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Service(CloudButton::Off));
    rig.send(Msg::Close);
    rig.key();
    let hint = BarHint::Cloud {
        why: CloudWhy::Off,
        local: LocalUse::Dimmed,
    };
    let off = VoiceBar::Note(BarNote {
        hint,
        act: None,
        mic: false,
    });
    assert_eq!(rig.bar(), off, "owner, 2026-11-21");
    assert!(rig.fake.speech.runs().is_empty(), "nothing listens");
    rig.act(VoiceClick::Bar(BarButton::Service));
    assert!(matches!(sheet(&rig), Some(VoiceSheet::Card(_))));
    rig.act(VoiceClick::Confirm(true));
    rig.send(Msg::Close);
    rig.listen();
    let runs = rig.fake.speech.runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].ask.model, None, "listen-only, for the service");
    assert_eq!(rig.mark(), Some(first().shown()));
}

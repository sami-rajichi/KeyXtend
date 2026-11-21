//! Open service through the actor (final review m5): the key note's Open service opens the panel on the key page with
//! the key's dots read, and the account note's opens the card.

use kx_module_api::ports::VoiceSheet;

use super::*;

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

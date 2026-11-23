//! Open service on a cloud note (final review m5): the five notes the service's own pages settle open its card, or its
//! key page for the key note, also once the dictation that showed the note ended, and at Start for a pick waiting for
//! a service Off; the other notes open nothing.

use super::*;

/// The pages the bar asks to open among `out`: each service, and whether its key page.
fn settles(out: &[Out]) -> Vec<(String, bool)> {
    let one = |o: &Out| match o {
        Out::Settle { service, key } => Some((service.clone(), *key)),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

/// Groq's key page when `key`, else its card.
fn groq_page(key: bool) -> Vec<(String, bool)> {
    vec![("groq".into(), key)]
}

#[test]
fn open_service_on_a_note_at_start_opens_the_key_page_or_the_card() {
    for (why, terms) in [(CloudWhy::Key, false), (CloudWhy::Terms, true)] {
        let mut rig = Rig::cloud();
        let pick = rig.book.1.get_mut("fr").unwrap();
        (pick.key, pick.terms) = (terms, terms);
        rig.key();
        let out = rig.bar(BarButton::Service);
        assert_eq!(settles(&out), groq_page(why == CloudWhy::Key), "{why:?}");
    }
}

#[test]
fn open_service_on_a_fault_the_service_tells_opens_its_card() {
    for fault in [
        CloudFault::Account,
        CloudFault::Allowance { renews_in: None },
    ] {
        let mut rig = Rig::cloud();
        rig.listen();
        rig.hand_over();
        rig.fail(fault);
        let out = rig.bar(BarButton::Service);
        assert_eq!(settles(&out), groq_page(false), "{fault:?}");
    }
}

#[test]
fn open_service_on_the_off_note_opens_the_card_also_once_the_dictation_ended() {
    let mut rig = Rig::cloud();
    rig.key();
    rig.book.1.clear();
    let settings = Box::new(VoiceSettings::from_defaults().unwrap());
    rig.on(In::Settings(settings));
    let off = cloud_note(CloudWhy::Off, LocalUse::Dimmed, false);
    assert_eq!(rig.shown(), off, "the take ended while preparing");
    assert_eq!(settles(&rig.bar(BarButton::Service)), groq_page(false));
}

#[test]
fn a_pick_waiting_for_a_service_off_shows_the_off_note_at_start_and_never_listens() {
    let mut rig = Rig::new();
    rig.book.3.insert("fr".into(), "groq".into());
    let out = rig.key();
    assert!(opened(&out).is_empty(), "nothing listens");
    assert!(uploads(&out).is_empty(), "nothing leaves");
    let off = cloud_note(CloudWhy::Off, LocalUse::Dimmed, false);
    assert_eq!(rig.shown(), off, "owner, 2026-11-21");
    assert_eq!(settles(&rig.bar(BarButton::Service)), groq_page(false));
    let mut password = Rig::new();
    password.book.3.insert("fr".into(), "groq".into());
    password.field(FieldKind::Password, BOX);
    assert_eq!(opened(&password.key()).len(), 1, "edge 10: the local model");
}

#[test]
fn the_other_notes_open_nothing() {
    let mut net = Rig::cloud();
    net.listen();
    net.hand_over();
    net.fail(CloudFault::Network);
    assert!(settles(&net.bar(BarButton::Service)).is_empty(), "Net");
    let mut moved = Rig::cloud();
    moved.book.2.insert("groq".into());
    moved.listen();
    moved.book.1.clear();
    moved.hand_over();
    let note = cloud_note(CloudWhy::Moved, LocalUse::Ready, true);
    assert_eq!(moved.shown(), note);
    assert!(settles(&moved.bar(BarButton::Service)).is_empty(), "Moved");
    let mut quiet = Rig::cloud();
    quiet.key();
    quiet.tell(SpeechNote::Stage(SpeechStage::Open));
    quiet.tell(SpeechNote::Done(done(0)));
    assert!(
        settles(&quiet.bar(BarButton::Service)).is_empty(),
        "Nothing"
    );
}

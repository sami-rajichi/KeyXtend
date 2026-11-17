//! What keeps the voice on the PC (edges 9, 10, 19 and 20): a password box takes the local model or says none is
//! ready, a box the system cannot read goes as text, the key or the service gone sends nothing after it, newer terms
//! wait for a confirmation, and "Use the local model" with none ready stays dimmed and offers Download.

use super::*;

/// A rig in front of Arabic, picked on Groq, with no local model downloaded.
fn arabic() -> Rig {
    let mut rig = Rig::cloud();
    rig.book.1.insert("ar".into(), groq());
    rig.on(In::Layout(layout("ar", "العربية", true)));
    rig
}

/// French's local model as a dictation asks for it.
fn local() -> SpeechAsk {
    let settings = VoiceSettings::from_defaults().unwrap();
    settings.ask(Some(ready("parakeet", "fr").model), None)
}

/// The note a password box shows with no local model ready: Download for Arabic's.
fn no_local() -> VoiceBar {
    let local = LocalUse::Download { bytes: BYTES };
    cloud_note(CloudWhy::NoLocal, local, false)
}

#[test]
fn edge_10_a_password_box_at_start_dictates_with_the_local_model() {
    let mut rig = Rig::cloud();
    rig.field(FieldKind::Password, BOX);
    let out = rig.key();
    assert_eq!(opened(&out), [local()]);
    assert_eq!(rig.mark(), None);
    rig.speak();
    let mut out = rig.key();
    out.extend(rig.write(&["secret"]));
    assert!(uploads(&out).is_empty() && ask(&out).is_some());
    assert!(matches!(rig.shown(), VoiceBar::Text(t) if t.text == VoiceText::Masked));
}

#[test]
fn edge_10_a_password_box_with_no_local_model_ready_says_so_and_offers_download() {
    let mut rig = arabic();
    rig.field(FieldKind::Password, BOX);
    let out = rig.key();
    assert!(opened(&out).is_empty());
    assert_eq!(rig.shown(), no_local());
    assert!(
        rig.bar(BarButton::Download)
            .contains(&Out::Download("ar".into()))
    );
}

#[test]
fn edge_10_a_box_the_system_cannot_read_goes_to_the_cloud_as_text() {
    let mut rig = Rig::cloud();
    rig.field(FieldKind::Unknown, BOX);
    let out = rig.listen();
    assert!(opened(&out)[0].model.is_none());
    assert_eq!(uploads(&rig.hand_over()).len(), 1);
}

#[test]
fn edge_19_a_password_box_before_stop_hands_the_sound_to_the_local_model() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.field(FieldKind::Password, BOX + 1);
    let out = rig.hand_over();
    assert!(uploads(&out).is_empty(), "nothing is uploaded");
    assert_eq!(calls(&out), [RunCall::Stop, RunCall::Again(local())]);
    assert_eq!((rig.shown(), rig.mark()), (VoiceBar::Transcribing, None));
    let out = rig.write(&["secret"]);
    assert!(ask(&out).is_some() && kept(&out).is_empty());
    assert!(matches!(rig.shown(), VoiceBar::Text(t) if t.text == VoiceText::Masked));
}

#[test]
fn edge_19_with_no_local_model_nothing_is_uploaded_and_the_bar_says_none_is_ready() {
    let mut rig = arabic();
    rig.listen();
    rig.field(FieldKind::Password, BOX + 1);
    let out = rig.hand_over();
    assert!(uploads(&out).is_empty());
    assert_eq!(calls(&out), [RunCall::Stop]);
    assert_eq!(rig.shown(), no_local());
}

#[test]
fn edge_9_the_service_switched_off_while_listening_sends_nothing_after_it() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.book.1.clear();
    let settings = Box::new(VoiceSettings::from_defaults().unwrap());
    let out = rig.on(In::Settings(settings));
    assert_eq!(calls(&out), [RunCall::Stop], "handed over, never sent");
    assert!(!rig.lit);
    let want = cloud_note(CloudWhy::Off, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
    let out = rig.tell(SpeechNote::Done(done(0)));
    assert!(uploads(&out).is_empty());
}

#[test]
fn edge_9_the_key_removed_while_sending_drops_the_upload() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    rig.book.1.get_mut("fr").unwrap().key = false;
    let out = rig.on(In::Keys);
    assert!(halts(&out));
    let want = cloud_note(CloudWhy::Key, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
    assert!(
        ask(&rig.reply(&["trop tard"])).is_none(),
        "nothing after it"
    );
}

#[test]
fn edge_9_the_key_removed_while_preparing_ends_the_dictation_with_its_note() {
    let mut rig = Rig::cloud();
    rig.key();
    rig.book.1.get_mut("fr").unwrap().key = false;
    let out = rig.on(In::Keys);
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
    let want = cloud_note(CloudWhy::Key, LocalUse::Dimmed, false);
    assert_eq!(rig.shown(), want);
}

#[test]
fn a_key_missing_at_stop_sends_nothing() {
    let mut rig = Rig::cloud();
    rig.book.1.get_mut("fr").unwrap().key = false;
    rig.listen();
    let out = rig.hand_over();
    assert!(uploads(&out).is_empty());
    let want = cloud_note(CloudWhy::Key, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
}

#[test]
fn newer_terms_send_nothing_until_they_are_confirmed() {
    let mut rig = Rig::cloud();
    rig.book.1.get_mut("fr").unwrap().terms = true;
    rig.listen();
    assert!(uploads(&rig.hand_over()).is_empty());
    let want = cloud_note(CloudWhy::Terms, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
    rig.book.1.get_mut("fr").unwrap().terms = false;
    assert_eq!(uploads(&rig.bar(BarButton::Again)).len(), 1);
}

#[test]
fn edge_20_use_the_local_model_with_none_ready_is_dimmed_and_offers_download() {
    let mut rig = arabic();
    rig.listen();
    rig.hand_over();
    rig.fail(CloudFault::Network);
    let local = LocalUse::Download { bytes: BYTES };
    assert_eq!(rig.shown(), cloud_note(CloudWhy::Net, local, true));
    assert!(calls(&rig.click(VoiceClick::Local)).is_empty(), "dimmed");
    assert!(
        rig.bar(BarButton::Download)
            .contains(&Out::Download("ar".into()))
    );
}

#[test]
fn a_local_model_once_ready_lights_use_the_local_model() {
    let mut rig = arabic();
    rig.listen();
    rig.hand_over();
    rig.fail(CloudFault::Network);
    rig.book.0.insert("ar".into(), Ok(ready("parakeet", "ar")));
    rig.on(In::Panel(None));
    let want = cloud_note(CloudWhy::Net, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
}

//! Acting on a cloud dictation's recording: Cancel hands the speech over and sends nothing, Transcribe anyway and Try
//! again send it to the service the language picks now, "Use the local model" writes it on this PC for good, and no
//! failure moves the voice to another service by itself; nothing is typed twice.

use super::*;

/// The Cancelled note with Transcribe anyway.
fn cancelled() -> (BarHint, Option<BarButton>) {
    (BarHint::Cancelled, Some(BarButton::Anyway))
}

/// The Custom row with a typed model, as another pick.
fn custom() -> Net {
    Net {
        service: "custom".into(),
        name: ServiceName::Custom,
        model: Redacted::new("custom:mine".into()),
        ..groq()
    }
}

/// A rig whose French dictation was handed over to Groq.
fn sending() -> Rig {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    rig
}

#[test]
fn cancel_while_listening_sends_nothing_and_transcribe_anyway_sends_it() {
    let mut rig = Rig::cloud();
    rig.listen();
    let out = rig.bar(BarButton::Cancel);
    assert_eq!(calls(&out), [RunCall::Stop], "the speech stays on the PC");
    assert_eq!(note(&rig.shown()), Some(cancelled()));
    assert!(uploads(&rig.tell(SpeechNote::Done(done(0)))).is_empty());
    let out = rig.bar(BarButton::Anyway);
    assert_eq!(uploads(&out).len(), 1);
    assert_eq!(rig.mark(), Some(groq_name()));
    assert!(ask(&rig.reply(&["quand même"])).is_some());
}

#[test]
fn transcribe_anyway_before_the_hand_over_sends_once_it_comes() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.bar(BarButton::Cancel);
    let out = rig.bar(BarButton::Anyway);
    assert!(uploads(&out).is_empty() && calls(&out).is_empty());
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert_eq!(uploads(&rig.tell(SpeechNote::Done(done(0)))).len(), 1);
}

#[test]
fn cancel_while_sending_drops_the_requests() {
    let mut rig = sending();
    let out = rig.bar(BarButton::Cancel);
    assert!(halts(&out) && calls(&out).is_empty());
    assert_eq!(note(&rig.shown()), Some(cancelled()));
    assert!(ask(&rig.reply(&["trop tard"])).is_none());
    let out = rig.bar(BarButton::Anyway);
    assert_eq!(uploads(&out)[0].round, 2);
}

#[test]
fn the_gear_or_the_stop_while_sending_drops_the_upload() {
    let mut rig = sending();
    let out = rig.gear();
    assert!(halts(&out));
    assert_eq!(note(&rig.shown()), Some(cancelled()));
    let mut rig = sending();
    let out = rig.on(In::Quit);
    assert!(halts(&out));
    assert_eq!(calls(&out), [RunCall::Cancel, RunCall::Close]);
}

#[test]
fn use_the_local_model_writes_the_held_recording_on_this_pc() {
    let mut rig = sending();
    rig.fail(CloudFault::Credit);
    let out = rig.click(VoiceClick::Local);
    let settings = VoiceSettings::from_defaults().unwrap();
    let local = settings.ask(Some(ready("parakeet", "fr").model), None);
    assert_eq!(calls(&out), [RunCall::Again(local)]);
    assert!(uploads(&out).is_empty());
    assert_eq!((rig.shown(), rig.mark()), (VoiceBar::Transcribing, None));
    let out = rig.write(&["en local"]);
    assert_eq!(kept(&out)[0].model.expose(), "parakeet");
}

#[test]
fn after_the_local_model_wrote_it_try_again_stays_on_this_pc() {
    let mut rig = sending();
    rig.fail(CloudFault::Credit);
    rig.click(VoiceClick::Local);
    let first = ask(&rig.write(&["en local"])).unwrap();
    let count = u32::try_from(first.text.expose().chars().count()).unwrap();
    rig.typed(&first, count);
    rig.book.1.insert("fr".into(), custom());
    let out = rig.again();
    let settings = VoiceSettings::from_defaults().unwrap();
    let local = settings.ask(Some(ready("parakeet", "fr").model), None);
    assert!(
        uploads(&out).is_empty(),
        "the local model's take never goes back"
    );
    assert_eq!(calls(&out), [RunCall::Again(local)]);
    assert_eq!((rig.shown(), rig.mark()), (VoiceBar::Transcribing, None));
    let second = ask(&rig.write(&["encore"])).unwrap();
    assert_eq!(
        second.erase, count,
        "the first text is erased, never typed twice"
    );
    assert_eq!(text(&rig.shown()).as_deref(), Some("encore"));
}

#[test]
fn try_again_sends_to_the_service_the_language_picks_now() {
    let mut rig = sending();
    rig.fail(CloudFault::Rate);
    rig.book.1.insert("fr".into(), custom());
    let out = rig.bar(BarButton::Again);
    assert_eq!(uploads(&out)[0].service, "custom");
    assert_eq!(rig.mark(), Some(ServiceName::Custom));
}

#[test]
fn try_again_with_no_cloud_pick_left_writes_with_the_local_model() {
    let mut rig = sending();
    rig.fail(CloudFault::Rate);
    rig.book.1.clear();
    let out = rig.bar(BarButton::Again);
    assert!(uploads(&out).is_empty());
    assert!(matches!(calls(&out)[..], [RunCall::Again(_)]));
    assert_eq!(rig.mark(), None);
}

#[test]
fn a_pick_moved_to_another_service_before_stop_sends_to_neither() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.book.1.insert("fr".into(), custom());
    assert!(uploads(&rig.hand_over()).is_empty());
    let want = cloud_note(CloudWhy::Off, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
    let mut out = rig.tick_after(1_000);
    let settings = Box::new(VoiceSettings::from_defaults().unwrap());
    out.extend(rig.on(In::Settings(settings)));
    out.extend(rig.on(In::Keys));
    assert!(uploads(&out).is_empty(), "only the user's act sends");
}

#[test]
fn the_panels_try_again_on_a_cloud_text_sends_again_and_replaces_the_text() {
    let mut rig = sending();
    let first = ask(&rig.reply(&["bonjour"])).unwrap();
    rig.typed(&first, 8);
    let out = rig.again();
    assert_eq!(uploads(&out).len(), 1);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let second = ask(&rig.reply(&["bonsoir"])).unwrap();
    assert_eq!(
        second.erase, 8,
        "the first text is erased, never typed twice"
    );
}

#[test]
fn a_try_again_that_brings_nothing_back_shows_the_text_again() {
    let mut rig = sending();
    rig.reply(&["bonjour"]);
    rig.again();
    rig.reply(&[]);
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour"));
}

#[test]
fn a_local_takes_try_again_stays_local_when_the_language_now_picks_the_cloud() {
    let mut rig = Rig::new();
    rig.dictate("bonjour");
    rig.book.1.insert("fr".into(), groq());
    let out = rig.again();
    assert!(uploads(&out).is_empty());
    assert!(matches!(calls(&out)[..], [RunCall::Again(_)]));
}

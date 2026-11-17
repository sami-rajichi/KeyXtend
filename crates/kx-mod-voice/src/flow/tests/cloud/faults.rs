//! A cloud fault (edges 6 and 7): the recording stays, the bar names why with Try again and "Use the
//! local model", nothing is typed, and a recording the job let go, or no speech at all, has its own note.

use super::*;

/// Each fault the service's run tells, with the line the bar shows.
fn faults() -> Vec<(CloudFault, CloudWhy)> {
    let renews_in = Some(Duration::from_secs(600));
    vec![
        (CloudFault::Network, CloudWhy::Net),
        (CloudFault::Timeout, CloudWhy::Net),
        (CloudFault::Key, CloudWhy::Key),
        (CloudFault::Credit, CloudWhy::Credit),
        (
            CloudFault::Allowance { renews_in },
            CloudWhy::Allowance { renews_in },
        ),
        (CloudFault::Rate, CloudWhy::Rate),
        (CloudFault::TooBig, CloudWhy::TooBig),
        (CloudFault::Language, CloudWhy::Language(french())),
        (CloudFault::Empty, CloudWhy::Empty),
        (CloudFault::Strange, CloudWhy::Strange),
        (CloudFault::Gone, CloudWhy::Fault),
        (CloudFault::Range("pieces"), CloudWhy::Fault),
    ]
}

/// A rig whose French dictation was handed over to Groq.
fn sending() -> Rig {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    rig
}

#[test]
fn edge_6_each_fault_keeps_the_recording_with_try_again_and_the_local_model() {
    for (fault, why) in faults() {
        let mut rig = sending();
        let out = rig.fail(fault);
        assert!(ask(&out).is_none() && kept(&out).is_empty(), "{fault:?}");
        assert!(calls(&out).is_empty(), "{fault:?}: the run keeps it");
        let want = cloud_note(why, LocalUse::Ready, true);
        assert_eq!(rig.shown(), want, "{fault:?}");
        assert_eq!(
            rig.mark(),
            Some(groq_name()),
            "{fault:?}: it names the service"
        );
        assert!(!rig.lit, "{fault:?}");
    }
}

#[test]
fn edge_6_pieces_told_before_a_fault_are_never_typed_and_try_again_types_once() {
    let mut rig = sending();
    let text = secret("le début");
    rig.sent(Sent::Note(CloudNote::Piece { index: 0, text }));
    let out = rig.fail(CloudFault::Network);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    let out = rig.bar(BarButton::Again);
    assert_eq!(uploads(&out).len(), 1);
    let out = rig.reply(&["tout le texte"]);
    let typed = ask(&out).unwrap();
    assert_eq!(typed.erase, 0);
    assert_eq!(typed.text.expose().as_str(), suffixed("tout le texte"));
}

#[test]
fn edge_7_a_code_the_service_refuses_names_the_language_in_front() {
    let mut rig = Rig::cloud();
    rig.book.1.insert("ar".into(), groq());
    rig.book.0.insert("ar".into(), Ok(ready("parakeet", "ar")));
    rig.on(In::Layout(layout("ar", "العربية", true)));
    rig.listen();
    rig.hand_over();
    rig.fail(CloudFault::Language);
    let lang = VoiceLang {
        name: "العربية".into(),
        iso: "ar".into(),
        rtl: true,
        arabic: true,
    };
    let want = cloud_note(CloudWhy::Language(lang), LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
}

#[test]
fn a_listen_only_runs_late_text_never_mixes_into_the_services_reply() {
    let mut rig = sending();
    let out = rig.write(&["intrus"]);
    assert!(ask(&out).is_none(), "the run wrote nothing to type");
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    let typed = ask(&rig.reply(&["bonjour"])).unwrap();
    assert_eq!(typed.text.expose().as_str(), suffixed("bonjour"));
}

#[test]
fn a_recording_the_job_let_go_says_so_and_offers_nothing() {
    let mut rig = sending();
    rig.fail(CloudFault::Network);
    rig.bar(BarButton::Again);
    let out = rig.sent(Sent::Speech(SpeechError::NoSound));
    assert_eq!(calls(&out), [RunCall::Close]);
    assert_eq!(note(&rig.shown()), Some((BarHint::Fault, None)));
    assert!(uploads(&rig.bar(BarButton::Again)).is_empty());
}

#[test]
fn another_speech_fault_keeps_the_recording_for_try_again() {
    let mut rig = sending();
    let out = rig.sent(Sent::Speech(SpeechError::Internal));
    assert!(calls(&out).is_empty());
    let want = cloud_note(CloudWhy::Fault, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
}

#[test]
fn a_wiped_recording_dims_try_again_and_the_local_model() {
    let mut rig = sending();
    rig.fail(CloudFault::Network);
    rig.tell(SpeechNote::Wiped);
    let want = cloud_note(CloudWhy::Net, LocalUse::Dimmed, false);
    assert_eq!(rig.shown(), want);
    assert!(uploads(&rig.bar(BarButton::Again)).is_empty());
    assert!(calls(&rig.click(VoiceClick::Local)).is_empty());
}

#[test]
fn no_speech_handed_over_sends_nothing_and_says_nothing_was_heard() {
    let mut rig = sending();
    let out = rig.sent(Sent::Silent);
    assert_eq!(calls(&out), [RunCall::Close]);
    let want = (BarHint::Nothing, Some(BarButton::Listen));
    assert_eq!(note(&rig.shown()), Some(want));
}

#[test]
fn the_voice_job_ending_while_sending_lets_the_upload_finish() {
    let mut rig = sending();
    let out = rig.tell(SpeechNote::Fault(SpeechError::Gone));
    assert!(calls(&out).is_empty() && !halts(&out));
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert!(ask(&rig.reply(&["bonjour"])).is_some());
    assert!(!rig.flow.can_again(), "the recording went with the job");
}

#[test]
fn an_empty_reply_says_so_and_keeps_the_recording() {
    let mut rig = sending();
    let out = rig.reply(&[" "]);
    assert!(ask(&out).is_none() && kept(&out).is_empty());
    let want = cloud_note(CloudWhy::Empty, LocalUse::Ready, true);
    assert_eq!(rig.shown(), want);
}

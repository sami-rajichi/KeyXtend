//! A cloud dictation (ADR-0047): the cloud mark from Start, a listen-only run cut by the row, nothing sent before
//! Stop, then the service's texts typed in order, kept and counted under the pick. Faults are in `faults`, the
//! password box, the key and the service's switch in `guard`, Cancel, Try again and the local model in `redo`, and a
//! hand-over that comes after the local model took over in `late`; the card's warning while it listens is in `warn`.

use super::*;
use kx_module_api::ports::{BarNote, CloudFault, CloudNote, CloudWhy, LocalUse, ServiceName};

mod faults;
mod guard;
mod late;
mod redo;
mod warn;

/// The pick the tests dictate French with.
const PICK: &str = "groq:whisper-large-v3-turbo";
/// The longest piece Groq's row takes, held to the most a run cuts.
const PIECE_S: u8 = 120;
/// The sound the listen-only run heard, in ms.
const SOUND_MS: u32 = 4_000;

/// Groq, as the bar's cloud mark names it.
fn groq_name() -> ServiceName {
    ServiceName::Named("Groq".into())
}

/// Groq's Whisper with a key set and its terms confirmed.
fn groq() -> Net {
    Net {
        service: "groq".into(),
        name: groq_name(),
        model: Redacted::new(PICK.into()),
        piece_s: PIECE_S,
        key: true,
        terms: false,
        unavailable: None,
        warn: None,
    }
}

/// The cloud note for `why` with "Use the local model" as `local`, and Try again when `again`.
fn cloud_note(why: CloudWhy, local: LocalUse, again: bool) -> VoiceBar {
    VoiceBar::Note(BarNote {
        hint: BarHint::Cloud { why, local },
        act: again.then_some(BarButton::Again),
        mic: false,
    })
}

/// The uploads among `out`.
fn uploads(out: &[Out]) -> Vec<Upload> {
    let one = |o: &Out| match o {
        Out::Send(up) => Some(up.clone()),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

/// True when `out` drops a running upload.
fn halts(out: &[Out]) -> bool {
    out.iter().any(|o| matches!(o, Out::Halt { .. }))
}

impl Rig {
    /// A rig whose French dictates with Groq.
    fn cloud() -> Self {
        let mut rig = Self::new();
        rig.book.1.insert("fr".into(), groq());
        rig
    }

    /// Starts, opens the microphone and hears speech.
    fn listen(&mut self) -> Vec<Out> {
        let mut out = self.key();
        out.extend(self.tell(SpeechNote::Stage(SpeechStage::Open)));
        out.extend(self.tell(SpeechNote::Heard));
        out
    }

    /// Stops, and the listen-only run hands its speech over.
    fn hand_over(&mut self) -> Vec<Out> {
        self.wait(1_000);
        let mut out = self.key();
        out.extend(self.tell(SpeechNote::Stage(SpeechStage::Shut)));
        let done = SpeechDone {
            sound_ms: SOUND_MS,
            ..done(0)
        };
        out.extend(self.tell(SpeechNote::Done(done)));
        out
    }

    /// The service's run tells `sent` for the last upload.
    fn sent(&mut self, sent: Sent) -> Vec<Out> {
        let (take, round) = self.upload.unwrap();
        self.on(In::Sent { take, round, sent })
    }

    /// The service tells `texts` as pieces, then its end.
    fn reply(&mut self, texts: &[&str]) -> Vec<Out> {
        let mut out = Vec::new();
        for (at, text) in texts.iter().enumerate() {
            let index = u16::try_from(at).unwrap();
            let text = secret(text);
            out.extend(self.sent(Sent::Note(CloudNote::Piece { index, text })));
        }
        out.extend(self.sent(Sent::Note(CloudNote::Done)));
        out
    }

    /// The service's run stops on `fault`.
    fn fail(&mut self, fault: CloudFault) -> Vec<Out> {
        self.sent(Sent::Note(CloudNote::Fault(fault)))
    }

    /// The cloud mark the last view shows.
    fn mark(&self) -> Option<ServiceName> {
        self.page.as_ref().and_then(|p| p.cloud.clone())
    }
}

#[test]
fn a_cloud_pick_shows_the_cloud_mark_with_its_service_from_start() {
    let mut rig = Rig::cloud();
    rig.key();
    assert_eq!(rig.mark(), Some(groq_name()), "while preparing");
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
    assert_eq!(rig.mark(), Some(groq_name()), "while listening");
    rig.tell(SpeechNote::Heard);
    rig.hand_over();
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert_eq!(rig.mark(), Some(groq_name()), "while sending");
    rig.reply(&["bonjour"]);
    assert_eq!(rig.mark(), None, "the text is typed");
    let mut local = Rig::new();
    local.key();
    local.tell(SpeechNote::Stage(SpeechStage::Open));
    assert_eq!(local.mark(), None, "a local dictation has none");
}

#[test]
fn a_cloud_dictation_listens_with_no_model_cut_within_the_row() {
    let mut rig = Rig::cloud();
    let out = rig.key();
    let settings = VoiceSettings::from_defaults().unwrap();
    let want = SpeechAsk {
        piece_s: PIECE_S,
        ..settings.ask(None, None)
    };
    assert_eq!(opened(&out), [want]);
}

#[test]
fn nothing_is_sent_before_stop_then_the_speech_goes_to_the_service_shown() {
    let mut rig = Rig::cloud();
    let mut out = rig.listen();
    rig.wait(1_000);
    let stop = rig.key();
    assert_eq!(calls(&stop), [RunCall::Stop]);
    out.extend(stop);
    out.extend(rig.tell(SpeechNote::Stage(SpeechStage::Shut)));
    assert!(
        uploads(&out).is_empty(),
        "nothing leaves before the hand-over"
    );
    let out = rig.tell(SpeechNote::Done(done(0)));
    let want = Upload {
        take: rig.take,
        round: 1,
        service: "groq".into(),
        lang: "fr".into(),
    };
    assert_eq!(uploads(&out), [want]);
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
}

#[test]
fn the_services_texts_are_typed_in_order_kept_and_counted_under_the_pick() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    rig.wait(700);
    let out = rig.reply(&["bonjour", "Maya"]);
    let typed = ask(&out).unwrap();
    assert_eq!(typed.text.expose().as_str(), suffixed("bonjour Maya"));
    let kept = kept(&out);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].model.expose(), PICK, "History names the service");
    let count = out.iter().find_map(|o| match o {
        Out::Count(count) => Some(count.clone()),
        _ => None,
    });
    let count = count.unwrap();
    assert_eq!(count.model.expose(), PICK, "the counts name the service");
    assert_eq!(count.card, None);
    let want = (2, SOUND_MS, 700, 0, false);
    let got = &count.done;
    let got = (got.pieces, got.sound_ms, got.wait_ms, got.load_ms, got.card);
    assert_eq!(got, want, "the wait runs from Stop to the text");
    assert_eq!(text(&rig.shown()).as_deref(), Some("bonjour Maya"));
    assert!(calls(&out).is_empty(), "the recording stays for Try again");
}

#[test]
fn a_run_that_stops_itself_sends_once_it_hands_the_speech_over() {
    let mut rig = Rig::cloud();
    rig.listen();
    let out = rig.tell(SpeechNote::Stage(SpeechStage::Limit));
    assert!(uploads(&out).is_empty() && calls(&out).is_empty());
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
    assert_eq!(uploads(&rig.tell(SpeechNote::Done(done(0)))).len(), 1);
    rig.reply(&["bonjour"]);
    let hint = |bar: VoiceBar| match bar {
        VoiceBar::Text(text) => text.hint,
        _ => None,
    };
    assert_eq!(hint(rig.shown()), Some(BarHint::TooLong));
}

#[test]
fn a_hand_over_with_no_speech_heard_sends_nothing() {
    let mut rig = Rig::cloud();
    rig.key();
    rig.tell(SpeechNote::Stage(SpeechStage::Open));
    let out = rig.tell(SpeechNote::Done(done(0)));
    assert!(uploads(&out).is_empty());
    let want = (BarHint::Nothing, Some(BarButton::Listen));
    assert_eq!(note(&rig.shown()), Some(want));
}

#[test]
fn a_hidden_run_keeps_and_counts_nothing_from_the_cloud() {
    let mut rig = Rig::with(VoiceSettings::from_defaults().unwrap(), true);
    rig.book.1.insert("fr".into(), groq());
    rig.listen();
    rig.hand_over();
    let out = rig.reply(&["bonjour"]);
    assert!(ask(&out).is_some());
    assert!(kept(&out).is_empty());
    assert!(!out.iter().any(|o| matches!(o, Out::Count(_))));
}

#[test]
fn a_note_of_an_older_upload_or_take_is_dropped() {
    let mut rig = Rig::cloud();
    rig.listen();
    rig.hand_over();
    let (take, round) = rig.upload.unwrap();
    let stale = [(take, round + 1), (take + 1, round)];
    for (take, round) in stale {
        let sent = Sent::Note(CloudNote::Done);
        assert!(ask(&rig.on(In::Sent { take, round, sent })).is_none());
    }
    assert_eq!(rig.shown(), VoiceBar::Transcribing);
}

#[test]
fn a_cloud_note_prints_no_text() {
    let sent = In::Sent {
        take: 1,
        round: 1,
        sent: Sent::Note(CloudNote::Piece {
            index: 0,
            text: secret("bonjour"),
        }),
    };
    assert!(!format!("{sent:?}").contains("bonjour"));
}

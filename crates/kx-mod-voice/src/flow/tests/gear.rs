//! The gear or the tray opening the panel (part 2's edge case 25): a dictation that prepares, listens or transcribes,
//! or Compare's recording, ends exactly as the bar's Cancel ends it; in Idle, Text or a note nothing changes.

use super::close::done_ms;
use super::*;

/// How far a dictation got when the panel opens.
#[derive(Copy, Clone, Debug)]
enum Run {
    /// Started, before the microphone's `Open` stage, so the key is not lit yet.
    Preparing,
    /// The microphone open and speech heard.
    Listening,
    /// Stopped, with a first piece written.
    Transcribing,
}

/// Every running stage.
const RUNS: [Run; 3] = [Run::Preparing, Run::Listening, Run::Transcribing];

/// A rig whose dictation got as far as `run`.
fn rig_at(run: Run) -> Rig {
    let mut rig = Rig::new();
    rig.key();
    if matches!(run, Run::Listening | Run::Transcribing) {
        rig.speak();
    }
    if matches!(run, Run::Transcribing) {
        rig.bar(BarButton::Stop);
        let text = secret("début");
        rig.tell(SpeechNote::Piece { index: 0, text });
    }
    rig
}

#[test]
fn the_gear_ends_a_running_dictation_exactly_as_cancel_does() {
    for run in RUNS {
        let (mut geared, mut cancelled) = (rig_at(run), rig_at(run));
        let out = geared.on(In::Opening);
        assert_eq!(out, cancelled.bar(BarButton::Cancel), "{run:?}");
        assert_eq!(calls(&out), [RunCall::Cancel], "{run:?}");
        assert!(ask(&out).is_none() && kept(&out).is_empty(), "{run:?}");
        assert!(!geared.lit, "{run:?}");
        assert_eq!(geared.shown(), cancelled.shown(), "{run:?}");
    }
}

#[test]
fn nothing_the_run_writes_after_the_gear_is_typed_kept_or_counted() {
    for run in RUNS {
        let mut rig = rig_at(run);
        rig.gear();
        let out = rig.write(&["trop tard"]);
        assert!(ask(&out).is_none() && kept(&out).is_empty(), "{run:?}");
        assert!(!out.iter().any(|o| matches!(o, Out::Count(_))), "{run:?}");
        let hint = note(&rig.shown()).map(|(hint, _)| hint);
        assert_eq!(hint, Some(BarHint::Cancelled), "{run:?}");
    }
}

#[test]
fn the_cancelled_note_stays_while_the_panel_is_open_and_transcribe_anyway_still_writes() {
    let mut rig = rig_at(Run::Listening);
    rig.gear();
    rig.tick_after(done_ms() * 2);
    let offered = Some((BarHint::Cancelled, Some(BarButton::Anyway)));
    assert_eq!(note(&rig.shown()), offered);
    assert!(
        rig.on(In::Opening).is_empty(),
        "a second gear changes nothing"
    );
    let out = rig.bar(BarButton::Anyway);
    assert!(matches!(calls(&out)[..], [RunCall::Again(_)]));
    let typed = ask(&rig.write(&["quand même"])).unwrap();
    assert_eq!(typed.text.expose().as_str(), suffixed("quand même"));
}

#[test]
fn the_gear_in_idle_text_or_a_note_changes_nothing() {
    let mut rig = Rig::new();
    assert!(rig.on(In::Opening).is_empty(), "idle");
    rig.dictate("bonjour");
    assert!(rig.on(In::Opening).is_empty(), "text");
    assert!(text(&rig.shown()).is_some() && rig.flow.can_again());
    rig.on(In::Layout(layout("ar", "العربية", true)));
    rig.wait(1_000);
    rig.key();
    let shown = rig.shown();
    assert!(note(&shown).is_some());
    assert!(rig.on(In::Opening).is_empty(), "note");
    assert_eq!(rig.shown(), shown);
}

#[test]
fn the_gear_while_compare_records_hands_it_the_bars_cancel() {
    let (mut geared, mut cancelled) = (Rig::new(), Rig::new());
    geared.on(In::Lend(Lend::Listen));
    cancelled.on(In::Lend(Lend::Listen));
    let out = geared.on(In::Opening);
    assert_eq!(out, cancelled.bar(BarButton::Cancel));
    assert!(out.contains(&Out::Lent(BarButton::Cancel)) && !geared.lit);
    let mut rig = Rig::new();
    rig.on(In::Lend(Lend::Write));
    assert!(
        rig.on(In::Opening).is_empty(),
        "the models write: as the bar's Cancel, nothing"
    );
}

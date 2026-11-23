//! The card's warning on the bar (ruling 11): while a dictation whose service carries it listens, the bar shows it and
//! the flow notes it seen once; nothing waits on it, and a service without one shows none.

use super::*;
use kx_module_api::ports::Fact;

/// The card's warning the tests' service carries.
fn warning() -> Fact {
    Fact {
        id: "trains".into(),
        args: Vec::new(),
        warn: true,
    }
}

/// The warning the bar shows; `None` for none, or for a bar that does not listen.
fn shown_warn(rig: &Rig) -> Option<Fact> {
    match rig.shown() {
        VoiceBar::Listening(listening) => listening.warn,
        _ => None,
    }
}

/// The services among `out` noted as having shown their warning.
fn seen(out: &[Out]) -> Vec<String> {
    let one = |o: &Out| match o {
        Out::Warned(id) => Some(id.clone()),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

#[test]
fn the_bar_shows_the_warning_while_it_listens_and_notes_it_seen_once() {
    let mut rig = Rig::cloud();
    let warned = Net {
        warn: Some(warning()),
        ..groq()
    };
    rig.book.1.insert("fr".into(), warned);
    let mut out = rig.key();
    assert_eq!(shown_warn(&rig), None, "not while preparing");
    out.extend(rig.tell(SpeechNote::Stage(SpeechStage::Open)));
    out.extend(rig.tell(SpeechNote::Heard));
    assert_eq!(shown_warn(&rig), Some(warning()), "while it listens");
    out.extend(rig.tick_after(1_000));
    assert_eq!(shown_warn(&rig), Some(warning()), "for the whole dictation");
    assert_eq!(seen(&out), ["groq"], "noted once, as the microphone opens");
    let out = rig.hand_over();
    assert_eq!(uploads(&out).len(), 1, "nothing waits on it");
    assert!(seen(&out).is_empty());
}

#[test]
fn a_service_without_a_warning_shows_none_and_notes_nothing() {
    let mut rig = Rig::cloud();
    let out = rig.listen();
    assert!(matches!(rig.shown(), VoiceBar::Listening(_)));
    assert_eq!(shown_warn(&rig), None);
    assert!(seen(&out).is_empty());
}

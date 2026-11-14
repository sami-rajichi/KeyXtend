//! Picking effects out of what the flow gave, and the check that no input or effect prints its text.

use super::*;

/// The run calls among `out`.
pub(super) fn calls(out: &[Out]) -> Vec<RunCall> {
    let call = |o: &Out| match o {
        Out::Run { call, .. } => Some(call.clone()),
        _ => None,
    };
    out.iter().filter_map(call).collect()
}

/// The text ask among `out`, if any.
pub(super) fn ask(out: &[Out]) -> Option<TextAsk> {
    out.iter().find_map(|o| match o {
        Out::Type(ask) => Some(ask.clone()),
        _ => None,
    })
}

/// The History entries among `out`.
pub(super) fn kept(out: &[Out]) -> Vec<Kept> {
    let one = |o: &Out| match o {
        Out::Keep(kept) => Some(kept.clone()),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

/// The asks opened among `out`.
pub(super) fn opened(out: &[Out]) -> Vec<SpeechAsk> {
    let one = |o: &Out| match o {
        Out::Open { ask, .. } => Some(ask.clone()),
        _ => None,
    };
    out.iter().filter_map(one).collect()
}

/// The bar's note, if it shows one.
pub(super) fn note(bar: &VoiceBar) -> Option<(BarHint, Option<BarButton>)> {
    match bar {
        VoiceBar::Note(n) => Some((n.hint.clone(), n.act)),
        _ => None,
    }
}

/// The text the bar shows, if any.
pub(super) fn text(bar: &VoiceBar) -> Option<String> {
    match bar {
        VoiceBar::Text(t) => match &t.text {
            VoiceText::Shown { text, .. } => Some(text.expose().clone()),
            VoiceText::Masked => None,
        },
        _ => None,
    }
}

#[test]
fn a_piece_prints_no_text_and_a_fault_prints_its_name() {
    let fault = In::Note {
        take: 1,
        note: SpeechNote::Fault(SpeechError::Gone),
    };
    assert!(format!("{fault:?}").contains("Gone"));
    let piece = In::Note {
        take: 1,
        note: SpeechNote::Piece {
            index: 0,
            text: secret("bonjour"),
        },
    };
    assert!(!format!("{piece:?}").contains("bonjour"));
}

#[test]
fn no_effect_prints_the_text() {
    let mut rig = Rig::new();
    let mut out = rig.dictate("bonjour Maya");
    out.extend(rig.bar(BarButton::Copy));
    assert!(ask(&out).is_some() && !kept(&out).is_empty());
    let printed = format!("{out:?}");
    assert!(!printed.contains("bonjour") && !printed.contains("Maya"));
}

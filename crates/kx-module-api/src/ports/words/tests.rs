//! The typed-text events: their names and gates, and `Debug` that never shows a typed or suggested text.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AssistAsk, AssistChanged, AssistNote, AssistToggle, LatchesAsk, LatchesChanged, LatchesUsed,
    LayoutChanged, PointerMoved, PointerPressed, PowerAsk, PowerAsked, PowerChanged, ScrollAimed,
    ScrollAsk, ScrollChanged, ScrollPage, ScrollPeek, ScrollStop, ScrollTap,
};
use crate::{ModuleStateChanged, Notice, REDACTED_MARKER, SettingsChanged};
use std::collections::HashSet;

/// Texts that appear nowhere else in the output: a Latin one and an Arabic one ("hello").
const SECRETS: [&str; 2] = ["zebra-quartz", "مرحبا"];
/// The `ReadFocusedText` gate.
const READ: Option<Capability> = Some(Capability::ReadFocusedText);

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The words events, in the order `events_have_their_names` checks.
fn words() -> [(&'static str, Option<Capability>); 7] {
    [
        row::<Typed>(),
        row::<TypingMode>(),
        row::<TypeWord>(),
        row::<Suggested>(),
        row::<ChipPicked>(),
        row::<WordsAsk>(),
        row::<PressesWanted>(),
    ]
}

/// Every other event name in the crate.
fn older() -> [&'static str; 23] {
    [
        Notice::NAME,
        ModuleStateChanged::NAME,
        SettingsChanged::NAME,
        LayoutChanged::NAME,
        AssistToggle::NAME,
        AssistChanged::NAME,
        AssistAsk::NAME,
        AssistNote::NAME,
        PointerMoved::NAME,
        PointerPressed::NAME,
        LatchesChanged::NAME,
        LatchesUsed::NAME,
        LatchesAsk::NAME,
        ScrollTap::NAME,
        ScrollStop::NAME,
        ScrollPeek::NAME,
        ScrollChanged::NAME,
        ScrollAsk::NAME,
        ScrollAimed::NAME,
        ScrollPage::NAME,
        PowerAsked::NAME,
        PowerChanged::NAME,
        PowerAsk::NAME,
    ]
}

/// Secret `n` as a redacted text.
fn secret(n: usize) -> Redacted<String> {
    Redacted::new(SECRETS[n].to_owned())
}

/// A lane of one chip per secret, with the email list open on both.
fn suggested() -> Suggested {
    let chip = |n| Chip {
        text: secret(n),
        kind: ChipKind::Word,
        rtl: n == 1,
    };
    let list = ListPage {
        items: vec![secret(0), secret(1)],
        page: 0,
        pages: 1,
    };
    Suggested {
        round: 7,
        lane: Lane::Chips(vec![chip(0), chip(1)]),
        forgetting: false,
        learn: true,
        on: true,
        list: Some(list),
    }
}

#[test]
fn events_have_their_names() {
    let want = [
        "typed",
        "typing-mode",
        "type-word",
        "suggested",
        "chip-picked",
        "words-ask",
        "presses-wanted",
    ];
    assert_eq!(words().map(|r| r.0), want);
}

#[test]
fn every_event_but_presses_wanted_needs_read_focused_text() {
    let want = [READ, READ, READ, READ, READ, READ, None];
    assert_eq!(
        words().map(|r| r.1),
        want,
        "a press request carries nothing"
    );
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = words().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().into_iter().collect();
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn debug_never_shows_a_typed_or_suggested_text() {
    let typed = Typed {
        piece: Piece::Text(secret(1)),
    };
    let word = TypeWord {
        round: 7,
        erase: 2,
        text: secret(0),
    };
    let lane = suggested();
    let list = lane.list.clone();
    let shown = [
        format!("{typed:?}"),
        format!("{word:?}"),
        format!("{lane:?}"),
        format!("{list:?}"),
    ];
    for text in shown {
        assert!(text.contains(REDACTED_MARKER), "{text}");
        for secret in SECRETS {
            assert!(!text.contains(secret), "{text} shows {secret}");
        }
    }
}

#[test]
fn erase_counts_characters_not_utf16_units() {
    assert_eq!(TypeWord::erase_of("ap"), Some(2));
    assert_eq!(TypeWord::erase_of(SECRETS[1]), Some(5), "Arabic letters");
    let face = "a\u{1F600}";
    assert_eq!(face.encode_utf16().count(), 3);
    assert_eq!(
        TypeWord::erase_of(face),
        Some(2),
        "one press per surrogate pair"
    );
    assert_eq!(
        TypeWord::erase_of("e\u{301}"),
        Some(2),
        "a combining accent counts"
    );
    let long = "a".repeat(usize::from(u16::MAX) + 1);
    assert_eq!(TypeWord::erase_of(&long), None);
}

#[test]
fn a_type_word_still_shows_its_round_and_erase() {
    let word = TypeWord {
        round: 7,
        erase: 2,
        text: secret(0),
    };
    let want = format!("TypeWord {{ round: 7, erase: 2, text: {REDACTED_MARKER} }}");
    assert_eq!(format!("{word:?}"), want);
}

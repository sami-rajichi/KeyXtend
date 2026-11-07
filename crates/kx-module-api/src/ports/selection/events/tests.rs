//! The selection chips' events: their names and gates, the chips' order, and `Debug` that hides the strip's boxes.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AimedLine, AssistAsk, AssistChanged, AssistNote, AssistToggle, ChipPicked, ClipAct, ClipKey,
    ClipLine, ClipOpen, ClipPanel, ClipTake, ClipTaken, ClipView, FillAct, FillAsk, FillClose,
    FillKey, FillLine, FillLive, FillNames, FillOpen, FillPanel, FillType, FillTyped, FillView,
    HistoryAsk, HistoryWanted, LatchesAsk, LatchesChanged, LatchesUsed, LayoutAsk, LayoutChanged,
    PanelPressesWanted, PasteAsk, Pasted, PointerMoved, PointerPressed, PointerReleased, PowerAsk,
    PowerAsked, PowerChanged, PressesAsk, PressesWanted, ScrollAimed, ScrollAsk, ScrollChanged,
    ScrollPage, ScrollPeek, ScrollStop, ScrollTap, Suggested, TypeWord, Typed, TypingMode,
    TypingModeAsk, VaultEmails, WordsAsk,
};
use crate::{ModuleStateChanged, Notice, NoticeAnswered, REDACTED_MARKER, SettingsChanged};
use std::collections::HashSet;

/// The `Selection` gate.
const SELECTION: Option<Capability> = Some(Capability::Selection);
/// A line box whose numbers appear nowhere else in the output.
const LINE: Rect = Rect {
    x: 1357,
    y: 2468,
    w: 9753,
    h: 8642,
};
/// Its numbers, which no output may show.
const HIDDEN: [&str; 4] = ["1357", "2468", "9753", "8642"];

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The chips' events, in the order `events_have_their_names` checks.
fn chips() -> [(&'static str, Option<Capability>); 4] {
    [
        row::<ChipsView>(),
        row::<ChipAct>(),
        row::<EditAsk>(),
        row::<EditDone>(),
    ]
}

/// Every other event name in the crate: the earlier ones, then the clipboard's and Quick-fill's.
fn older() -> impl Iterator<Item = &'static str> {
    earlier().into_iter().chain(panels())
}

/// The event names from before the clipboard history.
fn earlier() -> [&'static str; 37] {
    [
        Notice::NAME,
        NoticeAnswered::NAME,
        ModuleStateChanged::NAME,
        SettingsChanged::NAME,
        LayoutChanged::NAME,
        LayoutAsk::NAME,
        AssistToggle::NAME,
        AssistChanged::NAME,
        AssistAsk::NAME,
        AssistNote::NAME,
        PointerMoved::NAME,
        PointerPressed::NAME,
        PointerReleased::NAME,
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
        Typed::NAME,
        TypingMode::NAME,
        TypingModeAsk::NAME,
        TypeWord::NAME,
        Suggested::NAME,
        ChipPicked::NAME,
        WordsAsk::NAME,
        PressesWanted::NAME,
        PanelPressesWanted::NAME,
        PressesAsk::NAME,
        AimedLine::NAME,
    ]
}

/// The clipboard history's and Quick-fill's event names.
fn panels() -> [&'static str; 25] {
    [
        ClipOpen::NAME,
        ClipPanel::NAME,
        ClipKey::NAME,
        ClipView::NAME,
        ClipAct::NAME,
        PasteAsk::NAME,
        Pasted::NAME,
        HistoryWanted::NAME,
        HistoryAsk::NAME,
        ClipTake::NAME,
        ClipTaken::NAME,
        ClipLine::NAME,
        FillOpen::NAME,
        FillClose::NAME,
        FillAsk::NAME,
        FillLive::NAME,
        FillPanel::NAME,
        FillKey::NAME,
        FillView::NAME,
        FillAct::NAME,
        FillNames::NAME,
        FillType::NAME,
        FillTyped::NAME,
        VaultEmails::NAME,
        FillLine::NAME,
    ]
}

/// A strip of every chip by `LINE`, its Copy just clicked.
fn shown() -> ChipsView {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let screen = Screen {
        rect,
        dpi: 120,
        primary: true,
    };
    ChipsView::Shown {
        round: 7,
        chips: SelChip::ALL.to_vec(),
        spot: StripSpot {
            first: LINE,
            last: LINE,
            screen,
        },
        acted: Some(SelChip::Copy),
    }
}

#[test]
fn events_have_their_names() {
    let want = ["chips-view", "chip-act", "edit-ask", "edit-done"];
    assert_eq!(chips().map(|r| r.0), want);
}

#[test]
fn every_event_needs_selection() {
    assert_eq!(chips().map(|r| r.1), [SELECTION; 4], "ADR-0041 decision 3");
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = chips().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().collect();
    assert_eq!(
        others.len(),
        older().count(),
        "the older names are unique too"
    );
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn the_chips_keep_their_order_in_every_language() {
    let want = [
        SelChip::Copy,
        SelChip::Cut,
        SelChip::Paste,
        SelChip::Translate,
    ];
    assert_eq!(SelChip::ALL, want, "Arabic flips text only (ADR-0020)");
    assert_eq!(Edit::ALL, [Edit::Copy, Edit::Cut, Edit::Paste]);
}

#[test]
fn a_view_hides_the_strips_boxes_and_shows_the_rest() {
    let text = format!("{:?}", shown());
    for number in HIDDEN {
        assert!(!text.contains(number), "{text} shows {number}");
    }
    assert!(text.contains(REDACTED_MARKER), "{text}");
    for part in ["round: 7", "Translate", "acted: Some(Copy)", "dpi: 120"] {
        assert!(text.contains(part), "{text} lacks {part}");
    }
    assert_eq!(format!("{:?}", ChipsView::Hidden), "Hidden");
}

#[test]
fn an_edit_ask_carries_its_round_its_edit_and_whether_a_terminal_has_focus() {
    let ask = EditAsk {
        round: 3,
        edit: Edit::Paste,
        terminal: true,
    };
    let text = format!("{ask:?}");
    assert_eq!(text, "EditAsk { round: 3, edit: Paste, terminal: true }");
    let done = EditDone {
        round: 3,
        ok: false,
    };
    assert_eq!(format!("{done:?}"), "EditDone { round: 3, ok: false }");
    let act = ChipAct {
        round: 3,
        chip: SelChip::Cut,
    };
    assert_eq!(format!("{act:?}"), "ChipAct { round: 3, chip: Cut }");
}

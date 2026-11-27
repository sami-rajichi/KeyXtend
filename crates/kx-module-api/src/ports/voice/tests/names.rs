//! Every event name in the crate before voice typing, so the voice and snip events' names stay unique.

use crate::ports::{
    AimedLine, AssistAsk, AssistChanged, AssistNote, AssistToggle, BoardPlace, BoardPlaceAsk,
    ChipAct, ChipPicked, ChipsView, ClipAct, ClipKey, ClipLine, ClipOpen, ClipPanel, ClipTake,
    ClipTaken, ClipView, EditAsk, EditDone, FillAct, FillAsk, FillClose, FillKey, FillLine,
    FillLive, FillNames, FillOpen, FillPanel, FillType, FillTyped, FillView, HistoryAsk,
    HistoryWanted, LatchesAsk, LatchesChanged, LatchesUsed, LayoutAsk, LayoutChanged,
    PanelPressesWanted, PasteAsk, Pasted, PointerMoved, PointerPressed, PointerReleased, PowerAsk,
    PowerAsked, PowerChanged, PressesAsk, PressesWanted, ScrollAimed, ScrollAsk, ScrollChanged,
    ScrollPage, ScrollPeek, ScrollStop, ScrollTap, Suggested, TypeWord, Typed, TypingMode,
    TypingModeAsk, VaultEmails, WordsAsk,
};
use crate::{Event, ModuleStateChanged, Notice, NoticeAnswered, SettingsChanged};

/// The older names, from the kernel's notices to the selection chips.
pub(in crate::ports) fn older() -> impl Iterator<Item = &'static str> {
    kernel()
        .into_iter()
        .chain(typing())
        .chain(panels())
        .chain(chips())
}

/// The notices, settings, layouts, assist, pointer, latches, scroll and power events.
fn kernel() -> [&'static str; 26] {
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
    ]
}

/// Prediction's and the typing mode's events.
fn typing() -> [&'static str; 11] {
    [
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

/// The clipboard history's and Quick-fill's events.
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

/// The selection chips' and the keyboard's place events.
fn chips() -> [&'static str; 6] {
    [
        ChipsView::NAME,
        ChipAct::NAME,
        EditAsk::NAME,
        EditDone::NAME,
        BoardPlace::NAME,
        BoardPlaceAsk::NAME,
    ]
}

//! Paste at once's events: their names and gates, the start's share of the asker's wait, and `Debug` that never
//! shows the text.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    ClipAct, ClipKey, ClipLine, ClipOpen, ClipPanel, ClipTake, ClipTaken, ClipView, FillType,
    FillTyped, HistoryAsk, HistoryWanted, PasteAsk, Pasted, TextAsk, TextDone,
};
use crate::{Redacted, Zeroizing};
use std::collections::HashSet;

/// A text that appears nowhere else in the output: an Arabic one ("hello").
const SAID: &str = "مرحبا";

/// The ask for `SAID` with a wait of `wait_ms`.
fn ask(wait_ms: u32) -> PasteText {
    PasteText {
        round: 4,
        text: Redacted::new(Zeroizing::new(SAID.to_owned())),
        wait_ms,
    }
}

#[test]
fn the_pair_has_its_names_and_needs_write_clipboard_as_the_paste_does() {
    let write = Some(Capability::WriteClipboard);
    assert_eq!(
        [PasteText::NAME, TextPasted::NAME],
        ["paste-text", "text-pasted"]
    );
    assert_eq!([PasteText::CAPABILITY, TextPasted::CAPABILITY], [write; 2]);
    assert_eq!(
        PasteAsk::CAPABILITY,
        write,
        "the same gate as the paste request"
    );
}

#[test]
fn names_are_kebab_case_and_unique_among_the_clipboards_and_the_typists() {
    let ours = [PasteText::NAME, TextPasted::NAME];
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = [
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
        TextAsk::NAME,
        TextDone::NAME,
        FillType::NAME,
        FillTyped::NAME,
    ]
    .into_iter()
    .collect();
    assert!(ours.iter().all(|n| !others.contains(n)), "{ours:?}");
    assert_ne!(ours[0], ours[1]);
}

#[test]
fn a_paste_starts_only_within_the_first_half_of_the_askers_wait() {
    assert_eq!(ask(2_000).start_ms(), 1_000);
    assert_eq!(ask(0).start_ms(), 0, "no wait, no start");
    assert_eq!(ask(u32::MAX).start_ms(), u32::MAX / 2);
}

#[test]
fn debug_never_shows_the_text() {
    let shown = format!("{:?}", ask(2_000));
    assert!(!shown.contains(SAID), "{shown}");
    let answer = TextPasted { round: 4, ok: true };
    assert!(format!("{answer:?}").contains("ok: true"));
}

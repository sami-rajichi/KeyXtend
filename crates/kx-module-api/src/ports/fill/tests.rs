//! Quick-fill's events: their names and gates, a view shared by its clones, and `Debug` that never shows a value, a
//! name, a search, a password or a place. The page's own rules are in `page`, and fill text's in `text`.

mod page;
mod sheets;
mod text;

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AimedLine, AssistAsk, AssistChanged, AssistNote, AssistToggle, BoardPlace, BoardPlaceAsk,
    ChipPicked, ClipAct, ClipKey, ClipLine, ClipOpen, ClipPanel, ClipTake, ClipTaken, ClipView,
    HistoryAsk, HistoryWanted, LatchesAsk, LatchesChanged, LatchesUsed, LayoutAsk, LayoutChanged,
    PanelLine, PanelPressesWanted, PasteAsk, Pasted, PointerMoved, PointerPressed, PointerReleased,
    PowerAsk, PowerAsked, PowerChanged, PressesAsk, PressesWanted, ScrollAimed, ScrollAsk,
    ScrollChanged, ScrollPage, ScrollPeek, ScrollStop, ScrollTap, Suggested, TypeWord, Typed,
    TypingMode, TypingModeAsk, WordsAsk,
};
use crate::{
    ModuleStateChanged, Notice, NoticeAnswered, REDACTED_MARKER, Redacted, SettingsChanged,
    Zeroizing,
};
use sheets::{CARET, page, sheets};
use std::collections::HashSet;
use std::time::Duration;

/// Texts that appear nowhere else in the output: a Latin value, an Arabic name ("my bank") and a place's name.
const SECRETS: [&str; 3] = ["zebra-quartz", "بنكي", "OneDrive-Contoso"];
/// The `Vault` gate.
const VAULT: Option<Capability> = Some(Capability::Vault);

/// Secret `n` as a secret text.
fn secret(n: usize) -> Secret {
    Secret::new(Zeroizing::new(SECRETS[n].to_owned()))
}

/// Secret `n` as a redacted text.
fn text(n: usize) -> Redacted<String> {
    Redacted::new(SECRETS[n].to_owned())
}

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// Quick-fill's events, in the order `events_have_their_names` checks.
fn fills() -> [(&'static str, Option<Capability>); 13] {
    [
        row::<FillOpen>(),
        row::<FillClose>(),
        row::<FillAsk>(),
        row::<FillLive>(),
        row::<FillPanel>(),
        row::<FillKey>(),
        row::<FillView>(),
        row::<FillAct>(),
        row::<FillNames>(),
        row::<FillType>(),
        row::<FillTyped>(),
        row::<VaultEmails>(),
        row::<FillLine>(),
    ]
}

/// Every other event name in the crate: the earlier ones, the clipboard's, then the keyboard's place events.
fn older() -> impl Iterator<Item = &'static str> {
    let places = [BoardPlace::NAME, BoardPlaceAsk::NAME];
    earlier().into_iter().chain(clips()).chain(places)
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

/// The clipboard history's event names.
fn clips() -> [&'static str; 12] {
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
    ]
}

#[test]
fn events_have_their_names() {
    let want = [
        "fill-open",
        "fill-close",
        "fill-ask",
        "fill-live",
        "fill-panel",
        "fill-key",
        "fill-view",
        "fill-act",
        "fill-names",
        "fill-type",
        "fill-typed",
        "vault-emails",
        "fill-line",
    ];
    assert_eq!(fills().map(|r| r.0), want);
}

#[test]
fn every_event_needs_vault_but_the_open_toggle_the_close_and_the_ask() {
    let want = [
        None, None, None, VAULT, VAULT, VAULT, VAULT, VAULT, VAULT, VAULT, VAULT, VAULT, VAULT,
    ];
    assert_eq!(fills().map(|r| r.1), want, "these three carry nothing");
}

#[test]
fn the_label_names_map_each_table_label_to_its_shown_text() {
    let shown = [("email", "Courriel"), ("full-name", "الاسم الكامل")];
    let names = FillNames(
        shown
            .map(|(id, text)| (id.to_owned(), text.to_owned()))
            .into(),
    );
    assert_eq!(names.0.get("email").map(String::as_str), Some("Courriel"));
    assert_eq!(names.0.len(), shown.len());
    assert_eq!(names.clone(), names, "a subscriber keeps its own copy");
}

#[test]
fn a_fill_carries_the_wait_after_each_tab() {
    let gap = Duration::from_millis(150);
    let ask = FillType {
        round: 2,
        parts: vec![FillPart::Tab],
        tab_gap: gap,
    };
    assert_eq!(ask.clone().tab_gap, gap);
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = fills().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().collect();
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn debug_never_shows_a_value_a_name_a_search_a_password_or_a_place() {
    let line = |n| FillPart::Text(FillText::new(secret(n)).unwrap());
    let parts = vec![line(0), FillPart::Tab, line(1)];
    let emails = vec![secret(0)];
    let mut shown: Vec<_> = sheets()
        .into_iter()
        .map(|s| format!("{:?}", FillView::new(page(s))))
        .collect();
    shown.push(format!("{:?}", FillKey(PanelKey::Text(text(1)))));
    let word = PanelKey::Word {
        erase: 1,
        text: text(1),
    };
    shown.push(format!("{:?}", FillKey(word)));
    shown.push(format!("{:?}", FillLine(PanelLine::Search(text(0)))));
    let tab_gap = Duration::from_millis(1);
    let ask = FillType {
        round: 1,
        parts,
        tab_gap,
    };
    shown.push(format!("{ask:?}"));
    shown.push(format!("{:?}", VaultEmails { emails }));
    for text in &shown {
        assert!(text.contains(REDACTED_MARKER), "{text}");
    }
    let click = FillClick::Row {
        row: 1,
        button: RowButton::Both,
    };
    shown.push(format!("{:?}", FillAct { round: 3, click }));
    for text in shown {
        for secret in SECRETS {
            assert!(!text.contains(secret), "{text} shows {secret}");
        }
    }
}

#[test]
fn debug_never_shows_a_clicks_caret() {
    let click = FillClick::Caret {
        at: Redacted::new(CARET),
    };
    let text = format!("{:?}", FillAct { round: 3, click });
    assert!(text.contains(REDACTED_MARKER), "{text}");
    assert!(!text.contains(&CARET.to_string()), "{text} shows the caret");
}

#[test]
fn a_view_and_its_clones_share_one_page() {
    let view = FillView::new(page(Sheet::List(Vec::new())));
    let copy = view.clone();
    assert!(Arc::ptr_eq(&view.0, &copy.0));
    assert_eq!(copy.0.round, view.0.round);
}

#[test]
fn quick_fill_is_off_until_the_vault_says_it_runs() {
    assert_eq!(FillLive::OFF, FillLive { live: false });
    assert_eq!(FillLive::NAME, "fill-live");
}

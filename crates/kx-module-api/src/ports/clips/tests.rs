//! The clipboard history's events: their names and gates, a view shared by its clones, and `Debug` that never shows
//! a copied text, a search, a path or an app.

use super::*;
use crate::capability::is_kebab;
use crate::ports::{
    AimedLine, AssistAsk, AssistChanged, AssistNote, AssistToggle, ChipPicked, Date, LatchesAsk,
    LatchesChanged, LatchesUsed, LayoutAsk, LayoutChanged, MAX_FIELD_CHARS, PanelPressesWanted,
    Pixels, PointerMoved, PointerPressed, PointerReleased, PowerAsk, PowerAsked, PowerChanged,
    PressesAsk, PressesWanted, Room, ScrollAimed, ScrollAsk, ScrollChanged, ScrollPage, ScrollPeek,
    ScrollStop, ScrollTap, Suggested, TypeWord, Typed, TypingMode, TypingModeAsk, WordsAsk,
};
use crate::{
    ModuleStateChanged, Notice, NoticeAnswered, REDACTED_MARKER, Secret, SettingsChanged, Zeroizing,
};
use std::collections::HashSet;
use std::path::PathBuf;

/// Texts that appear nowhere else in the output: a Latin one, an Arabic one ("hello"), and an app's name.
const SECRETS: [&str; 3] = ["zebra-quartz", "مرحبا", "KeePass"];
/// A caret that appears nowhere else.
const CARET: u32 = 73_519;
/// The `ReadClipboard` gate.
const READ: Option<Capability> = Some(Capability::ReadClipboard);
/// The `WriteClipboard` gate.
const WRITE: Option<Capability> = Some(Capability::WriteClipboard);

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The clipboard events, in the order `events_have_their_names` checks.
fn clips() -> [(&'static str, Option<Capability>); 12] {
    [
        row::<ClipOpen>(),
        row::<ClipPanel>(),
        row::<ClipKey>(),
        row::<ClipView>(),
        row::<ClipAct>(),
        row::<PasteAsk>(),
        row::<Pasted>(),
        row::<HistoryWanted>(),
        row::<HistoryAsk>(),
        row::<ClipTake>(),
        row::<ClipTaken>(),
        row::<ClipLine>(),
    ]
}

/// Every other event name in the crate.
fn older() -> [&'static str; 37] {
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

/// Secret `n` as a redacted text.
fn secret(n: usize) -> Redacted<String> {
    Redacted::new(SECRETS[n].to_owned())
}

/// A ticked, pinned row of secret `n`, copied today at 16:32 in the secret app.
fn clip_row(n: usize) -> ClipRow {
    ClipRow {
        id: 4,
        kind: ClipKind::Text,
        title: secret(n),
        rtl: n == 1,
        arabic: n == 1,
        app: Some(secret(2)),
        when: When {
            days_ago: 0,
            date: Date::new(2026, 10, 31).unwrap(),
            hour: 16,
            minute: 32,
        },
        bytes: Some(12),
        tick: Some(1),
        pinned: true,
        expires: false,
        thumb: Pixels::new(1, 1, vec![0; 4], 1),
    }
}

/// A page of both secrets, searched for one, previewing the other.
fn page(preview: Preview) -> ClipPage {
    ClipPage {
        round: 3,
        rows: vec![clip_row(0), clip_row(1)],
        highlight: Some(0),
        filter: Filter::All,
        search: secret(0),
        caret: Redacted::new(CARET),
        pages: Pages { at: 0, count: 1 },
        ticked: 2,
        unpin: true,
        preview: Some(preview),
        confirm: None,
        unpinned: 1,
        room: Some(Room {
            used: 1,
            free: 2,
            drive: "D:".to_owned(),
        }),
        cap: 3,
        note: Some(PanelNote::Disk),
        kept_days: 7,
        empty: false,
    }
}

#[test]
fn events_have_their_names() {
    let want = [
        "clip-open",
        "clip-panel",
        "clip-key",
        "clip-view",
        "clip-act",
        "paste-ask",
        "pasted",
        "history-wanted",
        "history-ask",
        "clip-take",
        "clip-taken",
        "clip-line",
    ];
    assert_eq!(clips().map(|r| r.0), want);
}

#[test]
fn the_panel_and_content_need_read_clipboard_and_the_paste_write_clipboard() {
    let want = [
        None, READ, READ, READ, READ, WRITE, WRITE, None, None, READ, READ, READ,
    ];
    assert_eq!(
        clips().map(|r| r.1),
        want,
        "the open toggle and the chord flags carry nothing"
    );
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = clips().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = older().into_iter().collect();
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn debug_never_shows_a_text_a_search_a_path_or_an_app() {
    let text = Preview::Text {
        text: secret(1),
        chars: 5,
    };
    let files = Preview::Files(Redacted::new(vec![PathBuf::from(SECRETS[0])]));
    let taken = Secret::new(Zeroizing::new(SECRETS[0].to_owned()));
    let shown = [
        format!("{:?}", ClipKey(PanelKey::Text(secret(1)))),
        format!(
            "{:?}",
            ClipKey(PanelKey::Word {
                erase: 1,
                text: secret(1)
            })
        ),
        format!("{:?}", ClipView::new(page(text))),
        format!("{:?}", ClipView::new(page(files))),
        format!(
            "{:?}",
            ClipTaken {
                round: 1,
                text: ClipText::Text(taken)
            }
        ),
        format!("{:?}", ClipLine(PanelLine::Search(secret(1)))),
        format!("{:?}", ClipAct::Tick { round: 3, row: 1 }),
        format!("{:?}", ClipAct::Rows(5)),
        format!("{:?}", ClipTake::new(1, 9)),
    ];
    for text in &shown[..6] {
        assert!(text.contains(REDACTED_MARKER), "{text}");
    }
    for text in shown {
        for secret in SECRETS {
            assert!(!text.contains(secret), "{text} shows {secret}");
        }
    }
}

#[test]
fn debug_never_shows_a_clicks_caret() {
    let click = ClipAct::Caret {
        at: Redacted::new(CARET),
    };
    let text = format!("{click:?}");
    assert!(text.contains(REDACTED_MARKER), "{text}");
    assert!(!text.contains(&CARET.to_string()), "{text} shows the caret");
}

#[test]
fn a_line_with_no_text_shows_only_its_kind() {
    assert_eq!(format!("{:?}", PanelLine::None), "None");
    assert_eq!(format!("{:?}", PanelLine::Private), "Private");
}

#[test]
fn a_view_and_its_clones_share_one_page() {
    let view = ClipView::new(page(Preview::Image(None)));
    let copy = view.clone();
    assert!(Arc::ptr_eq(&view.0, &copy.0));
    assert_eq!(copy.0.rows.len(), 2);
}

#[test]
fn the_filters_step_from_all_through_each_kind_once() {
    assert_eq!(Filter::ORDER[0], Filter::All);
    let each: HashSet<_> = Filter::ORDER.into_iter().collect();
    assert_eq!(each.len(), Filter::ORDER.len());
}

#[test]
fn every_kind_and_note_is_listed_once_in_declaration_order() {
    // `listed!` declares each enum with its `ALL`, so a new variant is listed by being declared.
    for (at, kind) in ClipKind::ALL.into_iter().enumerate() {
        assert_eq!(kind as usize, at, "{kind:?}");
    }
    for (at, note) in PanelNote::ALL.into_iter().enumerate() {
        assert_eq!(note as usize, at, "{note:?}");
    }
}

#[test]
fn a_megabyte_is_the_binary_one_windows_counts_sizes_in() {
    assert_eq!(MB, 1024 * 1024);
}

#[test]
fn a_take_reads_twice_a_fields_characters_but_never_past_the_longest_fields_room() {
    let most = MAX_FIELD_CHARS * 2;
    assert_eq!(
        (ClipTake::new(3, 10).round, ClipTake::new(3, 10).max_chars),
        (3, 20)
    );
    assert_eq!(ClipTake::new(1, 10).cap(), 20);
    assert_eq!(ClipTake::new(1, MAX_FIELD_CHARS).cap(), most);
    assert_eq!(ClipTake::new(1, usize::MAX).cap(), most);
    let wide = ClipTake {
        round: 1,
        max_chars: most + 1,
    };
    assert_eq!(wide.cap(), most);
}

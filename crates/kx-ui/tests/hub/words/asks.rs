//! Taps on the words menu, the email list and the control chips: each kind QML names sends its `WordsAsk`, a list's
//! rows come from `[ui] page_rows`, and an unknown kind or an argument out of range sends nothing.

use super::{heard, speaking};
use crate::Rig;
use kx_module_api::ports::WordsAsk;
use kx_ui::asks::{AGAIN, CANCEL, FORGET, MORE, PAGE, PAUSE, PICK, POWER, UNDO};

/// Rows per page in these tests, unlike the default, so the ask shows where its rows came from.
const ROWS: u8 = 3;

#[test]
fn qml_names_each_ask_as_the_top_bar_sends_it() {
    let kinds = [FORGET, CANCEL, PAUSE, POWER, UNDO, MORE, PAGE, PICK, AGAIN];
    let want = [
        "forget", "cancel", "pause", "power", "undo", "more", "page", "pick", "again",
    ];
    assert_eq!(kinds, want);
}

#[test]
fn each_kind_sends_its_ask_and_the_list_takes_the_ui_rows() {
    let (rig, _host) = speaking("en", ROWS.into());
    let (_bus, sent, _subs) = heard(&rig);
    sent.asks();
    let page = |page| WordsAsk::Page { page, rows: ROWS };
    let cases = [
        (FORGET, 0, WordsAsk::Forget),
        (CANCEL, 0, WordsAsk::Cancel),
        (PAUSE, 1, WordsAsk::Pause(true)),
        (PAUSE, 0, WordsAsk::Pause(false)),
        (POWER, 1, WordsAsk::Power(true)),
        (POWER, 0, WordsAsk::Power(false)),
        (UNDO, 0, WordsAsk::Undo),
        (MORE, 0, WordsAsk::More { rows: ROWS }),
        (PAGE, 1, page(1)),
        (PAGE, -1, page(-1)),
        (PICK, 4, WordsAsk::ListPick(4)),
        (AGAIN, 0, WordsAsk::Again),
    ];
    for (kind, arg, want) in cases {
        rig.hub.words_ask(kind, arg);
        assert_eq!(sent.asks(), [want], "{kind} {arg}");
    }
}

#[test]
fn a_kind_without_an_argument_ignores_it() {
    let rig = Rig::new();
    let (_bus, sent, _subs) = heard(&rig);
    sent.asks();
    rig.hub.words_ask(FORGET, -9);
    rig.hub.words_ask(UNDO, 300);
    assert_eq!(sent.asks(), [WordsAsk::Forget, WordsAsk::Undo]);
}

#[test]
fn an_unknown_kind_or_an_argument_out_of_range_sends_nothing() {
    let rig = Rig::new();
    let (_bus, sent, _subs) = heard(&rig);
    sent.asks();
    let bad = [
        ("mystery", 0),
        ("", 0),
        ("Forget", 0),
        (PAUSE, 2),
        (PAUSE, -1),
        (POWER, 2),
        (PAGE, i32::from(i8::MAX) + 1),
        (PAGE, i32::from(i8::MIN) - 1),
        (PICK, -1),
        (PICK, i32::from(u8::MAX) + 1),
    ];
    for (kind, arg) in bad {
        rig.hub.words_ask(kind, arg);
    }
    assert!(sent.asks().is_empty());
}

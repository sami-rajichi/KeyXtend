//! The keyboard's typing mode: asked for at each start, as the keyboard may have started first, a change tells a new
//! lane while the same mode again keeps the shown one, and its text cap drops every chip the keyboard would refuse,
//! emails included.

use crate::rig::{Heard, ROWS, Rig, plant};
use kx_mod_predict::Msg;
use kx_mod_predict::email::Emails;
use kx_module_api::ports::{ChipKind, ChipPicked, FieldKind, TypingMode, WordsAsk};

/// A long known email (19 characters) and a short one (10).
const LONG: &str = "someone@example.org";
const SHORT: &str = "alex@x.org";

/// The mode with the keyboard's text cap at `cap` characters.
fn capped(cap: u16) -> TypingMode {
    TypingMode {
        cap: Some(cap),
        ..TypingMode::default()
    }
}

/// A rig in English, in a text box, whose learner offers `hello` and `help` and which knows both emails.
fn ready() -> Rig {
    let bytes = Emails::new(&[LONG.to_owned(), SHORT.to_owned()], usize::MAX).to_bytes();
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello", "help"]);
        f.store
            .plant(&kx_mod_predict::emails_blob().unwrap(), bytes);
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

#[test]
fn it_asks_for_the_typing_mode_once_at_start() {
    let rig = Rig::new();
    let asks = rig.all().into_iter().filter(|h| *h == Heard::ModeAsk);
    assert_eq!(asks.count(), 1);
}

#[test]
fn an_unchanged_mode_tells_no_new_lane_and_a_pick_of_the_shown_round_still_types() {
    let mut rig = ready();
    rig.typed("hel");
    let shown = rig.lane().round;
    rig.take();
    rig.send(Msg::Mode(TypingMode::default()));
    let lanes = rig
        .take()
        .into_iter()
        .filter(|h| matches!(h, Heard::Lane(_)));
    assert_eq!(lanes.count(), 0, "the same mode again");
    rig.send(Msg::Picked(ChipPicked {
        round: shown,
        index: 0,
    }));
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())]);
}

#[test]
fn a_changed_mode_tells_a_new_lane() {
    let mut rig = ready();
    rig.typed("hel");
    let shown = rig.lane().round;
    rig.send(Msg::Mode(capped(2)));
    assert_ne!(rig.lane().round, shown, "a new round");
    assert_eq!(rig.words(), ["help"]);
}

#[test]
fn a_word_chip_past_the_cap_is_dropped() {
    let mut rig = ready();
    rig.send(Msg::Mode(capped(2)));
    rig.typed("hel");
    assert_eq!(
        rig.words(),
        ["help"],
        "the rest of hello and its space take 3"
    );
    rig.send(Msg::Mode(TypingMode::default()));
    assert_eq!(rig.words(), ["hello", "help"], "no cap told, none dropped");
}

#[test]
fn an_email_chip_or_list_row_past_the_cap_is_dropped() {
    let mut rig = ready();
    rig.send(Msg::Mode(capped(12)));
    rig.typed("@@");
    let emails: Vec<String> = rig.chips().into_iter().map(|c| c.0).collect();
    assert_eq!(emails, [SHORT], "the long one does not fit");
    rig.ask(WordsAsk::More { rows: ROWS });
    let list = rig.lane().list.unwrap();
    let rows: Vec<&str> = list
        .items
        .iter()
        .map(|e| e.text.expose().as_str())
        .collect();
    assert_eq!(
        (rows, list.pages),
        (vec![SHORT], 1),
        "the long row is dropped"
    );
    rig.ask(WordsAsk::ListPick(0));
    assert_eq!(rig.typed_words(), [(2, SHORT.to_owned())]);
    rig.typed(" so");
    let kinds: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    assert!(!kinds.contains(&ChipKind::Email), "{kinds:?}");
}

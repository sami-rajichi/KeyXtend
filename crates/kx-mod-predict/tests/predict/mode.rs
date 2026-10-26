//! The keyboard's typing mode: asked for at each start, as the keyboard may have started first, and its text cap
//! drops every chip the keyboard would refuse, emails included.

use crate::rig::{Heard, Rig, plant};
use kx_mod_predict::Msg;
use kx_mod_predict::email::Emails;
use kx_module_api::ports::{ChipKind, FieldKind, TypingMode, WordsAsk};

/// A long known email (19 characters) and a short one (10).
const LONG: &str = "someone@example.org";
const SHORT: &str = "sami@x.org";

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
fn a_word_chip_past_the_cap_is_dropped() {
    let mut rig = ready();
    rig.send(Msg::Mode(capped(5)));
    rig.typed("hel");
    assert_eq!(rig.words(), ["help"], "hello and its space take 6");
    rig.send(Msg::Mode(TypingMode::default()));
    assert_eq!(rig.words(), ["hello", "help"], "no cap told, none dropped");
}

#[test]
fn an_email_chip_or_row_past_the_cap_is_dropped_or_sends_nothing() {
    let mut rig = ready();
    rig.send(Msg::Mode(capped(12)));
    rig.typed("@@");
    let emails: Vec<String> = rig.chips().into_iter().map(|c| c.0).collect();
    assert_eq!(emails, [SHORT], "the long one does not fit");
    rig.ask(WordsAsk::More { rows: 5 });
    rig.ask(WordsAsk::ListPick(0));
    assert!(rig.typed_words().is_empty(), "the long row types nothing");
    assert_ne!(
        rig.lane().lane,
        kx_module_api::ports::Lane::Empty,
        "no pick waits"
    );
    rig.typed(" so");
    let kinds: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    assert!(!kinds.contains(&ChipKind::Email), "{kinds:?}");
}

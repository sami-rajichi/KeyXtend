//! The menu: Pause and Off are saved through the settings handle without a restart, Forget then Undo restore the word
//! by its stored spelling, forget mode offers word and known-email chips, the Undo chip goes at the next key, and
//! Again tells the lane again.

use crate::rig::{Rig, plant, saved};
use kx_mod_predict::email::Emails;
use kx_mod_predict::score::Learned;
use kx_mod_predict::{ID, Msg, emails_blob};
use kx_module_api::ports::{ChipKind, FieldKind, Lane, TypingMode, WordsAsk};
use toml::{Table, Value};

/// A rig in English whose learner knows `hello` and `help`, in a text box.
fn ready() -> Rig {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["hello", "help"]));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

/// The one-key write the settings handle made.
fn write(key: &str, on: bool) -> (kx_module_api::ModuleId, Table) {
    (ID, Table::from_iter([(key.to_owned(), Value::Boolean(on))]))
}

#[test]
fn pause_is_saved_and_a_paused_learner_learns_nothing() {
    let mut rig = ready();
    rig.ask(WordsAsk::Pause(true));
    assert_eq!(rig.host.sink.writes(), [write("learn", false)]);
    assert!(!rig.lane().learn && rig.lane().on);
    rig.typed("hello ");
    rig.stop();
    assert_eq!(rig.fakes.store.saves(), 0, "nothing learned, nothing saved");
    let mut rig = ready();
    rig.ask(WordsAsk::Pause(true));
    rig.ask(WordsAsk::Pause(false));
    assert_eq!(rig.host.sink.writes().last(), Some(&write("learn", true)));
    assert!(rig.lane().learn);
}

#[test]
fn off_is_saved_empties_the_lane_and_on_brings_the_chips_back() {
    let mut rig = ready();
    rig.ask(WordsAsk::Power(false));
    assert_eq!(rig.host.sink.writes(), [write("on", false)]);
    assert_eq!(rig.lane().lane, Lane::Off);
    assert!(!rig.lane().on);
    rig.typed("hel");
    assert_eq!(rig.lane().lane, Lane::Off, "typing changes nothing");
    rig.ask(WordsAsk::Power(true));
    assert_eq!(rig.host.sink.writes().last(), Some(&write("on", true)));
    rig.typed("hel");
    assert_eq!(rig.words(), ["hello", "help"]);
}

#[test]
fn forget_then_undo_restores_the_word_and_the_undo_chip_goes_at_the_next_key() {
    let mut rig = ready();
    rig.typed("hel");
    rig.ask(WordsAsk::Forget);
    assert!(rig.lane().forgetting);
    rig.pick(0);
    let after: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    assert_eq!(after, [ChipKind::Undo, ChipKind::Word]);
    assert_eq!(rig.words(), ["help"], "hello is gone");
    assert!(!rig.lane().forgetting && rig.typed_words().is_empty());
    rig.pick(0);
    assert_eq!(
        rig.words(),
        ["hello", "help"],
        "the Undo chip brings it back"
    );
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    rig.ask(WordsAsk::Undo);
    assert_eq!(rig.words(), ["hello", "help"], "the menu's Undo too");
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    rig.typed("p");
    assert!(
        rig.chips().iter().all(|c| c.1 != ChipKind::Undo),
        "gone at the next key"
    );
    rig.ask(WordsAsk::Undo);
    rig.stop();
    let learner = saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("hello"), 0, "too late to undo");
}

#[test]
fn forget_takes_the_stored_spelling_not_the_capitalised_chip() {
    let mut rig = Rig::build(|f, _| plant(f, "en", &["may"]));
    rig.lang("en");
    rig.focus(FieldKind::Text);
    let shift = TypingMode {
        shift: true,
        ..TypingMode::default()
    };
    rig.send(Msg::Mode(shift));
    rig.typed("M");
    rig.send(Msg::Mode(TypingMode::default()));
    assert_eq!(rig.words(), ["May"]);
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    rig.stop();
    let learner = saved(&rig.fakes, "en").unwrap();
    assert_eq!(learner.view(None).count("may"), 0);
}

#[test]
fn cancel_leaves_forget_mode_and_a_word_chip_then_types() {
    let mut rig = ready();
    rig.typed("hel");
    rig.ask(WordsAsk::Forget);
    rig.ask(WordsAsk::Cancel);
    assert!(!rig.lane().forgetting);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(3, "hello ".to_owned())]);
}

#[test]
fn again_tells_the_same_lane_once_more() {
    let mut rig = ready();
    rig.typed("hel");
    let before = rig.lane();
    rig.take();
    rig.ask(WordsAsk::Again);
    assert_eq!(rig.take(), [crate::rig::Heard::Lane(before)]);
}

/// A rig in English whose learner knows `sad` and `safe` and that knows the emails `known`, in a text box.
fn knowing(known: &[&str]) -> Rig {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["sad", "safe"]);
        let known: Vec<String> = known.iter().map(|e| (*e).to_owned()).collect();
        let bytes = Emails::new(&known, usize::MAX).to_bytes();
        f.store.plant(&emails_blob().unwrap(), bytes);
    });
    rig.lang("en");
    rig.focus(FieldKind::Text);
    rig
}

/// The emails saved now.
fn saved_emails(rig: &Rig) -> Vec<String> {
    let bytes = rig.fakes.store.blob(&emails_blob().unwrap()).unwrap();
    Emails::from_bytes(&bytes, usize::MAX)
        .unwrap()
        .list()
        .to_vec()
}

#[test]
fn forget_mode_offers_word_and_email_chips_and_a_pick_forgets_the_one_shown() {
    let mut rig = knowing(&["sami@x.org"]);
    rig.typed("sa");
    let words = rig.words();
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    let kinds: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    assert_eq!(kinds, [ChipKind::Undo, ChipKind::Word, ChipKind::Email]);
    rig.ask(WordsAsk::Forget);
    let email = ("sami@x.org".to_owned(), ChipKind::Email);
    let kept = [(words[1].clone(), ChipKind::Word), email];
    assert_eq!(rig.chips(), kept, "no Undo");
    rig.pick(0);
    assert!(rig.words().is_empty(), "the word shown was forgotten");
}

#[test]
fn a_known_email_picked_in_forget_mode_is_forgotten_and_saved() {
    let mut rig = knowing(&["sami@x.org", "sara@y.com"]);
    rig.typed("sa");
    rig.ask(WordsAsk::Forget);
    let at = rig.chips().iter().position(|c| c.0 == "sami@x.org");
    rig.pick(u8::try_from(at.unwrap()).unwrap());
    assert!(!rig.lane().forgetting && rig.typed_words().is_empty());
    let shown: Vec<String> = rig.chips().into_iter().map(|c| c.0).collect();
    assert!(!shown.contains(&"sami@x.org".to_owned()), "{shown:?}");
    rig.advance(std::time::Duration::from_secs(rig.settings.save_s.into()));
    rig.tick();
    assert_eq!(saved_emails(&rig), ["sara@y.com"]);
}

#[test]
fn forget_mode_drops_domains_and_more() {
    let mut rig = knowing(&[]);
    rig.typed("sami@");
    rig.ask(WordsAsk::Forget);
    assert!(rig.chips().is_empty(), "no domain");
    let seven: Vec<String> = (0..7).map(|n| format!("user{n}@mail.org")).collect();
    let seven: Vec<&str> = seven.iter().map(String::as_str).collect();
    let mut rig = knowing(&seven);
    rig.typed("@@");
    rig.ask(WordsAsk::Forget);
    let kinds: Vec<ChipKind> = rig.chips().into_iter().map(|c| c.1).collect();
    assert!(!kinds.is_empty() && kinds.iter().all(|k| *k == ChipKind::Email));
}

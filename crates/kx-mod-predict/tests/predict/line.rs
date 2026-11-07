//! A panel's line (ADR-0040 decisions 6 to 8): a search is read from its own text and never from the box, whatever the
//! app behind holds; a private line shows nothing; and when the aim leaves, the lane follows the app's box again.

mod boxes;
mod emails;
mod never;
mod picks;
mod props;

use crate::rig::{Heard, Rig, plant};
use kx_mod_predict::{Msg, ON_KEY};
use kx_module_api::Redacted;
use kx_module_api::ports::{ChipKind, FieldKind, Lane, PanelLine, WordsAsk};
use toml::Value;

impl Rig {
    /// The keyboard tells the line it aims at.
    pub fn line(&mut self, line: PanelLine) {
        self.send(Msg::Line(line));
    }

    /// Whether each word the actor asked the keyboard to type was offered for a line.
    pub fn word_targets(&self) -> Vec<bool> {
        let words = self.all().into_iter().filter_map(|h| match h {
            Heard::Word(w) => Some(w.line),
            _ => None,
        });
        words.collect()
    }
}

/// A search holding `text`.
pub fn search(text: &str) -> PanelLine {
    PanelLine::Search(Redacted::new(text.to_owned()))
}

/// A rig in `iso` whose learner knows `words`, in a text box that tells `before`.
pub fn in_box(iso: &str, words: &[&str], before: &str) -> Rig {
    let mut rig = Rig::build(|f, _| {
        plant(f, iso, words);
        f.field.set_before(Some(before));
    });
    rig.lang(iso);
    rig.focus(FieldKind::Text);
    rig
}

/// A rig in English whose learner knows `hello` and `help`, aimed at a search holding `text`.
pub fn aimed(text: &str) -> Rig {
    let mut rig = in_box("en", &["hello", "help"], "");
    rig.line(search(text));
    rig
}

#[test]
fn a_search_shows_chips_for_its_open_word_in_english_french_and_arabic() {
    let cases = [
        ("en", "hello", "hel"),
        ("fr", "école", "éc"),
        ("ar", "مرحبا", "مر"),
    ];
    for (iso, word, typed) in cases {
        let mut rig = in_box(iso, &[word], "");
        rig.line(search(typed));
        assert_eq!(rig.words(), [word], "{iso}");
    }
}

#[test]
fn each_new_text_brings_a_new_round_for_its_open_word() {
    let mut rig = aimed("he");
    let old = rig.lane().round;
    assert_eq!(rig.words(), ["hello", "help"]);
    rig.line(search("hell"));
    assert_ne!(rig.lane().round, old);
    assert_eq!(rig.words(), ["hello"]);
}

#[test]
fn an_earlier_word_ends_and_a_space_at_the_end_asks_for_the_next_word() {
    let mut rig = aimed("hell");
    assert_eq!(rig.words(), ["hello"]);
    rig.line(search("hell "));
    assert_eq!(
        rig.words(),
        ["hello", "help"],
        "next words, nothing to erase"
    );
    rig.line(search("pizza hell"));
    assert_eq!(rig.words(), ["hello"], "the open word is the last");
}

#[test]
fn a_change_of_line_clears_the_hold_and_the_undo_chip() {
    let mut rig = in_box("en", &["hello", "help"], "hel");
    rig.pick(0);
    assert_eq!(rig.lane().lane, Lane::Empty, "a pick in the box holds");
    rig.line(search("hel"));
    assert_ne!(rig.lane().lane, Lane::Empty, "the line's lane does not");
    rig.ask(WordsAsk::Forget);
    rig.pick(0);
    assert!(rig.chips().iter().any(|c| c.1 == ChipKind::Undo));
    rig.line(search("he"));
    assert!(rig.chips().iter().all(|c| c.1 != ChipKind::Undo));
}

#[test]
fn the_same_line_again_changes_nothing() {
    let mut rig = aimed("hel");
    let round = rig.lane().round;
    rig.line(search("hel"));
    assert_eq!(
        rig.lane().round,
        round,
        "a re-ask's answer tells no new lane"
    );
}

#[test]
fn a_language_switch_on_a_search_reads_its_text_again_in_the_new_language() {
    let mut rig = Rig::build(|f, _| {
        plant(f, "en", &["hello"]);
        plant(f, "fr", &["helvète"]);
    });
    rig.lang("en");
    rig.line(search("hel"));
    assert_eq!(rig.words(), ["hello"]);
    let reads = rig.fakes.field.before_calls();
    rig.lang("fr");
    assert_eq!(rig.words(), ["helvète"]);
    assert_eq!(rig.fakes.field.before_calls(), reads);
}

#[test]
fn prediction_off_shows_off_on_any_line_and_holds_no_text() {
    for line in [search("hel"), PanelLine::Private, PanelLine::None] {
        let mut rig = in_box("en", &["hello", "help"], "");
        rig.ask(WordsAsk::Power(false));
        rig.line(line.clone());
        assert_eq!(rig.lane().lane, Lane::Off, "{line:?}");
        rig.fakes.field.set_before(Some("xyz"));
        rig.ask(WordsAsk::Power(true));
        assert!(
            rig.words().is_empty(),
            "the app's box; the line was not kept"
        );
        rig.pick(0);
        assert!(rig.typed_words().is_empty());
    }
}

#[test]
fn turning_off_drops_the_line_and_turning_on_asks_for_it_again() {
    let mut rig = aimed("hel");
    let asks = |rig: &Rig| {
        let all = rig.all().into_iter();
        all.filter(|h| *h == Heard::ModeAsk).count()
    };
    let before = asks(&rig);
    rig.ask(WordsAsk::Power(false));
    assert_eq!(rig.lane().lane, Lane::Off);
    assert_eq!(asks(&rig), before, "off asks nothing");
    rig.fakes.field.set_before(Some("xyz"));
    rig.ask(WordsAsk::Power(true));
    assert_eq!(asks(&rig), before + 1);
    assert!(
        rig.words().is_empty(),
        "the app's box, until the keyboard tells"
    );
    rig.line(search("hel"));
    assert_eq!(rig.words(), ["hello", "help"]);
}

#[test]
fn a_start_with_prediction_off_keeps_no_line() {
    let mut rig = Rig::build(|f, host| {
        plant(f, "en", &["hello"]);
        host.put(ON_KEY, Value::Boolean(false));
    });
    rig.lang("en");
    rig.line(search("hel"));
    assert_eq!(rig.lane().lane, Lane::Off);
}

#[test]
fn a_text_longer_than_a_read_is_cut_and_its_open_word_stays_open() {
    let mut rig = in_box("en", &["hello", "help"], "");
    let long = format!("{} hel", "a ".repeat(usize::from(rig.settings.read_chars)));
    rig.line(search(&long));
    assert_eq!(rig.words(), ["hello", "help"]);
    rig.pick(0);
    assert_eq!(rig.typed_words(), [(0, "lo ".to_owned())]);
    assert_eq!(rig.word_targets(), [true]);
}

#[test]
fn one_word_longer_than_a_read_leaves_no_open_word_as_a_platform_read_does() {
    let mut rig = in_box("en", &["hello", "help"], "");
    let chars = usize::from(rig.settings.read_chars);
    rig.line(search(&"a".repeat(chars + 1)));
    assert_eq!(rig.words(), ["hello", "help"], "next words, nothing split");
}

#[test]
fn a_search_of_only_spaces_or_nothing_offers_next_words() {
    for text in ["", " ", "     "] {
        let rig = aimed(text);
        assert_eq!(rig.words(), ["hello", "help"], "{text:?}");
    }
}

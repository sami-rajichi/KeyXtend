//! The tracker: words end at spaces, marks and breaks; reads fill the context; apostrophes split as the packs do;
//! emails and `@@`; and the plans a pick sends.

use kx_module_api::Redacted;
use kx_module_api::ports::{FieldChange, FieldKind, Piece};

use super::*;

mod emails;
mod ends;
mod reads;

/// The words a test language knows: elisions and whole words that hold an apostrophe.
const KNOWN: [&str; 4] = ["l'", "qu'", "quelqu'un", "aujourd'hui"];
/// The longest word the test tracker guesses and learns, in characters.
const MAX_WORD: usize = 48;

/// True for the test language's words, in any case.
fn knows(word: &str) -> bool {
    KNOWN.contains(&word.to_lowercase().as_str())
}

/// A tracker keeping `n` words before the current one.
fn tracker(n: usize) -> Tracker {
    Tracker::new(TrackConfig {
        context_words: n,
        max_word: MAX_WORD,
    })
}

/// Types `text` one character at a time, as the keyboard tells it, and gathers what the tracker hands out.
fn type_text(t: &mut Tracker, text: &str) -> Vec<Told> {
    let mut told = Vec::new();
    for c in text.chars() {
        let piece = Piece::Text(Redacted::new(c.to_string()));
        told.extend(t.typed(&piece, &knows));
    }
    told
}

/// The words handed out for learning, each with its context.
fn learned(told: &[Told]) -> Vec<(String, Vec<String>)> {
    let each = |t: &Told| match t {
        Told::Learn { word, context } => Some((word.expose().clone(), context.expose().clone())),
        _ => None,
    };
    told.iter().filter_map(each).collect()
}

/// Each reading of the word guesses: its context, lead and prefix.
fn readings(show: &Show) -> Vec<(Vec<String>, String, String)> {
    let each = |r: &Reading| {
        let (c, l, p) = (r.context.expose(), r.lead.expose(), r.prefix.expose());
        (c.clone(), l.clone(), p.clone())
    };
    show.words
        .iter()
        .flat_map(|q| q.readings.iter().map(each))
        .collect()
}

/// `words` as owned strings.
fn owned(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

#[test]
fn hello_world_hands_out_both_words_and_keeps_them_in_context() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "hello world ");
    let expected = [
        ("hello".to_owned(), owned(&[])),
        ("world".to_owned(), owned(&["hello"])),
    ];
    assert_eq!(learned(&told), expected);
    assert_eq!(t.context(), Some(owned(&["hello", "world"]).as_slice()));
    assert_eq!(
        readings(&t.show(&knows)),
        [(owned(&["hello", "world"]), String::new(), String::new())]
    );
}

#[test]
fn the_context_keeps_only_the_last_words() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "one two three ");
    assert_eq!(
        learned(&told).last().map(|l| l.1.clone()),
        Some(owned(&["one", "two"]))
    );
    assert_eq!(t.context(), Some(owned(&["two", "three"]).as_slice()));
}

#[test]
fn an_erase_past_the_token_start_marks_the_context_unknown_and_asks_for_a_read() {
    let mut t = tracker(2);
    type_text(&mut t, "hi");
    assert!(t.typed(&Piece::Erase, &knows).is_empty());
    assert!(t.typed(&Piece::Erase, &knows).is_empty());
    assert_eq!(t.typed(&Piece::Erase, &knows), [Told::Read]);
    assert_eq!(t.context(), None);
    type_text(&mut t, "go ");
    assert_eq!(t.typed(&Piece::Erase, &knows), [Told::Read]);
    assert_eq!(t.context(), None);
}

#[test]
fn break_click_new_box_and_layout_switch_reset_unlearned_and_ask_for_a_read() {
    let resets: [fn(&mut Tracker) -> Vec<Told>; 3] = [
        |t| t.typed(&Piece::Break, &knows),
        Tracker::reset,
        |t| {
            t.focus(FieldChange {
                kind: FieldKind::Text,
                box_id: 7,
            })
        },
    ];
    for reset in resets {
        let mut t = tracker(2);
        type_text(&mut t, "one hel");
        assert_eq!(reset(&mut t), [Told::Read]);
        assert_eq!(t.context(), None);
        assert!(!t.holds());
        assert_eq!(
            readings(&t.show(&knows)),
            [(owned(&[]), String::new(), String::new())]
        );
    }
}

#[test]
fn the_same_box_again_changes_nothing() {
    let mut t = tracker(2);
    let change = FieldChange {
        kind: FieldKind::Text,
        box_id: 7,
    };
    assert_eq!(t.focus(change), [Told::Read]);
    type_text(&mut t, "one ");
    assert!(t.focus(change).is_empty());
    assert_eq!(t.context(), Some(owned(&["one"]).as_slice()));
}

#[test]
fn digits_mixed_scripts_and_long_tokens_get_no_word_guesses_and_are_never_learned() {
    for token in ["abc123", "abcعربي", &"a".repeat(MAX_WORD + 1)] {
        let mut t = tracker(2);
        type_text(&mut t, "one ");
        type_text(&mut t, token);
        assert_eq!(t.show(&knows).words, None, "{token}");
        let told = type_text(&mut t, " ");
        assert!(learned(&told).is_empty(), "{token}");
        assert_eq!(t.context(), Some(&[][..]), "{token}");
    }
    let mut t = tracker(2);
    type_text(&mut t, &"a".repeat(MAX_WORD));
    assert!(t.show(&knows).words.is_some());
}

#[test]
fn an_elision_splits_off_and_the_chain_runs_through_it() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "vois l'école ");
    let expected = [
        ("vois".to_owned(), owned(&[])),
        ("l'".to_owned(), owned(&["vois"])),
        ("école".to_owned(), owned(&["vois", "l'"])),
    ];
    assert_eq!(learned(&told), expected);
}

#[test]
fn a_known_whole_word_stays_whole_and_an_unknown_apostrophe_glues_unlearned() {
    let mut t = tracker(2);
    let told = type_text(&mut t, "quelqu'un Muiriel's ");
    assert_eq!(learned(&told), [("quelqu'un".to_owned(), owned(&[]))]);
    assert_eq!(t.context(), Some(&[][..]));
}

#[test]
fn an_open_elision_reads_whole_and_split() {
    let mut t = tracker(2);
    type_text(&mut t, "vois l'");
    let expected = [
        (owned(&["vois"]), String::new(), "l'".to_owned()),
        (owned(&["vois", "l'"]), "l'".to_owned(), String::new()),
    ];
    assert_eq!(readings(&t.show(&knows)), expected);
}

#[test]
fn a_typographic_apostrophe_keeps_its_typed_form_in_the_lead() {
    let mut t = tracker(2);
    type_text(&mut t, "L\u{2019}éc");
    let expected = [
        (owned(&[]), String::new(), "L\u{2019}éc".to_owned()),
        (owned(&["L'"]), "L\u{2019}".to_owned(), "éc".to_owned()),
    ];
    assert_eq!(readings(&t.show(&knows)), expected);
}

#[test]
fn a_glued_tail_gets_no_split_reading() {
    let mut t = tracker(2);
    type_text(&mut t, "Muiriel's");
    let found = readings(&t.show(&knows));
    assert_eq!(found, [(owned(&[]), String::new(), "Muiriel's".to_owned())]);
}

#[test]
fn an_arabic_word_with_harakat_is_one_word_and_its_plan_erases_every_character() {
    let mut t = tracker(2);
    let word = "كَتَبَ";
    let told = type_text(&mut t, &format!("{word} "));
    assert_eq!(learned(&told), [(word.to_owned(), owned(&[]))]);
    type_text(&mut t, "ان");
    let query = t.show(&knows).words.unwrap();
    assert_eq!(query.plan("أن").map(|p| p.erase), Some(2));
    assert_eq!(
        readings(&t.show(&knows)),
        [(owned(&[word]), String::new(), "ان".to_owned())]
    );
}

#[test]
fn a_tracker_holds_something_once_it_knows_a_word_or_a_token() {
    let mut t = tracker(2);
    assert!(!t.holds());
    type_text(&mut t, "h");
    assert!(t.holds());
    t.reset();
    type_text(&mut t, " ");
    assert!(!t.holds());
}

#[test]
fn debug_never_shows_the_words() {
    let mut t = tracker(2);
    type_text(&mut t, "secret hidden");
    assert!(!format!("{t:?}").contains("secret"));
    assert!(!format!("{:?}", t.show(&knows)).contains("hidden"));
}

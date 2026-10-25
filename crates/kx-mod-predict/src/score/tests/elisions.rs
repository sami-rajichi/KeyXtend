//! Elisions through the tracker and the scorer: a split reading, its weight and its followers.

use kx_module_api::ports::Piece;

use super::*;
use crate::track::{TrackConfig, Tracker};

/// The query after typing `text` in French, and its guesses as the module asks for them.
fn typed_fr(text: &str) -> (Query, Vec<String>) {
    let fr = french();
    let pack = Pack::parse(&fr).unwrap();
    let mut tracker = Tracker::new(TrackConfig {
        context_words: 2,
        max_word: 48,
    });
    let known = |w: &str| knows(Some(&pack), w);
    tracker.typed(&Piece::Text(Redacted::new(text.to_owned())), &known);
    let query = tracker.show(&known).words.unwrap();
    let found = guesses(
        pack.iso(),
        Some(&pack),
        &NoLearned,
        &query,
        TypingMode::default(),
        LIMIT,
        &CFG,
    );
    (query, texts(&found))
}

#[test]
fn the_tracker_and_the_scorer_finish_an_elided_word() {
    let (query, found) = typed_fr("vois l'éc");
    assert_eq!(found.first().map(String::as_str), Some("l'école"));
    let plan = query.plan("l'école").unwrap();
    assert_eq!((plan.erase, plan.text.expose().as_str()), (4, "l'école "));
}

#[test]
fn a_split_reading_weighs_its_elision_so_whole_words_come_first() {
    let (_, found) = typed_fr("jusqu'a");
    let at = |w: &str| found.iter().position(|t| t == w);
    assert_eq!(found.first().map(String::as_str), Some("jusqu'à"));
    assert!(
        at("jusqu'a").is_some() && at("jusqu'au") < at("jusqu'a"),
        "{found:?}"
    );
}

#[test]
fn a_typed_apostrophe_form_is_kept_whichever_reading_wins() {
    let (_, found) = typed_fr("jusqu\u{2019}a");
    assert!(found.iter().all(|t| !t.contains('\'')), "{found:?}");
    let once = found.iter().filter(|t| *t == "jusqu\u{2019}à").count();
    assert_eq!(once, 1, "{found:?}");
}

#[test]
fn after_an_elision_a_frequent_word_that_never_follows_it_is_not_offered() {
    let (_, found) = typed_fr("vois l'e");
    assert!(found.contains(&"l'école".to_owned()), "{found:?}");
    assert!(!found.contains(&"l'et".to_owned()), "{found:?}");
    let (_, whole) = typed_fr("quelqu'un");
    assert!(whole.contains(&"quelqu'un".to_owned()), "{whole:?}");
    assert!(!whole.iter().any(|t| t == "quelqu'unis"), "{whole:?}");
}

#[test]
fn a_prefix_no_follower_starts_still_gets_word_list_guesses() {
    let (_, found) = typed_fr("vois l'h");
    assert_eq!(found.first().map(String::as_str), Some("l'homme"));
}

#[test]
fn after_an_elision_only_its_followers_come_and_the_elision_itself_is_not_offered() {
    assert_eq!(typed_fr("vois l'").1, ["l'école"]);
}

#[test]
fn an_elision_without_followers_shows_nothing_until_a_letter_is_typed() {
    assert_eq!(typed_fr("lorsqu'").1, Vec::<String>::new());
}

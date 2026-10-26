//! The lane's chips: words before emails within the lane's size, each chip with its pick, and the Undo chip's place.

use kx_module_api::ports::Piece;

use super::*;
use crate::learn::Learner;
use crate::learn::tests::{CFG, DAY};
use crate::score::CFG as SCORE;
use crate::settings::PredictSettings;
use crate::track::Tracker;

/// The chips after typing `text`, with `words` learned twice and `known` emails, at most `limit`.
fn lane(text: &str, words: &[&str], known: &[&str], limit: usize) -> Chips {
    let mut learner = Learner::new("en", CFG).unwrap();
    for word in words.iter().chain(words) {
        learner.learn(None, word, &[], DAY);
    }
    let known: Vec<String> = known.iter().map(|e| (*e).to_owned()).collect();
    let settings = PredictSettings::from_defaults().unwrap();
    let mut tracker = Tracker::new(settings.track_config());
    let unknown = |_: &str| false;
    tracker.typed(&Piece::Text(Redacted::new(text.to_owned())), &unknown);
    let view = learner.view(None);
    let (emails, email) = (Emails::new(&known, usize::MAX), settings.email_config());
    let src = Sources {
        iso: "en",
        seed: None,
        learned: &view,
        mode: TypingMode::default(),
        score: &SCORE,
        emails: &emails,
        email: &email,
        limit,
        confirms: true,
        forgetting: false,
    };
    Chips::of(&tracker.show(&unknown), &src)
}

/// Each chip's text and kind.
fn seen(chips: &Chips) -> Vec<(&str, ChipKind)> {
    let shown = chips.chips.iter();
    shown.map(|c| (c.text.expose().as_str(), c.kind)).collect()
}

#[test]
fn words_come_before_emails_and_emails_keep_their_room() {
    let chips = lane("sa", &["same", "say", "salt"], &["sami@x.org"], 3);
    let want = [
        ("salt", ChipKind::Word),
        ("same", ChipKind::Word),
        ("sami@x.org", ChipKind::Email),
    ];
    assert_eq!(seen(&chips), want);
    assert_eq!(chips.picks.len(), chips.chips.len());
}

#[test]
fn each_pick_carries_its_plan_and_a_word_its_stored_spelling() {
    let chips = lane("Sa", &["salt"], &[], 5);
    let Some(Pick::Word { plan, stored }) = chips.picks.first() else {
        panic!("a word chip")
    };
    assert_eq!((plan.erase, plan.text.expose().as_str()), (2, "Salt "));
    assert_eq!(stored.expose(), "salt");
    let at = lane("@@", &[], &["a@x.org", "b@x.org", "c@x.org"], 2);
    let kinds: Vec<ChipKind> = at.chips.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [ChipKind::Email, ChipKind::More]);
    assert_eq!(at.picks.last(), Some(&Pick::More));
}

#[test]
fn the_undo_chip_takes_its_place_and_the_lane_keeps_its_size() {
    let mut chips = lane("sa", &["same", "say", "salt"], &[], 3);
    chips.with_undo(1, false, 3);
    let kinds: Vec<ChipKind> = chips.chips.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [ChipKind::Word, ChipKind::Undo, ChipKind::Word]);
    assert_eq!(chips.picks.get(1), Some(&Pick::Undo));
    let mut short = Chips::default();
    short.with_undo(4, true, 3);
    assert_eq!(
        short.chips.first().map(|c| (c.kind, c.rtl)),
        Some((ChipKind::Undo, true))
    );
}

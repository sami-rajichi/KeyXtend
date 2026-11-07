//! The lane's chips: words before emails within the lane's size, each chip with its pick, saved emails as their own
//! picks, and the Undo chip's place.

use kx_module_api::ports::Piece;
use kx_module_api::{Secret, Zeroizing};

use super::*;
use crate::email::Saved;
use crate::learn::Learner;
use crate::learn::tests::{CFG, DAY};
use crate::score::CFG as SCORE;
use crate::settings::PredictSettings;
use crate::track::Tracker;

/// The emails a lane draws from: typed ones, the notebook's, and whether forget mode is on.
#[derive(Clone, Copy, Default)]
struct Known<'a> {
    typed: &'a [&'a str],
    saved: &'a [&'a str],
    forgetting: bool,
}

/// The chips after typing `text`, with `words` learned twice and `known` emails, at most `limit`.
fn lane(text: &str, words: &[&str], known: Known<'_>, limit: usize) -> Chips {
    let mut learner = Learner::new("en", CFG).unwrap();
    for word in words.iter().chain(words) {
        learner.learn(None, word, &[], DAY);
    }
    let typed: Vec<String> = known.typed.iter().map(|e| (*e).to_owned()).collect();
    let secret = |e: &&str| -> Secret { Redacted::new(Zeroizing::new((*e).to_owned())) };
    let saved = Saved::new(known.saved.iter().map(secret).collect());
    let settings = PredictSettings::from_defaults().unwrap();
    let mut tracker = Tracker::new(settings.track_config());
    let unknown = |_: &str| false;
    tracker.typed(&Piece::Text(Redacted::new(text.to_owned())), &unknown);
    let view = learner.view(None);
    let (emails, email) = (Emails::new(&typed, usize::MAX), settings.email_config());
    let src = Sources {
        iso: "en",
        seed: None,
        learned: &view,
        mode: TypingMode::default(),
        score: &SCORE,
        emails: &emails,
        saved: &saved,
        email: &email,
        limit,
        confirms: true,
        forgetting: known.forgetting,
    };
    Chips::of(&tracker.show(&unknown), &src)
}

/// Typed emails only.
fn typed<'a>(typed: &'a [&'a str]) -> Known<'a> {
    Known {
        typed,
        ..Known::default()
    }
}

/// Each chip's text and kind.
fn seen(chips: &Chips) -> Vec<(&str, ChipKind)> {
    let shown = chips.chips.iter();
    shown.map(|c| (c.text.expose().as_str(), c.kind)).collect()
}

#[test]
fn words_come_before_emails_and_emails_keep_their_room() {
    let chips = lane("al", &["also", "alto", "alps"], typed(&["alex@x.org"]), 3);
    let want = [
        ("alps", ChipKind::Word),
        ("also", ChipKind::Word),
        ("alex@x.org", ChipKind::Email),
    ];
    assert_eq!(seen(&chips), want);
    assert_eq!(chips.picks.len(), chips.chips.len());
}

#[test]
fn each_pick_carries_its_plan_and_a_word_its_stored_spelling() {
    let chips = lane("Sa", &["salt"], Known::default(), 5);
    let Some(Pick::Word { plan, stored }) = chips.picks.first() else {
        panic!("a word chip")
    };
    assert_eq!((plan.erase, plan.text.expose().as_str()), (0, "lt "));
    assert_eq!(stored.expose(), "salt");
    let at = lane("@@", &[], typed(&["a@x.org", "b@x.org", "c@x.org"]), 2);
    let kinds: Vec<ChipKind> = at.chips.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [ChipKind::Email, ChipKind::More]);
    assert_eq!(at.picks.last(), Some(&Pick::More));
}

#[test]
fn a_saved_email_is_a_saved_pick_after_at_at_or_its_start_and_forget_mode_leaves_it_out() {
    let both = Known {
        typed: &["a@x.org"],
        saved: &["alex@y.org"],
        forgetting: false,
    };
    let at = lane("@@", &[], both, 5);
    let want = [
        ("a@x.org", ChipKind::Email),
        ("alex@y.org", ChipKind::Saved),
    ];
    assert_eq!(seen(&at), want);
    let Some(Pick::Saved(plan)) = at.picks.get(1) else {
        panic!("a saved pick")
    };
    assert_eq!((plan.erase, plan.text.expose().as_str()), (2, "alex@y.org"));
    let start = lane("al", &[], both, 5);
    assert_eq!(seen(&start), [("alex@y.org", ChipKind::Saved)]);
    let Some(Pick::Saved(plan)) = start.picks.first() else {
        panic!("a saved pick")
    };
    assert_eq!(plan.text.expose(), "ex@y.org");
    let forget = Known {
        forgetting: true,
        ..both
    };
    let left = lane("@@", &[], forget, 5);
    assert_eq!(
        seen(&left),
        [("a@x.org", ChipKind::Email)],
        "not ours to forget"
    );
}

#[test]
fn the_undo_chip_takes_its_place_and_the_lane_keeps_its_size() {
    let mut chips = lane("sa", &["same", "say", "salt"], Known::default(), 3);
    chips.with_undo(1, 3);
    let kinds: Vec<ChipKind> = chips.chips.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [ChipKind::Word, ChipKind::Undo, ChipKind::Word]);
    assert_eq!(chips.picks.get(1), Some(&Pick::Undo));
    let mut short = Chips::default();
    short.with_undo(4, 3);
    assert_eq!(short.chips.first().map(|c| c.kind), Some(ChipKind::Undo));
}

//! Forgetting: a word goes with its chains, a seed word stays hidden until typed `min_uses` times, and the last
//! forget comes back exactly, once.

use super::*;

#[test]
fn forget_removes_a_word_and_every_chain_holding_it() {
    let mut l = learner("en");
    teach(&mut l, None, "in my park in my park", DAY);
    assert!(l.forget(None, "my", DAY));
    let view = l.view(None);
    assert_eq!(
        (view.count("my"), view.count("in"), view.count("park")),
        (0, 2, 2)
    );
    assert_eq!(view.chain(&["in"], "my"), 0);
    assert_eq!(view.chain(&["my"], "park"), 0);
    assert_eq!(view.chain(&["in", "my"], "park"), 0);
    assert_eq!(view.chain(&["park"], "in"), 1);
    assert_eq!(lengths(&l), [1, 0], "no chain names the forgotten word");
    assert!(!view.blocked("my"), "only a seed word is blocked");
    assert!(!l.forget(None, "my", DAY), "nothing left to forget");
    l.learn(None, "my", &[], DAY);
    assert!(
        guessed(&l, None, &[], "m").is_empty(),
        "the gate starts again"
    );
}

#[test]
fn undo_restores_the_word_and_its_chains_exactly_once() {
    let mut l = learner("en");
    teach(&mut l, None, "in my park in my park my", DAY);
    let before = l.clone();
    assert!(l.forget(None, "My", DAY + 1));
    assert_ne!(l, before);
    assert!(l.undo_forget());
    assert_eq!(l, before);
    assert!(!l.undo_forget());
    assert_eq!(l, before);
}

#[test]
fn undo_works_only_right_after_the_forget() {
    let mut l = learner("en");
    teach(&mut l, None, "in my park in my park", DAY);
    assert!(l.forget(None, "my", DAY));
    l.learn(None, "in", &[], DAY);
    assert!(!l.undo_forget());
    assert_eq!(l.view(None).count("my"), 0);
    assert!(l.forget(None, "park", DAY));
    l.purge(DAY);
    assert!(!l.undo_forget());
}

#[cfg(feature = "build")]
#[test]
fn a_forgotten_seed_word_is_hidden_until_typed_min_uses_times() {
    let bytes = super::seed::english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, Some(&en), "in the theory in the theory", DAY);
    assert!(l.forget(Some(&en), "The", DAY));
    let hidden = |l: &Learner| !guessed(l, Some(&en), &[], "th").contains(&"the".to_owned());
    assert!(l.view(Some(&en)).blocked("the") && hidden(&l));
    assert_eq!(l.view(Some(&en)).count("the"), 0);
    teach(&mut l, Some(&en), "in the", DAY + 1);
    assert!(
        l.view(Some(&en)).blocked("the") && hidden(&l),
        "one use is not enough"
    );
    assert_eq!(l.view(Some(&en)).chain(&["in"], "the"), 0);
    teach(&mut l, Some(&en), "in the", DAY + 2);
    let view = l.view(Some(&en));
    assert!(!view.blocked("the") && !hidden(&l));
    assert_eq!((view.count("the"), view.chain(&["in"], "the")), (2, 1));
}

#[cfg(feature = "build")]
#[test]
fn undo_restores_the_blocklist_and_a_seed_word_never_typed_can_be_forgotten() {
    let bytes = super::seed::english();
    let en = Pack::parse(&bytes).unwrap();
    let mut l = learner("en");
    teach(&mut l, Some(&en), "in the theory", DAY);
    let before = l.clone();
    assert!(l.forget(Some(&en), "theory", DAY));
    assert!(l.undo_forget());
    assert_eq!(l, before);
    assert!(l.forget(Some(&en), "london", DAY));
    assert!(l.view(Some(&en)).blocked("London"));
    assert!(!l.forget(Some(&en), "London", DAY), "already hidden");
    assert!(l.undo_forget());
    assert_eq!(l, before);
}

#[cfg(feature = "build")]
#[test]
fn the_blocklist_keeps_max_words_dropping_the_oldest_forgotten() {
    let words: Vec<String> = (0..12).map(word).collect();
    let listed: Vec<(&str, u8)> = words.iter().map(|w| (w.as_str(), 100)).collect();
    let bytes = super::seed::pack("en", &listed);
    let en = Pack::parse(&bytes).unwrap();
    let cfg = LearnConfig {
        max_words: 10,
        ..CFG
    };
    let mut l = Learner::new("en", cfg).unwrap();
    for (w, day) in words.iter().take(10).zip(DAY..) {
        assert!(l.forget(Some(&en), w, day));
    }
    let before = l.clone();
    assert!(l.forget(Some(&en), &words[10], DAY + 10));
    let blocked: Vec<bool> = words.iter().map(|w| l.view(Some(&en)).blocked(w)).collect();
    let mut want = vec![true; 11];
    want[0] = false;
    want.push(false);
    assert_eq!(blocked, want);
    assert!(l.undo_forget());
    assert_eq!(l, before);
}

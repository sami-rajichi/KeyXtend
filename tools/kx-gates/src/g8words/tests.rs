//! Tests of `g8words`.

use std::cell::{Cell, RefCell};

use super::*;
use crate::config::load;

fn memory(mb: f64) -> Memory {
    Memory {
        working_mb: mb,
        commit_mb: mb,
    }
}

fn typed(picked: usize, stopped: bool) -> Typed {
    Typed {
        words: 30,
        picked,
        before: memory(60.0),
        after: memory(61.0),
        note: None,
        stopped,
    }
}

fn case(lang: &str) -> Case {
    Case {
        lang: lang.into(),
        prefix: "x".into(),
    }
}

/// The languages pass `at` of `plan` types, in order.
fn langs<'p>(plan: &'p Plan, at: usize) -> Vec<&'p str> {
    let (skip, count) = plan.passes[at];
    let taken = turn(plan.cases, skip, count).map(|c| c.lang.as_str());
    taken.collect()
}

#[test]
fn the_churn_is_whole_only_when_every_measured_word_was_picked() {
    assert!(typed(30, false).whole());
    assert!(!typed(29, false).whole());
}

#[test]
fn the_judged_churn_types_only_the_case_of_its_language() {
    let c = load().expect("loads");
    let mut p = c.g8_predict.clone();
    for k in &c.g27.cases {
        p.lang.clone_from(&k.lang);
        let plan = Plan::judged(&c.g27, &p).expect("a case");
        let typed = [langs(&plan, 0), langs(&plan, 1)].concat();
        assert_eq!(typed.len(), p.warm_words + p.words, "{}", k.lang);
        assert!(typed.iter().all(|&l| l == k.lang), "{typed:?}");
        assert_eq!(plan.words(), p.warm_words + p.words);
        assert_eq!(plan.keys(), k.prefix.chars().count());
    }
    p.lang = "xx".into();
    let e = Plan::judged(&c.g27, &p).expect_err("no case");
    assert!(e.contains("g8_predict.lang"), "{e}");
}

#[test]
fn the_mixed_churn_warms_one_word_per_case_then_takes_the_cases_in_turn() {
    let c = load().expect("loads");
    let all: Vec<&str> = c.g27.cases.iter().map(|k| k.lang.as_str()).collect();
    assert!(all.len() > 1, "the shipped file has several cases");
    let mut p = c.g8_predict.clone();
    p.mixed_words = all.len() + 1;
    p.warm_words = all.len() + 2;
    let plan = Plan::mixed(&c.g27, &p);
    assert_eq!(langs(&plan, 0), all, "one warm-up word per case");
    let measured = [all.clone(), vec![all[0]]].concat();
    assert_eq!(langs(&plan, 1), measured, "the measured words go round");
    assert_eq!(plan.words(), all.len() + p.mixed_words);
    let longest = c.g27.cases.iter().map(|k| k.prefix.chars().count()).max();
    assert_eq!(Some(plan.keys()), longest);
}

#[test]
fn a_judged_churn_a_failed_word_end_stopped_runs_no_mixed_churn() {
    let churns = after(typed(0, true), || panic!("no mixed churn"));
    let e = churns.mixed.expect_err("not run");
    assert_eq!(e, NO_MIXED);
    let churns = after(typed(29, false), || Ok(typed(30, false)));
    assert_eq!(
        churns.mixed,
        Ok(typed(30, false)),
        "a missed word that ended"
    );
}

#[test]
fn a_missed_word_keeps_its_error_and_is_ended_before_the_next_starts() {
    let cases = [case("en"), case("fr"), case("ar")];
    let (typed, ended) = (RefCell::new(Vec::new()), RefCell::new(Vec::new()));
    let word = |c: &Case| {
        typed.borrow_mut().push(c.lang.clone());
        if c.lang == "fr" {
            Err("no chip".to_string())
        } else {
            Ok(())
        }
    };
    let end = |c: &Case| {
        ended.borrow_mut().push(c.lang.clone());
        Ok(())
    };
    let pass = tally(turn(&cases, 0, 4), word, end);
    assert_eq!((pass.picked, pass.note.as_deref()), (3, Some("no chip")));
    assert!(!pass.stopped);
    assert_eq!(*typed.borrow(), ["en", "fr", "ar", "en"]);
    assert_eq!(*ended.borrow(), ["fr"], "only the missed word is ended");
}

#[test]
fn a_warm_up_a_failed_end_stopped_measures_nothing_and_carries_its_note() {
    let warm = Pass {
        picked: 1,
        note: Some("miss; stopped".into()),
        stopped: true,
    };
    let typed = measure(30, (warm, memory(60.0)), || panic!("no measured pass"));
    let typed = typed.unwrap();
    assert_eq!((typed.picked, typed.after), (0, memory(60.0)));
    assert!(!typed.whole() && typed.stopped);
    let note = typed.note.expect("a note");
    assert!(note.contains(WARM_UP) && note.contains("miss"), "{note}");
}

#[test]
fn a_warm_up_that_went_on_is_followed_by_the_measured_words() {
    let warm = Pass {
        picked: 2,
        note: Some("miss".into()),
        stopped: false,
    };
    let measured = Pass {
        picked: 28,
        note: Some("late miss; stopped".into()),
        stopped: true,
    };
    let typed = measure(30, (warm, memory(60.0)), || Ok((measured, memory(61.0))));
    let typed = typed.unwrap();
    assert_eq!((typed.picked, typed.stopped), (28, true));
    assert_eq!(typed.note.as_deref(), Some("late miss; stopped"));
    assert_eq!((typed.before, typed.after), (memory(60.0), memory(61.0)));
}

#[test]
fn a_word_end_that_fails_starts_no_further_word_and_the_first_error_stays() {
    let cases = [case("en")];
    let tries = Cell::new(0);
    let word = |_: &Case| {
        tries.set(tries.get() + 1);
        Err(format!("miss {}", tries.get()))
    };
    let pass = tally(turn(&cases, 0, 5), word, |_| Err("no gap key".into()));
    assert_eq!(
        (pass.picked, tries.get()),
        (0, 1),
        "stopped after the first"
    );
    assert!(pass.stopped);
    let note = pass.note.expect("a note");
    assert!(
        note.contains("miss 1") && note.contains("no gap key"),
        "{note}"
    );
}

#[test]
fn memory_is_read_after_the_warm_up_and_after_the_measured_words() {
    assert_eq!(passes(3, 30), [(0, 3), (3, 30)]);
}

#[test]
fn the_cases_are_taken_in_turn_from_where_the_warm_up_stopped() {
    let cases = [case("en"), case("fr"), case("ar")];
    let langs = |skip, count| {
        let taken = turn(&cases, skip, count).map(|c| c.lang.as_str());
        taken.collect::<Vec<_>>()
    };
    assert_eq!(langs(0, 2), ["en", "fr"], "the warm-up");
    assert_eq!(
        langs(2, 4),
        ["ar", "en", "fr", "ar"],
        "the measured words go on"
    );
    assert!(langs(0, 0).is_empty());
}

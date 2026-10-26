//! G8's typing churn (`g8 predict`): words typed through the product's chips in a target-window G8 starts, never
//! Notepad, taking `[g27]`'s cases in turn. Private memory is read after `[g8_predict] warm_words` and again after
//! `words` more.
#![cfg(windows)]

use serde::Serialize;

use crate::apps::Ctx;
use crate::g27type::Typist;
use crate::p7cfg::Case;
use crate::proc::{Memory, Process};
use crate::typebox;
use crate::win::sleep_ms;

/// The churn's memory readings, each after its words and `[g8_predict] settle_ms`: after the warm-up, then after the
/// measured words.
pub(crate) const READINGS: usize = 2;
/// Why a churn typed no more words after a missed one.
const STOPPED: &str = "the missed word could not be ended, so no word followed";
/// Begins the note of a churn whose warm-up stopped, so no word was measured.
const WARM_UP: &str = "the warm-up stopped";

/// What the typing churn did, and private memory around its measured words.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Typed {
    /// Words it measured.
    pub words: usize,
    /// Of those, the words whose chip showed and was clicked.
    pub picked: usize,
    /// Memory after the warm-up words.
    pub before: Memory,
    /// Memory after the measured words.
    pub after: Memory,
    /// Why the first missed measured word failed; none when every word was picked.
    pub note: Option<String>,
}

impl Typed {
    /// True when every measured word was picked, so the churn churned all it counts.
    pub fn whole(&self) -> bool {
        self.picked == self.words
    }
}

/// One word: its case's layout, its prefix, a chip that offers it, and the click.
fn word(t: &Typist, case: &Case) -> Result<(), String> {
    let layout = t.switch(&case.lang)?;
    let last = t.type_in(layout, &case.prefix)?;
    t.offered(case, last)?;
    t.pick().map(drop)
}

/// The cases `count` words take, in turn from the `skip`th on.
fn turn(cases: &[Case], skip: usize, count: usize) -> impl Iterator<Item = &Case> {
    cases.iter().cycle().skip(skip).take(count)
}

/// What one pass typed: the words picked, why the first missed one failed, and whether a failed word end stopped it.
#[derive(Debug, Default)]
struct Pass {
    picked: usize,
    note: Option<String>,
    stopped: bool,
}

/// Types each of `cases` with `word`. A missed word is ended with `end` before the next starts, so its prefix never
/// runs into the next; when that end fails, none follows.
fn tally<'c>(
    cases: impl Iterator<Item = &'c Case>,
    mut word: impl FnMut(&Case) -> Result<(), String>,
    mut end: impl FnMut(&Case) -> Result<(), String>,
) -> Pass {
    let mut pass = Pass::default();
    for case in cases {
        let Err(e) = word(case) else {
            pass.picked += 1;
            continue;
        };
        let first = pass.note.take().unwrap_or(e);
        if let Err(stop) = end(case) {
            pass.note = Some(format!("{first}; {STOPPED}: {stop}"));
            pass.stopped = true;
            return pass;
        }
        pass.note = Some(first);
    }
    pass
}

/// Types `count` words, the cases taken in turn from the `skip`th on, each missed one ended with the space bar's gap.
fn words(t: &Typist, cases: &[Case], skip: usize, count: usize) -> Pass {
    tally(
        turn(cases, skip, count),
        |c| word(t, c),
        |c| t.end_word(&c.prefix),
    )
}

/// The churn's passes before each reading, as words skipped and typed: the `warm` words, then `measured` more.
fn passes(warm: usize, measured: usize) -> [(usize, usize); READINGS] {
    [(0, warm), (warm, measured)]
}

/// Each pass's words, then `[g8_predict] settle_ms` and a memory reading.
fn churn(ctx: &Ctx, t: &Typist, process: &Process) -> Result<Typed, String> {
    let (cases, p) = (&ctx.cfg.g27.cases, &ctx.cfg.g8_predict);
    let read = |(skip, count)| {
        let typed = words(t, cases, skip, count);
        sleep_ms(p.settle_ms);
        process.memory().map(|m| (typed, m))
    };
    let [warm, measured] = passes(p.warm_words, p.words);
    measure(p.words, read(warm)?, || read(measured))
}

/// The churn of `words` measured words from the warm-up pass and its reading, then the measured pass. A warm-up that a
/// failed word end stopped measures nothing and carries its note.
fn measure(
    words: usize,
    (warm, before): (Pass, Memory),
    measured: impl FnOnce() -> Result<(Pass, Memory), String>,
) -> Result<Typed, String> {
    if warm.stopped {
        let note = warm.note.map(|n| format!("{WARM_UP}: {n}"));
        let after = before;
        return Ok(Typed {
            words,
            picked: 0,
            before,
            after,
            note,
        });
    }
    let (pass, after) = measured()?;
    Ok(Typed {
        words,
        picked: pass.picked,
        before,
        after,
        note: pass.note,
    })
}

/// Runs the typing churn on the product G8 started, held as `process`, in a fresh target-window; the window's first
/// layout comes back and the window closes after, or the churn fails saying so.
pub fn run(ctx: &Ctx, process: &Process) -> Result<Typed, String> {
    let b = typebox::in_target(ctx, &[], |t| churn(ctx, t, process));
    match (b.got, b.back, b.left.is_empty()) {
        (Ok(c), true, true) => Ok(c),
        (Ok(_), false, _) => Err("target-window's first layout did not come back".to_string()),
        (Ok(_), true, false) => Err(format!("the typing churn left {:?}", b.left)),
        (Err(e), _, _) => Err(format!("{e}; clean-up: {:?}", b.left)),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    fn memory(mb: f64) -> Memory {
        Memory {
            working_mb: mb,
            commit_mb: mb,
        }
    }

    #[test]
    fn the_churn_is_whole_only_when_every_measured_word_was_picked() {
        let mut c = Typed {
            words: 30,
            picked: 30,
            before: memory(60.0),
            after: memory(61.0),
            note: None,
        };
        assert!(c.whole());
        c.picked = 29;
        assert!(!c.whole());
    }

    fn case(lang: &str) -> Case {
        Case {
            lang: lang.into(),
            prefix: "x".into(),
        }
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
        assert!(!typed.whole());
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
            picked: 30,
            note: None,
            stopped: false,
        };
        let typed = measure(30, (warm, memory(60.0)), || Ok((measured, memory(61.0))));
        let typed = typed.unwrap();
        assert!(typed.whole() && typed.note.is_none());
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
}

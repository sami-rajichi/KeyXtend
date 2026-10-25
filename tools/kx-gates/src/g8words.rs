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

/// What the typing churn did, and private memory around its measured words.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Typed {
    /// Words it measured.
    pub words: usize,
    /// Of those, the words whose chip showed and was clicked.
    pub picked: usize,
    /// Memory after the warm-up words.
    pub before: Memory,
    /// Memory after the measured words.
    pub after: Memory,
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

/// Types `count` words, the cases taken in turn from the `skip`th on: how many were picked.
fn words(t: &Typist, cases: &[Case], skip: usize, count: usize) -> usize {
    turn(cases, skip, count)
        .filter(|case| word(t, case).is_ok())
        .count()
}

/// The warm-up words, a reading, the measured words, and another reading, each after `[g8_predict] settle_ms`.
fn churn(ctx: &Ctx, t: &Typist, process: &Process) -> Result<Typed, String> {
    let (cases, p) = (&ctx.cfg.g27.cases, &ctx.cfg.g8_predict);
    words(t, cases, 0, p.warm_words);
    sleep_ms(p.settle_ms);
    let before = process.memory()?;
    let picked = words(t, cases, p.warm_words, p.words);
    sleep_ms(p.settle_ms);
    Ok(Typed {
        words: p.words,
        picked,
        before,
        after: process.memory()?,
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
        };
        assert!(c.whole());
        c.picked = 29;
        assert!(!c.whole());
    }

    #[test]
    fn the_cases_are_taken_in_turn_from_where_the_warm_up_stopped() {
        let case = |lang: &str| Case {
            lang: lang.into(),
            prefix: "x".into(),
        };
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

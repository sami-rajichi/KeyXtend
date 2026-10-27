//! G8's typing churns (`g8 predict`): words typed through the product's chips in a target-window G8 starts, never
//! Notepad. The judged churn types `[g8_predict] lang`'s case alone; the mixed churn then takes every `[g27]` case in
//! turn, as a reading only.
#![cfg(windows)]

use serde::Serialize;

use crate::apps::Ctx;
use crate::g27type::Typist;
use crate::p7cfg::{Case, G8Predict, G27};
use crate::proc::{Memory, Process};
use crate::typebox;
use crate::win::sleep_ms;

/// Each churn's memory readings, each after its words and `[g8_predict] settle_ms`: after the warm-up, then after the
/// measured words.
pub(crate) const READINGS: usize = 2;
/// Why a churn typed no more words after a missed one.
const STOPPED: &str = "the missed word could not be ended, so no word followed";
/// Begins the note of a churn whose warm-up stopped, so no word was measured.
const WARM_UP: &str = "the warm-up stopped";
/// Why the mixed churn did not run.
const NO_MIXED: &str = "a failed word end stopped the judged churn, so the mixed churn did not run";

/// What a typing churn did, and private memory around its measured words.
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
    /// A missed word could not be ended, so no word followed it.
    pub stopped: bool,
}

impl Typed {
    /// True when every measured word was picked, so the churn churned all it counts.
    pub fn whole(&self) -> bool {
        self.picked == self.words
    }
}

/// Both churns: the judged one, and the mixed one, which never fails the run.
#[derive(Debug, Clone)]
pub struct Churns {
    /// The churn of `[g8_predict] lang`'s case, judged against `[g8] churn_growth_mb`.
    pub judged: Typed,
    /// The churn of every case in turn, or why it did not run; a reading only.
    pub mixed: Result<Typed, String>,
}

/// A churn's cases, taken in turn, and its passes before each reading, as words skipped and typed.
#[derive(Debug)]
pub(crate) struct Plan<'c> {
    cases: &'c [Case],
    passes: [(usize, usize); READINGS],
}

impl<'c> Plan<'c> {
    /// The judged churn: `p.lang`'s case alone, `warm_words` of it, then `words`; an error when no case has it.
    pub fn judged(g: &'c G27, p: &G8Predict) -> Result<Self, String> {
        Ok(Self {
            cases: std::slice::from_ref(p.case(g)?),
            passes: passes(p.warm_words, p.words),
        })
    }

    /// The mixed churn: every case in turn, a warm-up round of one word each, then `mixed_words`.
    pub fn mixed(g: &'c G27, p: &G8Predict) -> Self {
        Self {
            cases: &g.cases,
            passes: passes(g.cases.len(), p.mixed_words),
        }
    }

    /// Words it types in all.
    pub fn words(&self) -> usize {
        self.passes
            .iter()
            .fold(0, |n, &(_, count)| n.saturating_add(count))
    }

    /// Its longest prefix, in characters.
    pub fn keys(&self) -> usize {
        let keys = self.cases.iter().map(|k| k.prefix.chars().count());
        keys.max().unwrap_or_default()
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

/// A churn's passes before each reading, as words skipped and typed: the `warm` words, then `measured` more.
fn passes(warm: usize, measured: usize) -> [(usize, usize); READINGS] {
    [(0, warm), (warm, measured)]
}

/// Each of `plan`'s passes, then `[g8_predict] settle_ms` and a memory reading.
fn churn(ctx: &Ctx, t: &Typist, process: &Process, plan: &Plan) -> Result<Typed, String> {
    let settle_ms = ctx.cfg.g8_predict.settle_ms;
    let read = |(skip, count)| {
        let typed = words(t, plan.cases, skip, count);
        sleep_ms(settle_ms);
        process.memory().map(|m| (typed, m))
    };
    let [warm, measured] = plan.passes;
    measure(measured.1, read(warm)?, || read(measured))
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
            stopped: true,
        });
    }
    let (pass, after) = measured()?;
    Ok(Typed {
        words,
        picked: pass.picked,
        before,
        after,
        note: pass.note,
        stopped: pass.stopped,
    })
}

/// Both churns from the judged one; the mixed one runs unless a failed word end stopped the judged one.
fn then_mixed(judged: Typed, mixed: impl FnOnce() -> Result<Typed, String>) -> Churns {
    let mixed = if judged.stopped {
        Err(NO_MIXED.to_string())
    } else {
        mixed()
    };
    Churns { judged, mixed }
}

/// The judged churn, then the mixed one in the same box.
fn both(ctx: &Ctx, t: &Typist, process: &Process) -> Result<Churns, String> {
    let (g, p) = (&ctx.cfg.g27, &ctx.cfg.g8_predict);
    let judged = churn(ctx, t, process, &Plan::judged(g, p)?)?;
    Ok(then_mixed(judged, || {
        churn(ctx, t, process, &Plan::mixed(g, p))
    }))
}

/// Runs both typing churns on the product G8 started, held as `process`, in a fresh target-window; the window's first
/// layout comes back and the window closes after, or the run fails saying so.
pub fn run(ctx: &Ctx, process: &Process) -> Result<Churns, String> {
    let b = typebox::in_target(ctx, &[], |t| both(ctx, t, process));
    match (b.got, b.back, b.left.is_empty()) {
        (Ok(c), true, true) => Ok(c),
        (Ok(_), false, _) => Err("target-window's first layout did not come back".to_string()),
        (Ok(_), true, false) => Err(format!("the typing churns left {:?}", b.left)),
        (Err(e), _, _) => Err(format!("{e}; clean-up: {:?}", b.left)),
    }
}

#[cfg(test)]
mod tests;

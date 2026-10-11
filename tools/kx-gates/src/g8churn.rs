//! G8's churn (spec §10, no growth): rounds of key tips, the language list, the bubble and a full theme cycle
//! (`g8pops`), with private memory read before them and after each one. It runs after `PresentMon`'s capture, so no
//! frame counts.
#![cfg(windows)]

use serde::Serialize;

use crate::apps::Ctx;
use crate::board;
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::g8pops::{self, Seen};
use crate::keymap::Reader;
use crate::mouse;
use crate::proc::{Memory, Process};
use crate::win::sleep_ms;

/// What the churn did, and private memory around its measured rounds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Churn {
    /// Measured rounds.
    pub rounds: usize,
    /// Memory before the first measured round.
    pub before: Memory,
    /// Memory after the last one: the last of `per_round`.
    pub after: Memory,
    /// Memory after each measured round, in order, so a steady climb shows apart from one jump.
    pub per_round: Vec<Memory>,
    /// Key hovers in the measured rounds.
    pub hovers: usize,
    /// Hovers that showed a tip.
    pub tips: usize,
    /// Rounds that showed the language list; none when the list step was skipped.
    pub lists: Option<usize>,
    /// Rounds whose minimise hid the keyboard and showed the bubble.
    pub bubbles: usize,
    /// Rounds whose bubble brought the keyboard back.
    pub returns: usize,
    /// Why the list step was skipped: not the List design, or one layout installed.
    pub note: Option<&'static str>,
}

impl Churn {
    /// True when every hover showed a tip and every round showed the bubble, the keyboard again, and the list in the
    /// List design.
    pub fn whole(&self) -> bool {
        let every = |n: usize| n == self.rounds;
        let pops = self.tips == self.hovers && every(self.bubbles) && every(self.returns);
        pops && self.lists.is_none_or(every)
    }
}

/// The growth of the working set and of the commit over the measured rounds, each against `[g8] churn_growth_mb`.
pub fn parts(g: &G8, c: &Churn) -> [Part; 2] {
    let (b, a, most) = (c.before, c.after, g.churn_growth_mb);
    [
        Part::at_most("churn_working_mb", Some(a.working_mb - b.working_mb), most),
        Part::at_most("churn_commit_mb", Some(a.commit_mb - b.commit_mb), most),
    ]
}

/// Refuses churn settings that could pass with nothing churned.
pub fn check(g: &G8) -> Result<(), String> {
    let most = g.churn_growth_mb;
    if g.churn_rounds == 0 || !most.is_finite() || most <= 0.0 {
        return Err(
            "g8.churn_rounds and g8.churn_growth_mb must be above 0 and finite".to_string(),
        );
    }
    if g.churn_tips.is_empty() || g.churn_theme.is_empty() {
        return Err("g8.churn_tips and g8.churn_theme must name keys and a button".to_string());
    }
    let ids = board::ids()?;
    match g.churn_tips.iter().find(|t| !ids.contains(t)) {
        Some(t) => Err(format!(
            "g8.churn_tips: {t} is no key of the product's board"
        )),
        None => Ok(()),
    }
}

/// Memory from `read` before `n` rounds of `round` and after each one, in order; the first error ends it.
fn readings(
    n: usize,
    mut round: impl FnMut() -> Result<(), String>,
    mut read: impl FnMut() -> Result<Memory, String>,
) -> Result<(Memory, Vec<Memory>), String> {
    let before = read()?;
    let mut each = Vec::with_capacity(n);
    for _ in 0..n {
        round()?;
        each.push(read()?);
    }
    Ok((before, each))
}

/// Runs `[g8] churn_warm_rounds` unmeasured rounds, then `churn_rounds` measured ones, reading `p`'s memory before
/// them and after each. The warm-up keeps each pop-up's first show, a one-time cost, out of the growth.
pub fn run(ctx: &Ctx, p: &Process) -> Result<Churn, String> {
    let g = &ctx.cfg.g8;
    let (plan, reader) = (g8pops::plan(ctx)?, Reader::new()?);
    mouse::move_to(plan.away)?;
    sleep_ms(g.churn_settle_ms);
    for _ in 0..g.churn_warm_rounds {
        g8pops::round(g, &reader, &plan, &mut Seen::default())?;
    }
    let mut seen = Seen::default();
    let round = || g8pops::round(g, &reader, &plan, &mut seen);
    let (before, per_round) = readings(g.churn_rounds, round, || p.memory())?;
    let after = per_round.last().copied().unwrap_or(before);
    let lists = plan.lang.as_ref().map(|_| seen.lists);
    Ok(Churn {
        rounds: g.churn_rounds,
        before,
        after,
        per_round,
        hovers: g.churn_rounds * plan.tips.len(),
        tips: seen.tips,
        lists,
        bubbles: seen.bubbles,
        returns: seen.returns,
        note: plan.skip,
    })
}

/// A churn of `g`'s rounds from `before` to `after`, each round read as `after`, in which every pop-up showed, for
/// tests.
#[cfg(test)]
pub fn whole_over(g: &G8, before: Memory, after: Memory) -> Churn {
    let hovers = g.churn_rounds * g.churn_tips.len();
    Churn {
        rounds: g.churn_rounds,
        before,
        after,
        per_round: vec![after; g.churn_rounds],
        hovers,
        tips: hovers,
        lists: Some(g.churn_rounds),
        bubbles: g.churn_rounds,
        returns: g.churn_rounds,
        note: None,
    }
}

#[cfg(test)]
mod tests;

//! G8's churn (spec §10, no growth): rounds of key tips, the language list, the bubble and a full theme cycle
//! (`g8pops`), with private memory read before them and after each one and each of its steps; growth runs from the
//! first rounds' median to the last rounds'. It runs after `PresentMon`'s capture, so no frame counts.
#![cfg(windows)]

use serde::Serialize;

use crate::apps::Ctx;
use crate::board;
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::g8pops::{self, Plan, Seen};
use crate::g8steps::{self, Step, StepGrowth, slopes};
use crate::g8tips::KeyTips;
#[cfg(test)]
use crate::g8tips::{Hover, Keys};
use crate::keymap::Reader;
use crate::mouse;
use crate::proc::{Memory, Process};
use crate::stats;
use crate::win::sleep_ms;

/// Decimals of each MB reading in a round's error: a tenth of an MB, finer than any growth budget.
const MB_DECIMALS: usize = 1;

/// What the churn did, and private memory around its measured rounds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Churn {
    /// Measured rounds.
    pub rounds: usize,
    /// Memory before the first measured round.
    pub before: Memory,
    /// Unmeasured rounds run first, from `[g8] churn_warm_rounds`.
    pub warm_rounds: usize,
    /// Memory after each measured round, in order, so a steady climb shows apart from one jump.
    pub per_round: Vec<Memory>,
    /// The median of the first `[g8] churn_median_rounds` readings; `before` when there are none.
    pub first_median: Memory,
    /// The median of the last ones, which the growth compares with `first_median`.
    pub last_median: Memory,
    /// The least-squares slope of `per_round`, in MB per round; evidence only, as the growth decides.
    pub round_slope: Option<Memory>,
    /// What each step of the measured rounds changed, and its slope; the steps' slopes add up to about `round_slope`.
    pub step_growth: Vec<StepGrowth>,
    /// Key hovers in the measured rounds.
    pub hovers: usize,
    /// Hovers that showed a tip.
    pub tips: usize,
    /// Each tip key's hits, tips that hid again and show times in the measured rounds, in `[g8] churn_tips` order.
    pub tip_keys: Vec<KeyTips>,
    /// The same in the warm-up rounds, whose first round alone follows no theme cycle.
    pub warm_tip_keys: Vec<KeyTips>,
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
    /// `g`'s churn read as `before` and `per_round`, with no pop-up counted yet.
    fn measured(g: &G8, before: Memory, per_round: Vec<Memory>) -> Self {
        let n = g.churn_median_rounds.min(per_round.len());
        let (first, last) = (&per_round[..n], &per_round[per_round.len() - n..]);
        Self {
            rounds: g.churn_rounds,
            before,
            warm_rounds: g.churn_warm_rounds,
            first_median: median(first).unwrap_or(before),
            last_median: median(last).unwrap_or(before),
            round_slope: slopes(&per_round),
            step_growth: Vec::new(),
            per_round,
            hovers: 0,
            tips: 0,
            tip_keys: Vec::new(),
            warm_tip_keys: Vec::new(),
            lists: None,
            bubbles: 0,
            returns: 0,
            note: None,
        }
    }

    /// True when every hover showed a tip and every round showed the bubble, the keyboard again, and the list in the
    /// List design.
    pub fn whole(&self) -> bool {
        let every = |n: usize| n == self.rounds;
        let pops = self.tips == self.hovers && every(self.bubbles) && every(self.returns);
        pops && self.lists.is_none_or(every)
    }
}

/// The median working set and the median commit of `readings`, each on its own; none when empty.
fn median(readings: &[Memory]) -> Option<Memory> {
    let of = |mb: fn(&Memory) -> f64| {
        let mut all: Vec<f64> = readings.iter().map(mb).collect();
        all.sort_by(f64::total_cmp);
        stats::median(&all)
    };
    Some(Memory {
        working_mb: of(|m| m.working_mb)?,
        commit_mb: of(|m| m.commit_mb)?,
    })
}

/// The growth of the working set and of the commit, from the first rounds' medians to the last rounds', each against
/// `[g8] churn_growth_mb`.
pub fn parts(g: &G8, c: &Churn) -> [Part; 2] {
    let (b, a, most) = (c.first_median, c.last_median, g.churn_growth_mb);
    [
        Part::at_most("churn_working_mb", Some(a.working_mb - b.working_mb), most),
        Part::at_most("churn_commit_mb", Some(a.commit_mb - b.commit_mb), most),
    ]
}

/// Refuses churn settings that could pass with nothing churned, or whose two medians would share rounds.
pub fn check(g: &G8) -> Result<(), String> {
    let most = g.churn_growth_mb;
    if g.churn_rounds == 0 || !most.is_finite() || most <= 0.0 {
        return Err(
            "g8.churn_rounds and g8.churn_growth_mb must be above 0 and finite".to_string(),
        );
    }
    if g.churn_median_rounds == 0 || g.churn_median_rounds > g.churn_rounds / 2 {
        return Err(
            "g8.churn_median_rounds must be above 0 and at most half of g8.churn_rounds"
                .to_string(),
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

/// Round error `e` with the readings taken so far, `before` and then `each`, in MB, so a broken churn still shows them.
fn so_far(e: &str, before: Memory, each: &[Memory]) -> String {
    let mb = |m: &Memory| {
        let (w, c) = (m.working_mb, m.commit_mb);
        format!("{w:.MB_DECIMALS$}/{c:.MB_DECIMALS$}")
    };
    let taken = std::iter::once(&before)
        .chain(each)
        .map(mb)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{e}; working/commit MB before the rounds, then after each: {taken}")
}

/// Memory from `read` before `n` rounds of `round` and after each one, in order; the first error ends it, and a
/// round's error lists the readings taken so far.
fn readings(
    n: usize,
    mut round: impl FnMut() -> Result<(), String>,
    mut read: impl FnMut() -> Result<Memory, String>,
) -> Result<(Memory, Vec<Memory>), String> {
    let before = read()?;
    let mut each = Vec::with_capacity(n);
    for _ in 0..n {
        round().map_err(|e| so_far(&e, before, &each))?;
        each.push(read()?);
    }
    Ok((before, each))
}

/// One measured round with `reader` and `plan`, reading `p`'s memory after each step into a new list of `steps`.
fn stepped(
    g: &G8,
    (reader, plan): (&Reader, &Plan),
    seen: &mut Seen,
    p: &Process,
    steps: &mut Vec<Vec<(Step, Memory)>>,
) -> Result<(), String> {
    let mut now = Vec::new();
    let done = g8pops::round(g, reader, plan, seen, &mut |s| {
        p.memory().map(|m| now.push((s, m)))
    });
    steps.push(now);
    done
}

/// Runs `[g8] churn_warm_rounds` unmeasured rounds, then `churn_rounds` measured ones, reading `p`'s memory before
/// them and after each one and each step. The warm-up keeps each pop-up's first show, a one-time cost, out of the growth.
pub fn run(ctx: &Ctx, p: &Process) -> Result<Churn, String> {
    let g = &ctx.cfg.g8;
    let (plan, reader) = (g8pops::plan(ctx)?, Reader::new()?);
    mouse::move_to(plan.away)?;
    sleep_ms(g.churn_settle_ms);
    let mut warm = Seen::default();
    for _ in 0..g.churn_warm_rounds {
        g8pops::round(g, &reader, &plan, &mut warm, &mut |_| Ok(()))?;
    }
    let (mut seen, mut steps) = (Seen::default(), Vec::new());
    let round = || stepped(g, (&reader, &plan), &mut seen, p, &mut steps);
    let (before, per_round) = readings(g.churn_rounds, round, || p.memory())?;
    Ok(Churn {
        step_growth: g8steps::growth(before, &per_round, &steps),
        hovers: g.churn_rounds * plan.tips.len(),
        tips: seen.tips,
        tip_keys: seen.keys.report(&g.churn_tips),
        warm_tip_keys: warm.keys.report(&g.churn_tips),
        lists: plan.lang.as_ref().map(|_| seen.lists),
        bubbles: seen.bubbles,
        returns: seen.returns,
        note: plan.skip,
        ..Churn::measured(g, before, per_round)
    })
}

/// A churn of `g`'s rounds read as `before` and `per_round`, in which every pop-up showed, for tests.
#[cfg(test)]
pub fn whole_of(g: &G8, before: Memory, per_round: Vec<Memory>) -> Churn {
    let hovers = g.churn_rounds * g.churn_tips.len();
    let mut keys = Keys::default();
    let hit = Hover {
        hit: true,
        shown_ms: None,
    };
    for i in (0..g.churn_rounds).flat_map(|_| 0..g.churn_tips.len()) {
        keys.add(i, hit);
    }
    Churn {
        hovers,
        tips: hovers,
        tip_keys: keys.report(&g.churn_tips),
        lists: Some(g.churn_rounds),
        bubbles: g.churn_rounds,
        returns: g.churn_rounds,
        ..Churn::measured(g, before, per_round)
    }
}

/// A whole churn of `g`'s rounds whose first half reads `before` and second half `after`, so its medians are those two,
/// for tests.
#[cfg(test)]
pub fn whole_over(g: &G8, before: Memory, after: Memory) -> Churn {
    let half = g.churn_rounds / 2;
    let each = (0..g.churn_rounds).map(|r| if r < half { before } else { after });
    whole_of(g, before, each.collect())
}

#[cfg(test)]
mod tests;

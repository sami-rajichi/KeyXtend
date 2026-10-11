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
mod tests {
    use std::cell::Cell;

    use kx_theme::Themes;
    use serde_json::json;

    use super::*;
    use crate::budget;

    fn shipped() -> G8 {
        crate::config::load().expect("kx-gates.toml loads").g8
    }

    fn mem(working_mb: f64, commit_mb: f64) -> Memory {
        Memory {
            working_mb,
            commit_mb,
        }
    }

    fn churn(before: Memory, after: Memory) -> Churn {
        whole_over(&shipped(), before, after)
    }

    fn failed(c: &Churn) -> Vec<&'static str> {
        let parts = parts(&shipped(), c);
        parts.iter().filter(|p| !p.pass).map(|p| p.name).collect()
    }

    /// `n` readings, each `step` MB of working set above the one before, from `from`.
    fn climb(from: Memory, n: usize, step: f64) -> Vec<Memory> {
        let n = u32::try_from(n).expect("few rounds");
        let at = |r| mem(from.working_mb + step * f64::from(r), from.commit_mb);
        (1..=n).map(at).collect()
    }

    /// A churn of the shipped rounds read as `per_round`, whose last reading is its `after`.
    fn tracked(before: Memory, per_round: Vec<Memory>) -> Churn {
        let after = *per_round.last().expect("a round");
        Churn {
            per_round,
            ..churn(before, after)
        }
    }

    /// A round that only counts itself in `done`.
    fn counted(done: &Cell<u32>) -> impl FnMut() -> Result<(), String> + '_ {
        || {
            done.set(done.get() + 1);
            Ok(())
        }
    }

    #[test]
    fn a_churn_reads_memory_before_its_rounds_and_after_each_one_and_its_result_lists_them() {
        let n = shipped().churn_rounds;
        let done = Cell::new(0);
        // Each reading is the count of rounds run so far, so the list shows each followed its round.
        let read = || Ok(mem(f64::from(done.get()), 0.0));
        let (before, per_round) = readings(n, counted(&done), read).expect("every reading worked");
        assert_eq!(before, mem(0.0, 0.0));
        assert_eq!(per_round, climb(before, n, 1.0));
        let shown = serde_json::to_value(tracked(before, per_round)).expect("serialises");
        let listed = shown["per_round"].as_array().expect("a list");
        assert_eq!(listed.len(), n);
        assert_eq!(listed[0], json!({ "working_mb": 1.0, "commit_mb": 0.0 }));
        assert_eq!(listed.last(), Some(&shown["after"]), "the last reading");
    }

    #[test]
    fn a_reading_that_fails_ends_the_churn_with_its_own_error() {
        for at in [0, 2] {
            let done = Cell::new(0);
            let read = || match done.get() {
                r if r == at => Err("memory: access denied".to_string()),
                r => Ok(mem(f64::from(r), 0.0)),
            };
            let e = readings(5, counted(&done), read).expect_err("a reading failed");
            assert_eq!(e, "memory: access denied");
            assert_eq!(done.get(), at, "no round runs after a failed reading");
        }
    }

    #[test]
    fn the_verdict_reads_only_before_and_after_so_a_settled_jump_passes_and_a_steady_climb_fails() {
        let g = shipped();
        let (before, over) = (mem(98.0, 138.0), g.churn_growth_mb + 1.0);
        let mut jump = vec![before; g.churn_rounds];
        jump[g.churn_rounds / 2] = mem(before.working_mb + over, before.commit_mb + over);
        assert!(failed(&tracked(before, jump)).is_empty(), "it ended flat");
        let n = f64::from(u32::try_from(g.churn_rounds).expect("few rounds"));
        let steady = climb(before, g.churn_rounds, over / n);
        assert_eq!(failed(&tracked(before, steady)), ["churn_working_mb"]);
    }

    #[test]
    fn flat_or_shrinking_memory_passes_and_growth_past_the_budget_fails_the_measure_that_grew() {
        let flat = churn(mem(98.0, 138.0), mem(98.4, 138.2));
        assert!(budget::all_pass(&parts(&shipped(), &flat)));
        assert!(failed(&churn(mem(98.0, 138.0), mem(90.0, 130.0))).is_empty());
        let g = shipped();
        let grew = g.churn_growth_mb + 0.5;
        let commit = churn(mem(98.0, 138.0), mem(98.0, 138.0 + grew));
        assert_eq!(failed(&commit), ["churn_commit_mb"]);
        let working = churn(mem(98.0, 138.0), mem(98.0 + grew, 138.0));
        assert_eq!(failed(&working), ["churn_working_mb"]);
        let at = churn(mem(98.0, 138.0), mem(98.0 + g.churn_growth_mb, 138.0));
        assert!(failed(&at).is_empty(), "the budget itself is allowed");
    }

    #[test]
    fn a_churn_is_whole_when_every_pop_up_showed_and_a_design_without_a_list_still_counts() {
        let full = churn(mem(98.0, 138.0), mem(98.0, 138.0));
        assert!(full.whole());
        let no_list = Churn {
            lists: None,
            ..full.clone()
        };
        assert!(no_list.whole(), "the default Carousel design opens no list");
        let short = full.rounds - 1;
        let missed: [fn(&mut Churn, usize); 4] = [
            |c, n| c.tips = n,
            |c, n| c.lists = Some(n),
            |c, n| c.bubbles = n,
            |c, n| c.returns = n,
        ];
        for (i, miss) in missed.iter().enumerate() {
            let mut c = full.clone();
            miss(&mut c, short);
            assert!(
                !c.whole(),
                "case {i}: a pop-up or the keyboard's return was missed"
            );
        }
    }

    #[test]
    fn churn_settings_that_churn_nothing_are_refused() {
        assert_eq!(check(&shipped()), Ok(()));
        let bad: [fn(&mut G8); 6] = [
            |g| g.churn_rounds = 0,
            |g| g.churn_growth_mb = 0.0,
            |g| g.churn_growth_mb = f64::INFINITY,
            |g| g.churn_tips.clear(),
            |g| g.churn_tips.push("c-0xFFFF".into()),
            |g| g.churn_theme.clear(),
        ];
        for (i, f) in bad.iter().enumerate() {
            let mut g = shipped();
            f(&mut g);
            assert!(check(&g).is_err(), "case {i}");
        }
    }

    #[test]
    fn the_shipped_churn_warms_up_for_five_rounds_as_memory_settles_over_them() {
        assert_eq!(shipped().churn_warm_rounds, 5, "the owner's choice, P3.md");
    }

    #[test]
    fn each_hovered_key_has_an_icon_so_its_tip_shows() {
        // A key's tip shows its name when it has an icon (Key.qml); every theme shares icons.toml.
        let icons = Themes::shipped().expect("the tokens read").icons.keys;
        for id in &shipped().churn_tips {
            assert!(icons.contains_key(id), "{id}");
        }
    }
}

//! G8's churn (spec §10, no growth): rounds of key tips, the language list, the bubble and a full theme cycle
//! (`g8pops`), with private memory read before and after. It runs after `PresentMon`'s capture, so no frame counts.
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
    /// Memory after the last one.
    pub after: Memory,
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

/// Runs `[g8] churn_warm_rounds` unmeasured rounds, then `churn_rounds` measured ones, reading `p`'s memory around
/// them. The warm-up keeps each pop-up's first show, a one-time cost, out of the growth.
pub fn run(ctx: &Ctx, p: &Process) -> Result<Churn, String> {
    let g = &ctx.cfg.g8;
    let (plan, reader) = (g8pops::plan(ctx)?, Reader::new()?);
    mouse::move_to(plan.away)?;
    sleep_ms(g.churn_settle_ms);
    for _ in 0..g.churn_warm_rounds {
        g8pops::round(g, &reader, &plan, &mut Seen::default())?;
    }
    let before = p.memory()?;
    let mut seen = Seen::default();
    for _ in 0..g.churn_rounds {
        g8pops::round(g, &reader, &plan, &mut seen)?;
    }
    let after = p.memory()?;
    let lists = plan.lang.as_ref().map(|_| seen.lists);
    Ok(Churn {
        rounds: g.churn_rounds,
        before,
        after,
        hovers: g.churn_rounds * plan.tips.len(),
        tips: seen.tips,
        lists,
        bubbles: seen.bubbles,
        returns: seen.returns,
        note: plan.skip,
    })
}

/// A churn of `g`'s rounds from `before` to `after` in which every pop-up showed, for tests.
#[cfg(test)]
pub fn whole_over(g: &G8, before: Memory, after: Memory) -> Churn {
    let hovers = g.churn_rounds * g.churn_tips.len();
    Churn {
        rounds: g.churn_rounds,
        before,
        after,
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
    use kx_theme::Themes;

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
    fn each_hovered_key_has_an_icon_so_its_tip_shows() {
        // A key's tip shows its name when it has an icon (Key.qml); every theme shares icons.toml.
        let icons = Themes::shipped().expect("the tokens read").icons.keys;
        for id in &shipped().churn_tips {
            assert!(icons.contains_key(id), "{id}");
        }
    }
}

//! G8's churn (spec §10, no growth): rounds of key tips, the language list and a full theme cycle, with private
//! memory read before and after. It moves the real pointer but never clicks, and presses buttons by UI Automation.
#![cfg(windows)]

use kx_mod_keyboard::{LangKey, Role};
use kx_theme::Themes;
use serde::Serialize;
use windows::Win32::Foundation::POINT;

use crate::apps::Ctx;
use crate::budget::Part;
use crate::g8cfg::G8;
use crate::keymap::Reader;
use crate::proc::{Memory, Process};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, g8ui, mouse, uilook};

/// Why a churn is not whole when the language key opens no list.
const NO_LIST: &str = "no list showed, as the language key is not the List design; set [keyboard] \
     lang_key = \"list\" for a full pass";
/// What a churn that broke off mid-cycle leaves, for its error.
const THEME_LEFT: &str = "the theme may differ now: press the theme button to bring yours back";

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
    /// Rounds that showed the language list; none when the language key opens no list.
    pub lists: Option<usize>,
    /// Why no list showed, when none could.
    pub note: Option<&'static str>,
}

impl Churn {
    /// True when every hover showed a tip and every round showed the list.
    pub fn whole(&self) -> bool {
        self.tips == self.hovers && self.lists == Some(self.rounds)
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

/// What each round does, worked out once.
struct Plan {
    /// Screen centres of the keys whose tips show.
    tips: Vec<POINT>,
    /// Where the pointer rests between hovers: just off the keyboard.
    away: POINT,
    /// The pointer's rest on each key: the running copy's tip time and `[g8] churn_tip_ms`.
    hover_ms: u64,
    /// The language key's id, when its design opens the list.
    lang: Option<String>,
    /// Theme button presses that bring the theme back: one per shipped theme.
    themes: usize,
}

/// The plan for the running copy: its keys' places, its tip time and its language key's design.
fn plan(g: &G8) -> Result<Plan, String> {
    let kb = Keyboard::find()?;
    let tips: Vec<POINT> = g
        .churn_tips
        .iter()
        .map(|id| kb.at(id))
        .collect::<Result<_, _>>()?;
    let first = *tips.first().ok_or("g8.churn_tips names no key")?;
    let away = g8ui::away_from(win::rect(kb.hwnd)?, first, win::desktop().1);
    let lists = product::keyboard()?.lang_key == LangKey::List;
    let lang = lists.then(|| board::id_of(Role::Lang)).transpose()?;
    let themes = Themes::shipped().map_err(|e| format!("the shipped themes: {e}"))?;
    Ok(Plan {
        tips,
        away,
        hover_ms: uilook::running()?.tip_ms + g.churn_tip_ms,
        lang,
        themes: themes.list.len(),
    })
}

/// What showed in the measured rounds.
#[derive(Default)]
struct Seen {
    /// Hovers that showed a new window.
    tips: usize,
    /// List opens that showed a new window.
    lists: usize,
}

/// Rests the pointer on each tip key, counting each hover that showed a new window, then moves it away.
fn hover(plan: &Plan, base: usize, seen: &mut Seen) -> Result<(), String> {
    for &at in &plan.tips {
        mouse::move_to(at)?;
        sleep_ms(plan.hover_ms);
        seen.tips += usize::from(product::shown().len() > base);
    }
    mouse::move_to(plan.away)
}

/// Opens and closes the language list with its key, counting an open that showed a new window.
fn list(g: &G8, reader: &Reader, lang: &str, base: usize, seen: &mut Seen) -> Result<(), String> {
    product::invoke(reader, lang)?;
    sleep_ms(g.churn_settle_ms);
    seen.lists += usize::from(product::shown().len() > base);
    product::invoke(reader, lang)?;
    sleep_ms(g.churn_settle_ms);
    Ok(())
}

/// Presses the theme button `n` times, once per theme, so the theme ends where it started.
fn themes(g: &G8, reader: &Reader, n: usize) -> Result<(), String> {
    for i in 0..n {
        let left = |e| format!("{e}; pressed {i} of {n} times, so {THEME_LEFT}");
        product::invoke(reader, &g.churn_theme).map_err(left)?;
        sleep_ms(g.churn_settle_ms);
    }
    Ok(())
}

/// One round: the tips, the list when the language key opens one, and a full theme cycle.
fn round(g: &G8, reader: &Reader, plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    // The pointer rests away from the keys, so no tip shows yet.
    let base = product::shown().len();
    hover(plan, base, seen)?;
    sleep_ms(g.churn_settle_ms);
    if let Some(lang) = &plan.lang {
        list(g, reader, lang, base, seen)?;
    }
    themes(g, reader, plan.themes)
}

/// Runs `[g8] churn_warm_rounds` unmeasured rounds, then `churn_rounds` measured ones, reading `p`'s memory around
/// them. The warm-up keeps each pop-up's first show, a one-time cost, out of the growth.
pub fn run(ctx: &Ctx, p: &Process) -> Result<Churn, String> {
    let g = &ctx.cfg.g8;
    let (plan, reader) = (plan(g)?, Reader::new()?);
    mouse::move_to(plan.away)?;
    sleep_ms(g.churn_settle_ms);
    for _ in 0..g.churn_warm_rounds {
        round(g, &reader, &plan, &mut Seen::default())?;
    }
    let before = p.memory()?;
    let mut seen = Seen::default();
    for _ in 0..g.churn_rounds {
        round(g, &reader, &plan, &mut seen)?;
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
        note: lists.is_none().then_some(NO_LIST),
    })
}

/// A churn of `g`'s rounds from `before` to `after` in which every tip and list showed, for tests.
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
        note: None,
    }
}

#[cfg(test)]
mod tests {
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
    fn a_churn_is_whole_only_when_every_tip_and_every_list_showed() {
        let full = churn(mem(98.0, 138.0), mem(98.0, 138.0));
        assert!(full.whole());
        let missed = Churn {
            tips: full.hovers - 1,
            ..full.clone()
        };
        assert!(!missed.whole(), "a hover showed no tip");
        let no_list = Churn {
            lists: None,
            ..full.clone()
        };
        assert!(!no_list.whole(), "the language key opens no list");
        let short = Churn {
            lists: Some(full.rounds - 1),
            ..full
        };
        assert!(!short.whole(), "a round showed no list");
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

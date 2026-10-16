//! One round of G8's churn: key tips, the language list in the List design, the bubble, a full theme cycle and a
//! scroll (`g8scroll`). It moves the real pointer but never clicks, presses buttons by UI Automation and counts what
//! showed.
#![cfg(windows)]

use std::collections::HashSet;
use std::time::Instant;

use kx_mod_keyboard::langkey::lone;
use kx_mod_keyboard::{LangKey, Role};
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use kx_theme::Themes;
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::Ctx;
use crate::g8cfg::G8;
use crate::g8scroll::Scroller;
use crate::g8steps::Step;
use crate::g8tips::{self, Hover, Keys};
use crate::keymap::Reader;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, g8ui, mouse, uilook};

/// What a round that broke off mid-cycle leaves, for its error.
const THEME_LEFT: &str = "the theme may differ now: press the theme button to bring yours back";
/// Why the list step is skipped: only the List design has a list.
const NOT_LIST: &str = "the language key is not the List design, so the list step was skipped";
/// Why the list step is skipped: with one layout the key opens nothing.
const ONE_LAYOUT: &str =
    "only one layout is installed, so the List design opens no list and the step was skipped";
/// Why the bubble step stops: minimise was refused, or the bubble had not shown yet.
const NO_BUBBLE: &str =
    "minimise showed no bubble: the screen may have no spot for it, or raise g8.churn_settle_ms";

/// What each round does, worked out once.
pub struct Plan {
    /// Screen centres of the keys whose tips show.
    pub tips: Vec<POINT>,
    /// Where the pointer rests between hovers: just off the keyboard.
    pub away: POINT,
    /// The pointer's rest on each key: the running copy's tip time and `[g8] churn_tip_ms`.
    hover_ms: u64,
    /// How often each rest looks for a new window: `[timing] poll_ms`.
    poll_ms: u64,
    /// The product's process, whose windows each look lists.
    pid: u32,
    /// The language key's id, when it opens the list.
    pub lang: Option<String>,
    /// Why the list step is skipped, when it is.
    pub skip: Option<&'static str>,
    /// Theme button presses that bring the theme back: one per shipped theme.
    themes: usize,
    /// The owner's theme, which each cycle must end on.
    theme: String,
    /// The keyboard's window, which hides while the bubble shows.
    keyboard: HWND,
    /// The top bar's minimise button, from `[g9] put_away`.
    hide: String,
    /// The bubble, which brings the keyboard back, from `[g9] bring_back`.
    back: String,
}

/// Why the list step is skipped for a language key of `design` with `layouts` installed; nothing when it runs.
fn skip_list(design: LangKey, layouts: usize) -> Option<&'static str> {
    if design != LangKey::List {
        Some(NOT_LIST)
    } else if lone(layouts) {
        Some(ONE_LAYOUT)
    } else {
        None
    }
}

/// The plan for the running copy: its keys' places, its tip time and its language key's design.
pub fn plan(ctx: &Ctx) -> Result<Plan, String> {
    let (g, g9) = (&ctx.cfg.g8, &ctx.cfg.g9);
    let kb = Keyboard::find()?;
    let tips: Vec<POINT> = g
        .churn_tips
        .iter()
        .map(|id| kb.at(id))
        .collect::<Result<_, _>>()?;
    let first = *tips.first().ok_or("g8.churn_tips names no key")?;
    let away = g8ui::away_from(win::rect(kb.hwnd)?, first, win::desktop().1);
    let skip = skip_list(product::keyboard()?.lang_key, WinLayouts.installed().len());
    let lang = skip
        .is_none()
        .then(|| board::id_of(Role::Lang))
        .transpose()?;
    let themes = Themes::shipped().map_err(|e| format!("the shipped themes: {e}"))?;
    let look = uilook::running()?;
    Ok(Plan {
        tips,
        away,
        hover_ms: look.tip_ms + g.churn_tip_ms,
        poll_ms: ctx.cfg.timing.poll_ms,
        pid: win::owner(kb.hwnd).0,
        lang,
        skip,
        themes: themes.list.len(),
        theme: look.theme,
        keyboard: kb.hwnd,
        hide: g9.put_away.clone(),
        back: g9.bring_back.clone(),
    })
}

/// What showed in the rounds.
#[derive(Default)]
pub struct Seen {
    /// Hovers that showed a new window at the rest's end.
    pub tips: usize,
    /// Each tip key's hovers, in `[g8] churn_tips` order.
    pub keys: Keys,
    /// List opens that showed a new window.
    pub lists: usize,
    /// Minimises that hid the keyboard and showed the bubble.
    pub bubbles: usize,
    /// Bubble presses that brought the keyboard back.
    pub returns: usize,
    /// Scrolls whose pad lit.
    pub scrolls: usize,
    /// Scrolls whose outline showed every edge.
    pub outlines: usize,
}

impl Seen {
    /// Counts hover `h` on tip key number `i`; the total takes only a tip showing at the rest's end, as before.
    fn tip(&mut self, i: usize, h: Hover) {
        self.tips += usize::from(h.hit);
        self.keys.add(i, h);
    }
}

/// The shown windows of the product's process `pid` now, as plain numbers.
fn shown_now(pid: u32) -> Vec<isize> {
    product::shown_by(pid).into_iter().map(win::key).collect()
}

/// True when a window in `now` was not shown in `base`; a window that hid meanwhile, such as a banner, never counts.
fn any_new(base: &HashSet<isize>, now: &[isize]) -> bool {
    now.iter().any(|w| !base.contains(w))
}

/// The shown windows of the product's process `pid` now, to spot a pop-up later.
fn snapshot(pid: u32) -> HashSet<isize> {
    shown_now(pid).into_iter().collect()
}

/// Rests the pointer on each tip key, looking for a new window through the rest, then moves it away.
fn hover(plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    // The pointer rests away from the keys, so no tip shows yet.
    let base = snapshot(plan.pid);
    for (i, &at) in plan.tips.iter().enumerate() {
        mouse::move_to(at)?;
        let moved = Instant::now();
        let since = || win::ms_since(moved);
        let shown = || any_new(&base, &shown_now(plan.pid));
        let polls = g8tips::rest(plan.hover_ms, plan.poll_ms, since, sleep_ms, shown);
        seen.tip(i, Hover::of(&polls));
    }
    mouse::move_to(plan.away)
}

/// Opens and closes the language list with its key, counting an open that showed a new window; the close still goes
/// out after the mouse guard stopped the gate.
fn list(g: &G8, reader: &Reader, plan: &Plan, lang: &str, seen: &mut Seen) -> Result<(), String> {
    let base = snapshot(plan.pid);
    product::invoke(reader, lang)?;
    sleep_ms(g.churn_settle_ms);
    seen.lists += usize::from(any_new(&base, &shown_now(plan.pid)));
    product::restore(reader, lang)?;
    sleep_ms(g.churn_settle_ms);
    Ok(())
}

/// True when shown window `w` has a button with UI Automation id `id` on screen.
fn holds(reader: &Reader, w: HWND, id: &str) -> bool {
    reader.read(w).is_ok_and(|keys| keys.at(id).is_some())
}

/// Whether minimise swapped the keyboard for its bubble, from what shows after it; a miss when both show. An error
/// when no bubble showed, as nothing could then bring the keyboard back.
fn swapped(keyboard_shown: bool, bubble_shown: bool) -> Result<bool, String> {
    if bubble_shown {
        Ok(!keyboard_shown)
    } else {
        Err(NO_BUBBLE.to_string())
    }
}

/// Minimises the keyboard to its bubble and brings it back, counting a bubble that showed in its place and a keyboard
/// that came back; bringing it back still goes out after the mouse guard stopped the gate.
fn bubble(g: &G8, reader: &Reader, plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    product::invoke(reader, &plan.hide)?;
    sleep_ms(g.churn_settle_ms);
    let shown = product::shown();
    let bubble_shown = shown.iter().any(|&w| holds(reader, w, &plan.back));
    seen.bubbles += usize::from(swapped(shown.contains(&plan.keyboard), bubble_shown)?);
    product::restore(reader, &plan.back)?;
    sleep_ms(g.churn_settle_ms);
    seen.returns += usize::from(product::shown().contains(&plan.keyboard));
    Ok(())
}

/// Nothing when a theme cycle that started on theme `was` ended on it, `now`; else why the churn stops.
fn back_to(was: &str, now: &str) -> Result<(), String> {
    if was == now {
        Ok(())
    } else {
        Err(format!(
            "a theme cycle from {was} ended on {now}, so {THEME_LEFT}"
        ))
    }
}

/// Presses the theme button once per theme, then checks the running copy is back on the owner's theme.
fn themes(g: &G8, reader: &Reader, plan: &Plan) -> Result<(), String> {
    let n = plan.themes;
    for i in 0..n {
        let left = |e| format!("{e}; pressed {i} of {n} times, so {THEME_LEFT}");
        product::invoke(reader, &g.churn_theme).map_err(left)?;
        sleep_ms(g.churn_settle_ms);
    }
    back_to(&plan.theme, &uilook::running()?.theme)
}

/// Scrolls once with `scroller`, counting a pad that lit and an outline that showed.
fn scroll(scroller: &Scroller, seen: &mut Seen) -> Result<(), String> {
    let done = scroller.once()?;
    seen.scrolls += usize::from(done.lit);
    seen.outlines += usize::from(done.outlined);
    Ok(())
}

/// One round: the tips, the list when the language key opens one, the bubble, a full theme cycle, and a scroll when
/// there is a `scroller`; `after` runs after each step, as when it reads memory.
pub fn round(
    (g, reader, plan): (&G8, &Reader, &Plan),
    scroller: Option<&Scroller>,
    seen: &mut Seen,
    after: &mut dyn FnMut(Step) -> Result<(), String>,
) -> Result<(), String> {
    hover(plan, seen)?;
    sleep_ms(g.churn_settle_ms);
    after(Step::Tips)?;
    if let Some(lang) = &plan.lang {
        list(g, reader, plan, lang, seen)?;
        after(Step::List)?;
    }
    bubble(g, reader, plan, seen)?;
    after(Step::Bubble)?;
    themes(g, reader, plan)?;
    after(Step::Themes)?;
    if let Some(s) = scroller {
        scroll(s, seen)?;
        after(Step::Scroll)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

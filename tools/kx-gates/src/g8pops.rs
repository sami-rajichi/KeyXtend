//! One round of G8's churn: key tips, the language list in the List design, the bubble and a full theme cycle. It
//! moves the real pointer but never clicks, presses buttons by UI Automation and counts what showed.
#![cfg(windows)]

use kx_mod_keyboard::{LangKey, Role};
use kx_theme::Themes;
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::Ctx;
use crate::g8cfg::G8;
use crate::keymap::Reader;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{board, g8ui, mouse, uilook};

/// What a round that broke off mid-cycle leaves, for its error.
const THEME_LEFT: &str = "the theme may differ now: press the theme button to bring yours back";

/// What each round does, worked out once.
pub struct Plan {
    /// Screen centres of the keys whose tips show.
    pub tips: Vec<POINT>,
    /// Where the pointer rests between hovers: just off the keyboard.
    pub away: POINT,
    /// The pointer's rest on each key: the running copy's tip time and `[g8] churn_tip_ms`.
    hover_ms: u64,
    /// The language key's id, when its design opens the list.
    pub lang: Option<String>,
    /// Theme button presses that bring the theme back: one per shipped theme.
    themes: usize,
    /// The owner's theme, which each cycle must end on.
    theme: String,
    /// The keyboard's window, which hides while the bubble shows.
    keyboard: HWND,
    /// The buttons that minimise the keyboard and bring it back, from `[g9]`.
    hide: String,
    back: String,
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
    let lists = product::keyboard()?.lang_key == LangKey::List;
    let lang = lists.then(|| board::id_of(Role::Lang)).transpose()?;
    let themes = Themes::shipped().map_err(|e| format!("the shipped themes: {e}"))?;
    let look = uilook::running()?;
    Ok(Plan {
        tips,
        away,
        hover_ms: look.tip_ms + g.churn_tip_ms,
        lang,
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
    /// Hovers that showed a new window.
    pub tips: usize,
    /// List opens that showed a new window.
    pub lists: usize,
    /// Minimises that hid the keyboard and showed the bubble.
    pub bubbles: usize,
    /// Bubble presses that brought the keyboard back.
    pub returns: usize,
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

/// True when shown window `w` has a button with UI Automation id `id` on screen.
fn holds(reader: &Reader, w: HWND, id: &str) -> bool {
    reader.read(w).is_ok_and(|keys| keys.at(id).is_some())
}

/// Minimises the keyboard to its bubble and brings it back, counting a bubble that showed in its place and a keyboard
/// that came back.
fn bubble(g: &G8, reader: &Reader, plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    product::invoke(reader, &plan.hide)?;
    sleep_ms(g.churn_settle_ms);
    let shown = product::shown();
    let swapped =
        !shown.contains(&plan.keyboard) && shown.iter().any(|&w| holds(reader, w, &plan.back));
    seen.bubbles += usize::from(swapped);
    product::invoke(reader, &plan.back)?;
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

/// One round: the tips, the list when the language key opens one, the bubble, and a full theme cycle.
pub fn round(g: &G8, reader: &Reader, plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    // The pointer rests away from the keys, so no tip shows yet.
    let base = product::shown().len();
    hover(plan, base, seen)?;
    sleep_ms(g.churn_settle_ms);
    if let Some(lang) = &plan.lang {
        list(g, reader, lang, base, seen)?;
    }
    bubble(g, reader, plan, seen)?;
    themes(g, reader, plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_theme_cycle_that_ends_elsewhere_stops_the_churn_and_says_how_to_undo_it() {
        assert_eq!(back_to("native", "native"), Ok(()));
        let e = back_to("native", "et66").expect_err("the owner's theme moved");
        assert!(e.contains("native") && e.contains("et66"), "{e}");
        assert!(e.contains(THEME_LEFT), "{e}");
    }

    /// The windows whose buttons the bubble step presses.
    // Workspace paths from `tools/kx-gates/src`; a moved file breaks the build here, never silently.
    const TOP_BAR_QML: &str = include_str!("../../../crates/kx-ui/qml/TopBar.qml");
    const BUBBLE_QML: &str = include_str!("../../../crates/kx-ui/qml/Bubble.qml");

    #[test]
    fn the_bubble_step_presses_the_top_bars_minimise_and_the_bubbles_own_button() {
        let g9 = crate::config::load().expect("kx-gates.toml loads").g9;
        let hide = g9.put_away.strip_prefix("bar.").expect("a top-bar id");
        assert!(TOP_BAR_QML.contains(&format!("part: \"{hide}\"")), "{hide}");
        let (group, part) = g9.bring_back.split_once('.').expect("group.part");
        let uid = format!("uid(\"{group}\", \"{part}\")");
        assert!(BUBBLE_QML.contains(&uid), "{uid}");
    }
}

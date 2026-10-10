//! One round of G8's churn: key tips, the language list in the List design, the bubble and a full theme cycle. It
//! moves the real pointer but never clicks, presses buttons by UI Automation and counts what showed.
#![cfg(windows)]

use std::collections::HashSet;

use kx_mod_keyboard::langkey::lone;
use kx_mod_keyboard::{LangKey, Role};
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
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
    /// Hovers that showed a new window.
    pub tips: usize,
    /// List opens that showed a new window.
    pub lists: usize,
    /// Minimises that hid the keyboard and showed the bubble.
    pub bubbles: usize,
    /// Bubble presses that brought the keyboard back.
    pub returns: usize,
}

/// The product's shown windows now, as plain numbers.
fn shown_now() -> Vec<isize> {
    product::shown().into_iter().map(win::key).collect()
}

/// True when a window in `now` was not shown in `base`; a window that hid meanwhile, such as a banner, never counts.
fn any_new(base: &HashSet<isize>, now: &[isize]) -> bool {
    now.iter().any(|w| !base.contains(w))
}

/// The product's shown windows now, to spot a pop-up later.
fn snapshot() -> HashSet<isize> {
    shown_now().into_iter().collect()
}

/// Rests the pointer on each tip key, counting each hover that showed a new window, then moves it away.
fn hover(plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    // The pointer rests away from the keys, so no tip shows yet.
    let base = snapshot();
    for &at in &plan.tips {
        mouse::move_to(at)?;
        sleep_ms(plan.hover_ms);
        seen.tips += usize::from(any_new(&base, &shown_now()));
    }
    mouse::move_to(plan.away)
}

/// Opens and closes the language list with its key, counting an open that showed a new window.
fn list(g: &G8, reader: &Reader, lang: &str, seen: &mut Seen) -> Result<(), String> {
    let base = snapshot();
    product::invoke(reader, lang)?;
    sleep_ms(g.churn_settle_ms);
    seen.lists += usize::from(any_new(&base, &shown_now()));
    product::invoke(reader, lang)?;
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
/// that came back.
fn bubble(g: &G8, reader: &Reader, plan: &Plan, seen: &mut Seen) -> Result<(), String> {
    product::invoke(reader, &plan.hide)?;
    sleep_ms(g.churn_settle_ms);
    let shown = product::shown();
    let bubble_shown = shown.iter().any(|&w| holds(reader, w, &plan.back));
    seen.bubbles += usize::from(swapped(shown.contains(&plan.keyboard), bubble_shown)?);
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
    hover(plan, seen)?;
    sleep_ms(g.churn_settle_ms);
    if let Some(lang) = &plan.lang {
        list(g, reader, lang, seen)?;
    }
    bubble(g, reader, plan, seen)?;
    themes(g, reader, plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pop_up_counts_by_a_window_new_since_the_step_began_not_by_how_many_show() {
        let base: HashSet<isize> = [1, 2, 3].into();
        assert!(!any_new(&base, &[1, 2, 3]), "nothing new");
        assert!(!any_new(&base, &[1, 2]), "a banner that hid is no pop-up");
        assert!(
            any_new(&base, &[1, 2, 9]),
            "a tip that showed as a banner hid"
        );
    }

    #[test]
    fn a_minimise_counts_a_swap_or_a_miss_and_stops_with_its_cause_when_no_bubble_showed() {
        assert_eq!(
            swapped(false, true),
            Ok(true),
            "the bubble took the keyboard's place"
        );
        assert_eq!(swapped(true, true), Ok(false), "a slow hide is a miss");
        for keyboard in [true, false] {
            let e = swapped(keyboard, false).expect_err("nothing to press back");
            assert!(
                e.contains("no bubble") && e.contains("churn_settle_ms"),
                "{e}"
            );
        }
    }

    #[test]
    fn the_list_step_runs_only_in_the_list_design_with_two_layouts_and_says_why_not() {
        assert_eq!(skip_list(LangKey::List, 2), None);
        let one = skip_list(LangKey::List, 1).expect("one layout opens no list");
        assert!(one.contains("one layout"), "{one}");
        let carousel = skip_list(LangKey::Carousel, 3).expect("the Carousel has no list");
        assert!(carousel.contains("not the List design"), "{carousel}");
    }

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

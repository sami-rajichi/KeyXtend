//! G8's presses type, as keys act at the press (ADR-0024): G8 keeps target-window in front while it presses, so
//! nothing types into any other window, and checks that exactly one `[g8] press_key` character per press arrived.
#![cfg(windows)]

use kx_mod_keyboard::KeyboardSettings;
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use kx_platform_windows::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::apps::{self, Ctx};
use crate::g8cfg::G8;
use crate::g8ui::{self, Presses};
use crate::gatecfg::{G2, below_delay};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{launch, tlog, walk};

/// Refuses G8's G2 clicks and its presses, held up to `press_wait_ms` when a key never lights, when either hold with
/// `[g2] repeat_margin_ms` reaches the repeat delay of a keyboard with `settings`.
fn holds_below(g8: &G8, g2: &G2, settings: &KeyboardSettings) -> Result<(), String> {
    g2.holds_below(settings)?;
    below_delay(
        "g8.press_wait_ms",
        g8.press_wait_ms,
        g2.repeat_margin_ms,
        settings,
    )
}

/// `holds_below` with the running keyboard's settings, so a short repeat delay stops G8 before it measures anything.
pub fn check_running(ctx: &Ctx) -> Result<(), String> {
    holds_below(&ctx.cfg.g8, &ctx.cfg.g2, &product::keyboard()?)
}

/// What key `id` types at once in the layout of the app in front.
fn typed_by_key(id: &str) -> Result<String, String> {
    let layout = WinLayouts
        .foreground()
        .ok_or("no app in front to read a layout from")?;
    let key = walk::char_keys(layout)?.into_iter().find(|k| k.id == id);
    key.and_then(|k| k.types)
        .ok_or_else(|| format!("g8.press_key {id} types nothing at once in the layout in front"))
}

/// True when `text` is `one` typed once per press and nothing else.
fn exact(text: &str, one: &str, presses: usize) -> bool {
    text == one.repeat(presses)
}

/// Ok while `target` is in front, bringing it back if it left; an error when it cannot, so no press types elsewhere.
fn in_front(ctx: &Ctx, target: HWND) -> Result<(), String> {
    if foreground() == target || win::front(target, &ctx.cfg.timing, &ctx.cfg.keys) {
        return Ok(());
    }
    Err("target-window left the front and did not come back, so G8 stopped pressing".to_string())
}

/// What a press will type, read by `read` only once `front` confirmed target-window in front, as the layout read is
/// the front app's.
fn expected(
    front: impl FnOnce() -> Result<(), String>,
    read: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    front()?;
    read()
}

/// What target-window got, read once `[g2] drain_ms` has passed.
fn arrived(ctx: &Ctx) -> Result<String, String> {
    sleep_ms(ctx.cfg.g2.drain_ms);
    let log = std::fs::read_to_string(&ctx.target.log).map_err(|e| format!("target log: {e}"))?;
    Ok(tlog::decode(&tlog::parse(&log)))
}

/// G8's presses into target-window `target`, once the keyboard shows its layout, and what arrived there.
fn into(ctx: &Ctx, kb: &Keyboard, target: HWND) -> Result<Presses, String> {
    sleep_ms(product::follow_ms(ctx.cfg.timing.poll_ms)?);
    let read = || typed_by_key(&ctx.cfg.g8.press_key);
    let one = expected(|| in_front(ctx, target), read)?;
    let mut out = g8ui::presses(ctx, kb, || in_front(ctx, target))?;
    let text = arrived(ctx)?;
    out.arrived = text.chars().count();
    out.typed_ok = exact(&text, &one, ctx.cfg.g8.presses);
    Ok(out)
}

/// G8's presses on keyboard `kb`, typing into a target-window it starts in front and closes at the end.
pub fn presses(ctx: &Ctx, kb: &Keyboard) -> Result<Presses, String> {
    let target = launch::to_front(ctx, launch::start_target(ctx)?)?;
    let out = into(ctx, kb, target.hwnd);
    let left_open = apps::clean_up(ctx, target);
    let mut out = out.map_err(|e| format!("{e}; left open: {left_open:?}"))?;
    out.left_open = left_open;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn the_expected_character_is_read_only_once_the_window_is_in_front() {
        let steps = RefCell::new(Vec::new());
        let front = || {
            steps.borrow_mut().push("front");
            Ok(())
        };
        let read = || {
            steps.borrow_mut().push("read");
            Ok("a".to_string())
        };
        assert_eq!(expected(front, read), Ok("a".to_string()));
        assert_eq!(*steps.borrow(), ["front", "read"]);
        let gone = || Err("target-window left the front".to_string());
        let never = || -> Result<String, String> { panic!("read with another window in front") };
        assert!(expected(gone, never).is_err());
    }

    #[test]
    fn exactly_one_character_per_press_and_nothing_else_passes() {
        assert!(exact("aaa", "a", 3));
        assert!(!exact("aa", "a", 3), "a press typed nothing");
        assert!(!exact("aaaa", "a", 3), "a press repeated");
        assert!(!exact("aab", "a", 3), "a stray character");
        assert!(exact("", "a", 0), "no press, nothing typed");
        assert!(exact("لالا", "لا", 2), "a key that types two characters");
    }

    #[test]
    fn g8s_presses_and_g2s_clicks_stay_a_margin_below_the_running_repeat_delay() {
        let c = crate::config::load().expect("loads");
        let (g8, g2) = (&c.g8, &c.g2);
        let mut running = KeyboardSettings::from_defaults().expect("the defaults load");
        assert_eq!(holds_below(g8, g2, &running), Ok(()));
        running.repeat_delay_ms = g8.press_wait_ms + g2.repeat_margin_ms;
        let e = holds_below(g8, g2, &running).expect_err("a press that never lit would repeat");
        assert!(e.contains("g8.press_wait_ms"), "{e}");
        let quick = G8 {
            press_wait_ms: 0,
            ..g8.clone()
        };
        running.repeat_delay_ms = g2.press_ms + g2.repeat_margin_ms;
        let e = holds_below(&quick, g2, &running).expect_err("G8's G2 clicks would repeat");
        assert!(e.contains("g2.press_ms"), "{e}");
        running.repeat = false;
        assert_eq!(holds_below(g8, g2, &running), Ok(()), "nothing repeats");
    }
}

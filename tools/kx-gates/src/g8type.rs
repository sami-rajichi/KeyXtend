//! G8's presses type, as keys act at the press (ADR-0024): G8 keeps target-window in front while it presses, so
//! nothing types into any other window, and checks that exactly one `[g8] press_key` character per press arrived.
#![cfg(windows)]

use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use kx_platform_windows::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::apps::{self, Ctx};
use crate::g8ui::{self, Presses};
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{launch, tlog, walk};

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

/// What target-window got, read once `[g2] drain_ms` has passed.
fn arrived(ctx: &Ctx) -> Result<String, String> {
    sleep_ms(ctx.cfg.g2.drain_ms);
    let log = std::fs::read_to_string(&ctx.target.log).map_err(|e| format!("target log: {e}"))?;
    Ok(tlog::decode(&tlog::parse(&log)))
}

/// G8's presses into target-window `target`, once the keyboard shows its layout, and what arrived there.
fn into(ctx: &Ctx, kb: &Keyboard, target: HWND) -> Result<Presses, String> {
    sleep_ms(product::follow_ms(ctx.cfg.timing.poll_ms)?);
    let one = typed_by_key(&ctx.cfg.g8.press_key)?;
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
    use super::*;

    #[test]
    fn exactly_one_character_per_press_and_nothing_else_passes() {
        assert!(exact("aaa", "a", 3));
        assert!(!exact("aa", "a", 3), "a press typed nothing");
        assert!(!exact("aaaa", "a", 3), "a press repeated");
        assert!(!exact("aab", "a", 3), "a stray character");
        assert!(exact("", "a", 0), "no press, nothing typed");
        assert!(exact("لالا", "لا", 2), "a key that types two characters");
    }
}

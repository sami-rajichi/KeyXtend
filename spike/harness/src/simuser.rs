//! The probes' simulated user: moves, clicks, grabs and Esc, each only on our own window and tab.

use spike_core::hold::{Act, Button, ESC, Pt};
use spike_core::inject::{self, TAG};
use spike_core::window::foreground;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MOUSEEVENTF_LEFTUP, VK_CONTROL, VK_LBUTTON, VK_LWIN, VK_MENU, VK_SHIFT,
};

use crate::apps::{Ctx, Opened};
use crate::assist::{self, Assist, Setup};
use crate::hookio::Source;
use crate::win::{self, sleep_ms};
use crate::{keys, mouse};

/// The high bit of `GetAsyncKeyState`: the key is down now.
const KEY_DOWN: i16 = i16::MIN;

/// Fails unless our tab is the one shown and every point lands on our window.
pub fn guard(app: &Opened, pts: &[Pt]) -> Result<(), String> {
    if let Some(t) = &app.title_has
        && !win::title(app.hwnd).contains(t.as_str())
    {
        return Err(format!("our tab is not shown: {}", win::describe(app.hwnd)));
    }
    for p in pts {
        let top = win::root_at(POINT { x: p.x, y: p.y });
        if top != app.hwnd {
            return Err(format!("{} covers {},{}", win::describe(top), p.x, p.y));
        }
    }
    Ok(())
}

/// Fails unless our window is in front with our tab shown, so keys reach only us.
pub fn keys_ours(app: &Opened) -> Result<(), String> {
    in_front(app.hwnd)?;
    guard(app, &[])
}

/// Fails unless `hwnd` is the window in front, so keys reach only it.
pub fn in_front(hwnd: HWND) -> Result<(), String> {
    let front = foreground();
    if front != hwnd {
        return Err(format!(
            "keys held back: {} is in front",
            win::describe(front)
        ));
    }
    Ok(())
}

/// Starts the engine on the simulated user's input; presses inside `own` are never held.
pub fn start_assist(ctx: &Ctx, own: Vec<RECT>) -> Result<Assist, String> {
    let a = &ctx.cfg.assist;
    Assist::start(Setup {
        hold_ms: a.hold_ms,
        still_px: a.still_px,
        source: Source::Simulated,
        own,
        reply_ms: a.reply_ms,
    })
}

/// `steps` even points from just after `from` to exactly `to`.
pub fn path(from: Pt, to: Pt, steps: i32) -> Vec<Pt> {
    let n = steps.max(1);
    (1..=n)
        .map(|i| Pt {
            x: from.x + (to.x - from.x) * i / n,
            y: from.y + (to.y - from.y) * i / n,
        })
        .collect()
}

/// The pointer glides from `from` to `to`.
fn glide(ctx: &Ctx, from: Pt, to: Pt) -> Result<(), String> {
    for at in path(from, to, ctx.cfg.probes.move_steps) {
        assist::as_user(&[Act::Move(at)])?;
        sleep_ms(ctx.cfg.sim.step_ms);
    }
    Ok(())
}

/// A short left click at `p` on our window, then a pause to settle.
pub fn click(ctx: &Ctx, app: &Opened, p: Pt) -> Result<(), String> {
    let sim = &ctx.cfg.sim;
    guard(app, &[p])?;
    assist::as_user(&[Act::Move(p)])?;
    sleep_ms(sim.step_ms);
    assist::as_user(&[Act::Down(Button::Left, p)])?;
    sleep_ms(sim.click_ms);
    let up = assist::as_user(&[Act::Up(Button::Left, p)]);
    sleep_ms(ctx.cfg.probes.settle_ms);
    up
}

/// Holds still at `from` past the hold time, lets go, then glides to `to`: Grab now carries from `from`.
pub fn grab(ctx: &Ctx, app: &Opened, from: Pt, to: Pt) -> Result<(), String> {
    let sim = &ctx.cfg.sim;
    guard(app, &[from, to])?;
    assist::as_user(&[Act::Move(from)])?;
    sleep_ms(sim.step_ms);
    assist::as_user(&[Act::Down(Button::Left, from)])?;
    sleep_ms(ctx.cfg.assist.hold_ms + sim.hold_margin_ms);
    assist::as_user(&[Act::Up(Button::Left, from)])?;
    sleep_ms(sim.step_ms);
    glide(ctx, from, to)
}

/// Taps Esc, only while our window is in front.
pub fn esc(app: &Opened) -> Result<(), String> {
    keys_ours(app)?;
    assist::as_user(&[Act::KeyDown(ESC), Act::KeyUp(ESC)])
}

/// True when virtual key `vk`, a key or a mouse button, is up now.
pub fn is_up(vk: u16) -> bool {
    // SAFETY: a plain key-state query.
    let state = unsafe { GetAsyncKeyState(i32::from(vk)) };
    state & KEY_DOWN == 0
}

/// A last safety net: lets go of the left button, Shift, Ctrl, Alt and Win if one is still down; true if it had to.
pub fn release_all() -> Result<bool, String> {
    let mut inputs = Vec::new();
    if !is_up(VK_LBUTTON.0) {
        inputs.push(mouse::input(0, 0, MOUSEEVENTF_LEFTUP, TAG));
    }
    for vk in [VK_SHIFT.0, VK_CONTROL.0, VK_MENU.0, VK_LWIN.0] {
        if !is_up(vk) {
            inputs.push(keys::tagged(vk, true, TAG));
        }
    }
    if inputs.is_empty() {
        return Ok(false);
    }
    inject::send(&inputs).map(|()| true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_ends_exactly_at_the_target_in_even_steps() {
        let pts = path(Pt { x: 0, y: 0 }, Pt { x: 10, y: -5 }, 5);
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], Pt { x: 2, y: -1 });
        assert_eq!(pts[4], Pt { x: 10, y: -5 });
        assert_eq!(
            path(Pt::default(), Pt { x: 3, y: 3 }, 0),
            [Pt { x: 3, y: 3 }]
        );
    }
}

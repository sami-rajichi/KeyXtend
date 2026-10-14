//! The probes' simulated user: moves, clicks and grabs, each only on our own window and tab, and clicks on the
//! product's keys, all as `SIM_TAG` input, which only the test build's engine hears; the safety net's releases go out
//! tagged `TAG`.
#![cfg(windows)]

use kx_platform_windows::keys::TAG;
use kx_platform_windows::window::foreground;
use spike_core::pt::Pt;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTUP, VK_CONTROL, VK_LBUTTON,
    VK_LWIN, VK_MENU, VK_RBUTTON, VK_SHIFT,
};

use crate::apps::{Ctx, Opened};
use crate::product::Keyboard;
use crate::simin::{self, Act};
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
            return Err(format!("{} covers a test point", win::describe(top)));
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
        simin::as_user(&[Act::Move(at)])?;
        sleep_ms(ctx.cfg.sim.step_ms);
    }
    Ok(())
}

/// A short left click at `p`.
fn press(ctx: &Ctx, p: Pt) -> Result<(), String> {
    let sim = &ctx.cfg.sim;
    simin::as_user(&[Act::Move(p)])?;
    sleep_ms(sim.step_ms);
    simin::as_user(&[Act::Down(p)])?;
    sleep_ms(sim.click_ms);
    simin::as_user(&[Act::Up(p)])
}

/// A short left click at `p` on our window, then a pause to settle.
pub fn click(ctx: &Ctx, app: &Opened, p: Pt) -> Result<(), String> {
    guard(app, &[p])?;
    let clicked = press(ctx, p);
    sleep_ms(ctx.cfg.probes.settle_ms);
    clicked
}

/// A short click at `from`, then a glide to `to`: Grab picks up at the press and drops its release, so it carries from
/// `from`.
pub fn grab(ctx: &Ctx, app: &Opened, from: Pt, to: Pt) -> Result<(), String> {
    guard(app, &[from, to])?;
    press(ctx, from)?;
    sleep_ms(ctx.cfg.sim.step_ms);
    glide(ctx, from, to)
}

/// A short click at the centre of the product's key `id`, once that point shows the keyboard window, then a pause to
/// settle. In the test build a click on our window cancels a carry, and the engine sends Esc itself.
pub fn click_key(ctx: &Ctx, kb: &Keyboard, id: &str) -> Result<(), String> {
    let at = kb.top_at(id)?;
    let clicked = press(ctx, Pt { x: at.x, y: at.y });
    sleep_ms(ctx.cfg.probes.settle_ms);
    clicked
}

/// True when virtual key `vk`, a key or a mouse button, is up now.
#[allow(unsafe_code, reason = "A plain key-state query.")]
pub fn is_up(vk: u16) -> bool {
    // SAFETY: a plain key-state query.
    let state = unsafe { GetAsyncKeyState(i32::from(vk)) };
    state & KEY_DOWN == 0
}

/// The releases of each button or key `down` names: the left button, the right one the engine injects for a
/// right-click, then Shift, Ctrl, Alt and Win.
fn releases(down: impl Fn(u16) -> bool) -> Vec<INPUT> {
    let buttons = [
        (VK_LBUTTON.0, MOUSEEVENTF_LEFTUP),
        (VK_RBUTTON.0, MOUSEEVENTF_RIGHTUP),
    ];
    let ups = buttons.into_iter().filter(|&(vk, _)| down(vk));
    let mut inputs: Vec<INPUT> = ups.map(|(_, up)| mouse::input(0, 0, up, TAG)).collect();
    let mods = [VK_SHIFT.0, VK_CONTROL.0, VK_MENU.0, VK_LWIN.0];
    inputs.extend(
        mods.into_iter()
            .filter(|&vk| down(vk))
            .map(|vk| keys::tagged(vk, true, TAG)),
    );
    inputs
}

/// A last safety net: lets go of either button, Shift, Ctrl, Alt and Win if one is still down; true if it had to.
pub fn release_all() -> Result<bool, String> {
    let inputs = releases(|vk| !is_up(vk));
    if inputs.is_empty() {
        return Ok(false);
    }
    keys::send(&inputs).map(|()| true)
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::INPUT_MOUSE;

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

    #[allow(unsafe_code, reason = "Reads the union variant of the checked type.")]
    #[test]
    fn the_right_button_the_engine_injects_is_let_go_too() {
        assert!(releases(|_| false).is_empty(), "nothing is down");
        let right = releases(|vk| vk == VK_RBUTTON.0);
        assert_eq!(right.len(), 1);
        assert_eq!(right[0].r#type, INPUT_MOUSE);
        // SAFETY: mouse input, as checked above.
        let mi = unsafe { right[0].Anonymous.mi };
        assert_eq!((mi.dwFlags, mi.dwExtraInfo), (MOUSEEVENTF_RIGHTUP, TAG));
        assert_eq!(releases(|_| true).len(), 6, "two buttons and four keys");
    }
}

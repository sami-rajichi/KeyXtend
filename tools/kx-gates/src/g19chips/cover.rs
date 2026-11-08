//! Why our keyboard's key is covered: the windows above the keyboard in the z-order, where the window over the key
//! sits, the keyboard's styles, cut and layering, and the key's box read again. Classes and numbers only, no titles.
#![cfg(windows)]

use serde_json::{Value, json};
use windows::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateRectRgn, DeleteObject, GetRgnBox, GetWindowRgn, HGDIOBJ, PtInRegion, RGN_ERROR,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GET_WINDOW_CMD, GW_HWNDNEXT, GW_HWNDPREV, GWL_STYLE, GetLayeredWindowAttributes, GetWindow,
    GetWindowLongW, IsWindowVisible, LAYERED_WINDOW_ATTRIBUTES_FLAGS, WS_DISABLED, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_VISIBLE,
};

use crate::g9win::ex_of;
use crate::product::Keyboard;
use crate::win;

/// What the note says where nothing was found or read.
const NONE: &str = "none";
/// The extended style bits the note names.
const EX_BITS: [(&str, u32); 5] = [
    ("layered", WS_EX_LAYERED.0),
    ("transparent", WS_EX_TRANSPARENT.0),
    ("topmost", WS_EX_TOPMOST.0),
    ("noactivate", WS_EX_NOACTIVATE.0),
    ("toolwindow", WS_EX_TOOLWINDOW.0),
];
/// The plain style bits the note names.
const STYLE_BITS: [(&str, u32); 2] = [("visible", WS_VISIBLE.0), ("disabled", WS_DISABLED.0)];

/// The names of the bits of `bits` that `names` lists.
fn named(bits: u32, names: &[(&'static str, u32)]) -> Vec<&'static str> {
    let set = names.iter().filter(|(_, b)| bits & b != 0);
    set.map(|(n, _)| *n).collect()
}

/// A window met on the walk: its class, and whether it is topmost, shown and holds the key's point.
struct Met {
    class: String,
    top: bool,
    shown: bool,
    holds: bool,
}

/// The walk's windows worth naming, nearest first: each shown one, and each one out of the topmost band, which a
/// topmost keyboard should have none of above it; with how many hidden topmost ones were passed.
fn named_rows(met: &[Met]) -> Value {
    let worth = met.iter().filter(|m| m.shown || !m.top);
    let rows: Vec<Value> = worth
        .map(|m| json!([m.class, m.top, m.shown, m.holds]))
        .collect();
    json!({ "rows": rows, "hidden_topmost": met.len() - rows.len() })
}

/// Where window `cover` sits from the keyboard: `above n` or `below n` windows away; `none` when neither walk met it.
fn place<W: PartialEq>(up: &[W], down: &[W], cover: &W) -> String {
    let at = |list: &[W]| list.iter().position(|w| w == cover).map(|i| i + 1);
    match (at(up), at(down)) {
        (Some(n), _) => format!("above {n}"),
        (None, Some(n)) => format!("below {n}"),
        (None, None) => NONE.into(),
    }
}

/// The windows next to `w` in z-order direction `dir`, nearest first, at most as many as there are top-level windows.
#[allow(unsafe_code, reason = "Plain z-order queries.")]
fn walk(w: HWND, dir: GET_WINDOW_CMD) -> Vec<HWND> {
    let most = win::all_windows().len();
    let (mut out, mut at) = (Vec::new(), w);
    for _ in 0..most {
        // SAFETY: a plain query; the end of the list or a gone window gives an error, which ends the walk.
        match unsafe { GetWindow(at, dir) } {
            Ok(next) if !next.is_invalid() => {
                out.push(next);
                at = next;
            }
            _ => break,
        }
    }
    out
}

/// Window `w` as the walk meets it, against the key's point `at`.
#[allow(unsafe_code, reason = "A plain visibility query.")]
fn met(w: HWND, at: POINT) -> Met {
    let inside = |r: RECT| (r.left..r.right).contains(&at.x) && (r.top..r.bottom).contains(&at.y);
    // SAFETY: a plain query; any handle value is allowed.
    let shown = unsafe { IsWindowVisible(w) }.as_bool();
    Met {
        class: win::class(w),
        top: ex_of(w) & WS_EX_TOPMOST.0 != 0,
        shown,
        holds: win::rect(w).is_ok_and(inside),
    }
}

/// The keyboard window's named style bits and whether the compositor hides it.
#[allow(unsafe_code, reason = "A plain style read.")]
fn styles(hwnd: HWND) -> Value {
    // SAFETY: a plain query; a gone window reads as 0.
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) }.cast_unsigned();
    let (ex, plain) = (named(ex_of(hwnd), &EX_BITS), named(style, &STYLE_BITS));
    json!({ "ex": ex, "style": plain, "cloaked": win::cloaked(hwnd) })
}

/// The keyboard window's cut, its box relative to the window and whether it holds the key's point `at`; `none` with no
/// cut.
#[allow(
    unsafe_code,
    reason = "Reads a window's region into one of our own, then frees it."
)]
fn cut(hwnd: HWND, at: POINT) -> Value {
    let Ok(r) = win::rect(hwnd) else {
        return json!(NONE);
    };
    let (x, y) = (at.x.saturating_sub(r.left), at.y.saturating_sub(r.top));
    // SAFETY: the region is ours, filled by Windows, read, then deleted; `b` is a local the call fills.
    unsafe {
        let rgn = CreateRectRgn(0, 0, 0, 0);
        let v = if GetWindowRgn(hwnd, rgn) == RGN_ERROR {
            json!(NONE)
        } else {
            let mut b = RECT::default();
            let _ = GetRgnBox(rgn, &raw mut b);
            let holds = PtInRegion(rgn, x, y).as_bool();
            json!({ "box": [b.left, b.top, b.right, b.bottom], "holds_key": holds })
        };
        let _ = DeleteObject(HGDIOBJ(rgn.0));
        v
    }
}

/// The keyboard window's whole-window alpha and its flags, when it is layered that way; `none` otherwise.
#[allow(unsafe_code, reason = "Fills three locals.")]
fn layer(hwnd: HWND) -> Value {
    let (mut key, mut alpha) = (COLORREF(0), 0u8);
    let mut flags = LAYERED_WINDOW_ATTRIBUTES_FLAGS(0);
    let (k, a, f) = (&raw mut key, &raw mut alpha, &raw mut flags);
    // SAFETY: three live locals, as the call takes; a window not layered this way gives an error.
    let got = unsafe { GetLayeredWindowAttributes(hwnd, Some(k), Some(a), Some(f)) };
    got.map_or_else(
        |_| json!(NONE),
        |()| json!({ "alpha": alpha, "flags": flags.0 }),
    )
}

/// What lies over key `id` of keyboard `kb`, read now, and why, in classes and numbers.
pub fn note(kb: &Keyboard, id: &str) -> Value {
    let at = kb.at(id).unwrap_or_default();
    let (up, down) = (walk(kb.hwnd, GW_HWNDPREV), walk(kb.hwnd, GW_HWNDNEXT));
    let cover = win::root_at(at);
    let met: Vec<Met> = up.iter().map(|&w| met(w, at)).collect();
    let key = kb.rect(id).map(|r| [r.left, r.top, r.right, r.bottom]);
    json!({
        "above": named_rows(&met), "cover": [win::class(cover), place(&up, &down, &cover)],
        "hit": win::class(win::under(at)), "keyboard": styles(kb.hwnd), "cut": cut(kb.hwnd, at),
        "layer": layer(kb.hwnd), "key_again": key.ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_bits_a_list_names_are_told_in_its_order() {
        let names = [
            ("layered", 0x8_0000),
            ("transparent", 0x20),
            ("topmost", 0x8),
        ];
        assert_eq!(named(0x8_0008 | 0x100, &names), ["layered", "topmost"]);
        assert!(named(0, &names).is_empty());
    }

    #[test]
    fn the_walk_names_shown_windows_and_any_out_of_the_band_and_counts_the_rest() {
        let met = |class: &str, top, shown, holds| Met {
            class: class.into(),
            top,
            shown,
            holds,
        };
        let walk = [
            met("tooltips_class32", true, false, false),
            met("IME", false, false, false),
            met("Notepad", false, true, true),
        ];
        let want = json!({
            "rows": [["IME", false, false, false], ["Notepad", false, true, true]],
            "hidden_topmost": 1
        });
        assert_eq!(named_rows(&walk), want);
    }

    #[test]
    fn the_cover_is_placed_above_or_below_the_keyboard_or_not_at_all() {
        let (up, down) = ([4, 9], [7, 3, 5]);
        assert_eq!(place(&up, &down, &9), "above 2");
        assert_eq!(place(&up, &down, &5), "below 3");
        assert_eq!(place(&up, &down, &1), NONE);
    }
}

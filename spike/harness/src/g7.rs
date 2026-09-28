//! G7 Arabic panel: the face's right-to-left panel takes an Arabic word with harakat, and its caret stops only between letters.
//!
//! It opens the panel with the Settings key and types into it, and presses keys only while the panel is in front.

use serde_json::{Value, json};
use spike_core::legend::is_mark;
use spike_core::uia::{self, Uia};
use spike_core::{inject, window};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::IUIAutomationElement;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_END, VK_HOME, VK_LEFT, VK_RIGHT,
};

use crate::apps::Ctx;
use crate::featcfg::Move;
use crate::simuser::in_front;
use crate::win::{self, sleep_ms};
use crate::{facetools, keys, mouse};

/// The side key that opens and closes the panel.
const OPENER: &str = "settings";

/// A caret key's virtual key (adapter table).
fn vk(m: Move) -> VIRTUAL_KEY {
    match m {
        Move::Home => VK_HOME,
        Move::End => VK_END,
        Move::Left => VK_LEFT,
        Move::Right => VK_RIGHT,
    }
}

/// Where the caret may stop in `word`, in UTF-16 units: at the start, before each letter and at the end, never before a mark.
fn stops(word: &str) -> Vec<usize> {
    let mut at = 0;
    let mut out = Vec::new();
    for c in word.chars() {
        if at == 0 || !is_mark(c) {
            out.push(at);
        }
        at += c.len_utf16();
    }
    out.push(at);
    out
}

/// Where the caret should be after each of `keys` in right-to-left `word`, starting at the end.
fn expected(word: &str, keys: &[Move]) -> Vec<usize> {
    let s = stops(word);
    let last = s.len() - 1;
    let mut i = last;
    let step = |k: &Move| {
        i = match k {
            Move::Home => 0,
            Move::End => last,
            Move::Left => (i + 1).min(last),
            Move::Right => i.saturating_sub(1),
        };
        s[i]
    };
    keys.iter().map(step).collect()
}

/// Opens the panel on face `name`, runs the check, then closes the panel and checks that it went.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let t = &ctx.cfg.timing;
    let uia = Uia::new()?;
    let face = mouse::find_face(ctx.spike, name)?;
    let pid = win::owner(face).0;
    let opener = ctx.spike.keys.get(OPENER).ok_or("no Settings key")?;
    let key = uia
        .named(face, &opener.name)
        .ok_or("the Settings key has no UIA element")?;
    // The key toggles the panel, so one left open is closed first.
    if shown(ctx, pid).is_some() && !(uia::invoke(&key).is_ok() && gone(ctx, pid)) {
        return Err("a panel left open did not close".into());
    }
    uia::invoke(&key)?;
    let panel = win::poll_until(ctx.cfg.g7.open_ms, t.poll_ms, || shown(ctx, pid))
        .ok_or("the panel did not open")?;
    let out = check(ctx, &uia, panel);
    let closed = close(ctx, &uia, panel, pid, &key);
    let unclosed = |e: String| {
        if closed {
            e
        } else {
            format!("{e}; the panel did not close")
        }
    };
    let mut v = out.map_err(unclosed)?;
    v["pass"] = json!(v["pass"] == json!(true) && closed);
    v["closed"] = json!(closed);
    v["face"] = json!(name);
    Ok(v)
}

/// The face's visible panel.
fn shown(ctx: &Ctx, pid: u32) -> Option<HWND> {
    facetools::shown_by(pid, &ctx.spike.panel.title)
}

/// Types the word into the panel's field, then presses each caret key and reads where the caret went.
fn check(ctx: &Ctx, uia: &Uia, panel: HWND) -> Result<Value, String> {
    let (g, p, t) = (&ctx.cfg.g7, &ctx.spike.panel, &ctx.cfg.timing);
    let field = win::poll_until(t.read_wait_ms, t.poll_ms, || uia.named(panel, &p.field))
        .ok_or("the panel has no text field")?;
    uia::focus(&field)?;
    // Brought forward without keys: an Alt tap would reach the app in front.
    let forward = || (window::foreground() == panel || window::bring_back(panel, 0)).then_some(());
    let _ = win::poll_until(t.focus_wait_ms, t.poll_ms, forward);
    in_front(panel)?;
    inject::text(&g.word)?;
    let typed = || (uia::value(&field).as_deref() == Some(g.word.as_str())).then_some(());
    let typed_ok = win::poll_until(t.read_wait_ms, t.poll_ms, typed).is_some();
    let tp = uia.text_of(&field).ok_or("the field has no TextPattern")?;
    let mut caret = Vec::new();
    for &k in &g.keys {
        in_front(panel)?;
        keys::combo(&[vk(k).0])?;
        sleep_ms(t.key_settle_ms);
        caret.push(uia::caret(&tp)?);
    }
    let want = expected(&g.word, &g.keys);
    let pass = typed_ok && caret == want;
    Ok(json!({ "gate": "G7", "typed_ok": typed_ok, "caret": caret, "want": want, "pass": pass }))
}

/// Closes the panel with its close button, else the Settings key; true once it has gone.
fn close(ctx: &Ctx, uia: &Uia, panel: HWND, pid: u32, key: &IUIAutomationElement) -> bool {
    let pressed = match uia.named(panel, &ctx.spike.panel.close.name) {
        Some(button) => uia::invoke(&button),
        None => uia::invoke(key),
    };
    pressed.is_ok() && gone(ctx, pid)
}

/// Waits for the face's panel to go; true when it went.
fn gone(ctx: &Ctx, pid: u32) -> bool {
    let t = &ctx.cfg.timing;
    let hidden = || shown(ctx, pid).is_none().then_some(());
    win::poll_until(t.close_wait_ms, t.poll_ms, hidden).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Marhaban with four harakat: meem, fatha, reh, sukun, hah, fatha, beh, fathatan, alef.
    const WORD: &str = "مَرْحَبًا";

    #[test]
    fn the_caret_stops_before_each_letter_and_at_the_end() {
        assert_eq!(stops(WORD), [0, 2, 4, 6, 8, 9]);
    }

    #[test]
    fn the_start_is_a_stop_even_before_a_mark() {
        assert_eq!(stops("\u{064E}\u{0645}"), [0, 1, 2]);
    }

    #[test]
    fn in_right_to_left_text_left_goes_on_and_right_goes_back() {
        use crate::featcfg::Move::*;
        let keys = [End, Right, Right, Left, Home, Left, End, Left, Home, Right];
        assert_eq!(expected(WORD, &keys), [9, 8, 6, 8, 0, 2, 9, 9, 0, 0]);
    }

    #[test]
    fn the_configured_keys_visit_every_stop_and_both_ends_twice() {
        let g = crate::config::load().expect("harness.toml loads").g7;
        let seen = expected(&g.word, &g.keys);
        assert!(stops(&g.word).iter().all(|s| seen.contains(s)), "{seen:?}");
        let at = |i: usize| seen.windows(2).filter(|w| w[0] == i && w[1] == i).count();
        assert!(
            at(0) > 0 && at(g.word.encode_utf16().count()) > 0,
            "no push past an end: {seen:?}"
        );
    }
}

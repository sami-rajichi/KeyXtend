//! Clicks on the product keyboard's character keys, shared by G2 and G4: the keys, a click loop, and focus checks.
#![cfg(windows)]

use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use spike_core::clock::now_us;
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::{Ctx, Opened};
use crate::mouse;
use crate::out::say;
use crate::product::{self, Keyboard};
use crate::rng::Rng;
use crate::stats::Click;
use crate::walk;
use crate::win::{self, sleep_ms};

/// A clickable key: scan code, the character it types in the target's layout, screen centre in physical pixels.
pub struct Key {
    code: u32,
    text: String,
    at: POINT,
}

/// Counts and records of one click run.
#[derive(Default)]
pub struct Tally {
    /// Every click made, in order.
    pub clicks: Vec<Click>,
    /// Clicks after which the target was still in front.
    pub focus_kept: usize,
    /// Clicks after which the target's caret was unchanged.
    pub caret_kept: usize,
    /// The first failed checks, for the report.
    pub failures: Vec<Value>,
    /// Failed checks so far.
    pub failed: usize,
    /// Why the run stopped early.
    pub stopped: Option<String>,
    /// Clicks skipped because another window covered the key.
    pub covered: usize,
    /// The windows that covered keys, each named once.
    pub covered_by: Vec<String>,
}

impl Tally {
    /// Counts one click skipped because `who` covered the key; names at most `shown` windows.
    fn skip(&mut self, who: String, shown: usize) {
        self.covered += 1;
        if self.covered_by.len() < shown && !self.covered_by.contains(&who) {
            self.covered_by.push(who);
        }
    }
}

/// The text the clicks should have typed, in click order.
pub fn typed(clicks: &[Click]) -> String {
    let units: Vec<u16> = clicks
        .iter()
        .flat_map(|c| c.expect.iter().copied())
        .collect();
    String::from_utf16_lossy(&units)
}

/// The keyboard's character keys that type in the layout of the app in front, and the ones UI Automation did not
/// find; none with no app in front.
fn keys(kb: &Keyboard) -> Result<(Vec<Key>, Vec<String>), String> {
    let (mut found, mut missing) = (Vec::new(), Vec::new());
    let Some(layout) = WinLayouts.foreground() else {
        return Ok((found, missing));
    };
    for key in walk::char_keys(layout)? {
        let Some(text) = key.types else {
            continue;
        };
        match kb.at(&key.id) {
            Ok(at) => found.push(Key {
                code: u32::from(key.scan.code()),
                text,
                at,
            }),
            Err(e) => missing.push(e),
        }
    }
    Ok((found, missing))
}

/// Splits keys into those on top and those another window covers, named by that window.
fn visible(keys: Vec<Key>, kb: HWND) -> (Vec<Key>, Vec<String>) {
    let (shown, hidden): (Vec<Key>, Vec<Key>) =
        keys.into_iter().partition(|k| win::root_at(k.at) == kb);
    let hidden = hidden
        .iter()
        .map(|k| format!("{:#X} under {}", k.code, win::describe(win::root_at(k.at))))
        .collect();
    (shown, hidden)
}

/// The keys a click run can use, and how long it first waited for the keyboard to follow the app in front.
pub struct Usable {
    /// The printable keys on top.
    pub keys: Vec<Key>,
    /// The keys left out, covered or not found, by name.
    pub hidden: Vec<String>,
    /// The wait, from the running copy's relabel period.
    pub follow_ms: u64,
}

/// The printable keys on top of keyboard `kb`, and the ones it leaves out. It first waits until the keyboard shows
/// the layout of the app in front, as a click on a stale layout sends nothing.
pub fn usable(ctx: &Ctx, kb: &Keyboard) -> Result<Usable, String> {
    let follow_ms = product::follow_ms(ctx.cfg.timing.poll_ms)?;
    sleep_ms(follow_ms);
    let (keys, mut hidden) = keys(kb)?;
    let (keys, covered) = visible(keys, kb.hwnd);
    hidden.extend(covered);
    if keys.is_empty() {
        return Err(format!(
            "no printable key on top; left out: {}",
            hidden.join("; ")
        ));
    }
    if !hidden.is_empty() {
        say!("left out {} keys: {}", hidden.len(), hidden.join("; "));
    }
    Ok(Usable {
        keys,
        hidden,
        follow_ms,
    })
}

/// Clicks one key; the character that arrives, not the cursor, shows where it landed.
fn click(key: &Key, tally: &mut Tally) -> Result<(), String> {
    let us = now_us();
    mouse::click_at(key.at)?;
    tally.clicks.push(Click {
        code: key.code,
        us,
        expect: key.text.encode_utf16().collect(),
    });
    Ok(())
}

/// Checks focus and caret after a click; `Err` stops the run.
fn check(
    ctx: &Ctx,
    target: &Opened,
    caret0: HWND,
    key: &Key,
    tally: &mut Tally,
) -> Result<(), String> {
    let (cfg, g) = (ctx.cfg, &ctx.cfg.g2);
    let front = foreground();
    let caret = win::caret(win::owner(target.hwnd).1);
    let (focus_ok, caret_ok) = (front == target.hwnd, caret == caret0);
    tally.focus_kept += usize::from(focus_ok);
    tally.caret_kept += usize::from(caret_ok);
    if focus_ok && caret_ok {
        return Ok(());
    }
    tally.failed += 1;
    if tally.failures.len() < g.failures_shown {
        tally.failures.push(json!({
            "index": tally.clicks.len() - 1, "key": format!("{:#X}", key.code), "label": key.text,
            "front": win::describe(front), "caret": win::describe(caret),
        }));
    }
    if !focus_ok && !win::front(target.hwnd, &cfg.timing, &cfg.keys) {
        return Err("the target could not be brought back".to_string());
    }
    if tally.failed >= g.max_failures {
        return Err(format!("{} failures", tally.failed));
    }
    Ok(())
}

/// Clicks random keys of keyboard window `kb`; a key another window covers is skipped, never clicked.
pub fn click_loop(ctx: &Ctx, target: &Opened, kb: HWND, keys: &[Key], run: (usize, u64)) -> Tally {
    let g = &ctx.cfg.g2;
    let caret0 = win::caret(win::owner(target.hwnd).1);
    let mut rng = Rng::new(run.1);
    let mut tally = Tally::default();
    while tally.clicks.len() < run.0 {
        let key = &keys[rng.below(keys.len())];
        let top = win::root_at(key.at);
        if top != kb {
            tally.skip(win::describe(top), g.failures_shown);
            if tally.covered >= g.max_failures {
                tally.stopped = Some(format!("{} keys covered by other windows", tally.covered));
                break;
            }
            continue;
        }
        let step = click(key, &mut tally).and_then(|()| {
            sleep_ms(g.click_gap_ms);
            check(ctx, target, caret0, key, &mut tally)
        });
        if let Err(reason) = step {
            tally.stopped = Some(reason);
            break;
        }
    }
    tally
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covered_keys_are_counted_and_each_window_named_once() {
        let mut t = Tally::default();
        for who in ["osk", "osk", "menu", "tip"] {
            t.skip(who.to_string(), 2);
        }
        assert_eq!(t.covered, 4);
        assert_eq!(t.covered_by, vec!["osk".to_string(), "menu".to_string()]);
    }

    #[test]
    fn typed_joins_each_click_text_in_click_order() {
        let click = |s: &str| Click {
            code: 0,
            us: 0,
            expect: s.encode_utf16().collect(),
        };
        assert_eq!(typed(&[click("a"), click("لا"), click("€")]), "aلا€");
        assert_eq!(typed(&[]), "");
    }
}

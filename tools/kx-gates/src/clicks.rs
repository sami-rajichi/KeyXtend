//! Clicks on the product keyboard's character keys, shared by G2 and G4: the keys, a click loop, and focus checks.
#![cfg(windows)]

use kx_module_api::ports::LayoutSource;
use kx_platform_windows::WinLayouts;
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use spike_core::clock::now_us;
use windows::Win32::Foundation::{HWND, POINT};

use crate::apps::{Ctx, Opened};
use crate::g2hand;
use crate::out::say;
use crate::product::{self, Keyboard};
use crate::rng::Rng;
use crate::stats::Click;
use crate::walk;
use crate::win::{self, sleep_ms};

/// A clickable key: scan code, the character it types in the target's layout, screen centre and the point a click
/// slips to, in physical pixels.
pub struct Key {
    code: u32,
    text: String,
    at: POINT,
    to: POINT,
}

/// Counts and records of one click run.
#[derive(Default)]
pub struct Tally {
    /// Every click made, in order.
    pub clicks: Vec<Click>,
    /// When each click let go, in QPC µs.
    pub released: Vec<i64>,
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

/// How many clicks a run makes, from which seed, and whether it first clicks each key once in board order.
#[derive(Debug, Clone, Copy)]
pub struct Run {
    /// Clicks in all, the sweep's included.
    pub count: usize,
    /// Seed of the random clicks.
    pub seed: u64,
    /// Clicks each key once in board order before the random clicks, so no key goes untyped.
    pub sweep: bool,
}

/// The key for step `step` of `run` over `n` keys: each in board order first when sweeping, then a random one.
fn pick(run: &Run, step: usize, n: usize, rng: &mut Rng) -> usize {
    if run.sweep && step < n {
        step
    } else {
        rng.below(n)
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

/// Where a click on key `id` goes down and where it slips to, `slip_px` away inside the key.
fn spots(kb: &Keyboard, id: &str, slip_px: i32) -> Result<(POINT, POINT), String> {
    let at = kb.at(id)?;
    let to = g2hand::slip_to(at, kb.rect(id)?, slip_px)
        .ok_or_else(|| format!("a {slip_px} px slip leaves key {id}"))?;
    Ok((at, to))
}

/// The keyboard's character keys that type in the layout of the app in front, and the ones UI Automation did not
/// find or a slip of `slip_px` would leave; none with no app in front.
fn keys(kb: &Keyboard, slip_px: i32) -> Result<(Vec<Key>, Vec<String>), String> {
    let (mut found, mut missing) = (Vec::new(), Vec::new());
    let Some(layout) = WinLayouts.foreground() else {
        return Ok((found, missing));
    };
    for key in walk::char_keys(layout)? {
        let Some(text) = key.types else {
            continue;
        };
        match spots(kb, &key.id, slip_px) {
            Ok((at, to)) => found.push(Key {
                code: u32::from(key.scan.code()),
                text,
                at,
                to,
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

/// The printable keys on top of keyboard `kb`, and the ones it leaves out. It refuses a hold the running keyboard
/// would repeat, then waits until it shows the layout of the app in front, as a click on a stale layout sends nothing.
pub fn usable(ctx: &Ctx, kb: &Keyboard) -> Result<Usable, String> {
    let running = product::keyboard()?;
    ctx.cfg.g2.holds_below(&running)?;
    let follow_ms = product::follow_after(&running, ctx.cfg.timing.poll_ms);
    sleep_ms(follow_ms);
    let (keys, mut hidden) = keys(kb, ctx.cfg.g2.slip_px)?;
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

/// Clicks one key as a hand does, held `press_ms` with a slip; the character that arrives, not the cursor, shows
/// where it landed.
fn click(key: &Key, press_ms: u64, tally: &mut Tally) -> Result<(), String> {
    let us = now_us();
    g2hand::perform(&g2hand::steps(key.at, key.to, press_ms), g2hand::act)?;
    tally.clicks.push(Click {
        code: key.code,
        us,
        expect: key.text.encode_utf16().collect(),
    });
    tally.released.push(now_us());
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

/// Clicks keys of keyboard window `kb` as `run` says; a key another window covers is skipped, never clicked.
pub fn click_loop(ctx: &Ctx, target: &Opened, kb: HWND, keys: &[Key], run: Run) -> Tally {
    let g = &ctx.cfg.g2;
    let caret0 = win::caret(win::owner(target.hwnd).1);
    let mut rng = Rng::new(run.seed);
    let mut tally = Tally::default();
    let mut step = 0;
    while tally.clicks.len() < run.count {
        let key = &keys[pick(&run, step, keys.len(), &mut rng)];
        step += 1;
        let top = win::root_at(key.at);
        if top != kb {
            tally.skip(win::describe(top), g.failures_shown);
            if tally.covered >= g.max_failures {
                tally.stopped = Some(format!("{} keys covered by other windows", tally.covered));
                break;
            }
            continue;
        }
        let step = click(key, g.press_ms, &mut tally).and_then(|()| {
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
    fn a_sweep_clicks_each_key_once_in_board_order_before_the_random_clicks() {
        let sweep = Run {
            count: 6,
            seed: 7,
            sweep: true,
        };
        let mut rng = Rng::new(sweep.seed);
        let picks: Vec<usize> = (0..6).map(|step| pick(&sweep, step, 4, &mut rng)).collect();
        assert_eq!(picks[..4], [0, 1, 2, 3]);
        assert!(picks[4..].iter().all(|&i| i < 4));
        let random = Run {
            sweep: false,
            ..sweep
        };
        let (mut a, mut b) = (Rng::new(random.seed), Rng::new(random.seed));
        let got: Vec<usize> = (0..4).map(|step| pick(&random, step, 4, &mut a)).collect();
        let want: Vec<usize> = (0..4).map(|_| b.below(4)).collect();
        assert_eq!(
            got, want,
            "without a sweep every click is random, as before"
        );
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

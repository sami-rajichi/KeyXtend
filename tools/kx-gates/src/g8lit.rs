//! G8's key light alone, by its screen copy: `g8 lit` on the running product with no cold start, and `g8 osk` on
//! Windows' On-Screen Keyboard as a baseline. Neither starts, closes nor fronts a keyboard; both type into target-window.
#![cfg(windows)]

use serde::Deserialize;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, RECT};

use crate::apps::Ctx;
use crate::budget;
use crate::g8cfg::G8;
use crate::g8parts::highlight;
use crate::g8type::{self, Expect};
use crate::g8ui::{self, Presses, Spot};
use crate::keymap::{KeyRead, Reader, centre};
use crate::product::Keyboard;
use crate::win;

/// The name that times the running product's key light.
pub const LIT: &str = "lit";
/// The name that times the On-Screen Keyboard's.
pub const OSK: &str = "osk";

/// `[osk]`: Windows' On-Screen Keyboard, which `g8 osk` presses but never starts or closes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Osk {
    /// Its program's file name.
    pub program: String,
    /// The key it presses, by UI Automation name or id in any letter case.
    pub key: String,
}

impl Osk {
    /// Refuses settings that name no program or key.
    pub fn check(&self) -> Result<(), String> {
        if self.program.is_empty() || self.key.is_empty() {
            return Err("osk.program and osk.key must name a program and a key".to_string());
        }
        Ok(())
    }
}

/// Runs `g8 lit` or `g8 osk`; none for another name.
pub fn run(ctx: &Ctx, name: &str) -> Option<Result<Value, String>> {
    match name {
        LIT => Some(lit(ctx)),
        OSK => Some(osk(ctx)),
        _ => None,
    }
}

/// The running product's `[g8] press_key`, pressed as G8 presses it.
fn lit(ctx: &Ctx) -> Result<Value, String> {
    g8type::check_running(ctx)?;
    let kb = Keyboard::find()?;
    let key = &ctx.cfg.g8.press_key;
    let presses = g8type::presses(ctx, &Spot::of(&kb, key)?, Expect::Key(key))?;
    Ok(summary(&ctx.cfg.g8, LIT, &presses, g8ui::refresh_hz()))
}

/// The On-Screen Keyboard's `[osk] key`, pressed as G8 presses a key; it stops when that keyboard is not shown or the
/// key is covered.
fn osk(ctx: &Ctx) -> Result<Value, String> {
    let o = &ctx.cfg.osk;
    let shown = shown_by(&o.program);
    if shown.is_empty() {
        return Err(format!(
            "{} is not running or not shown; G8 never starts it, so start it and run g8 osk again",
            o.program
        ));
    }
    let (hwnd, key) = find_key(&shown, &o.key)?;
    let spot = Spot {
        key,
        at: centre(&key),
        window: win::rect(hwnd)?,
    };
    if win::root_at(spot.at) != hwnd {
        return Err(format!(
            "another window covers {}'s key {}",
            o.program, o.key
        ));
    }
    let presses = g8type::presses(ctx, &spot, Expect::Same)?;
    Ok(summary(&ctx.cfg.g8, OSK, &presses, g8ui::refresh_hz()))
}

/// The shown windows of program `name`, topmost first.
fn shown_by(name: &str) -> Vec<HWND> {
    let ours = |w: &HWND| win::program_name(*w).is_some_and(|p| p.eq_ignore_ascii_case(name));
    win::seen_windows().into_iter().filter(ours).collect()
}

/// The first of `windows` with a button named `want`, and that button's box.
fn find_key(windows: &[HWND], want: &str) -> Result<(HWND, RECT), String> {
    let reader = Reader::new()?;
    for &hwnd in windows {
        if let Some(r) = pick(&reader.walk(hwnd)?, want) {
            return Ok((hwnd, r));
        }
    }
    Err(format!("no shown button is named {want} by UI Automation"))
}

/// The box of the first button in `keys` whose name or id is `want` in any letter case and that has an area on screen.
fn pick(keys: &[KeyRead], want: &str) -> Option<RECT> {
    let named = |k: &&KeyRead| k.name.eq_ignore_ascii_case(want) || k.id.eq_ignore_ascii_case(want);
    let shown = |k: &&KeyRead| k.rect.right > k.rect.left && k.rect.bottom > k.rect.top;
    keys.iter().filter(named).find(shown).map(|k| k.rect)
}

/// The result line of keyboard `name`'s `presses` on a screen of `hz`: each light, G8's two key light parts (the median
/// and the slowest), and whether every press typed one character.
fn summary(g: &G8, name: &str, presses: &Presses, hz: Option<f64>) -> Value {
    let parts = highlight(g, presses, hz);
    json!({
        "gate": "G8", "keyboard": name, "hz": hz, "presses": presses, "parts": parts,
        "pass": budget::all_pass(&parts) && presses.typed_ok,
    })
}

#[cfg(test)]
mod tests {
    use kx_module_api::MS_PER_S;

    use super::*;
    use crate::g8parts::{LIT_P50, LIT_WORST};

    fn key(id: &str, name: &str, rect: RECT) -> KeyRead {
        KeyRead {
            id: id.into(),
            name: name.into(),
            rect,
            invokable: true,
        }
    }

    const BOX: RECT = RECT {
        left: 10,
        top: 20,
        right: 50,
        bottom: 60,
    };

    #[test]
    fn a_key_is_found_by_its_name_or_id_in_any_case_and_only_with_an_area() {
        let flat = RECT { right: 10, ..BOX };
        let keys = [key("", "a", flat), key("", "Q", BOX), key("kA", "A", BOX)];
        assert_eq!(pick(&keys, "q"), Some(BOX), "by name, any case");
        assert_eq!(pick(&keys, "KA"), Some(BOX), "by id");
        assert_eq!(pick(&keys, "a"), Some(BOX), "the flat one is skipped");
        assert_eq!(pick(&keys, "z"), None);
    }

    /// Part `i` of result `v`: its name, what it got and its limit.
    fn part(v: &Value, i: usize) -> (&str, Option<f64>, f64) {
        let p = &v["parts"][i];
        let name = p["name"].as_str().expect("a name");
        (
            name,
            p["got"].as_f64(),
            p["limit"].as_f64().expect("a limit"),
        )
    }

    #[test]
    fn the_summary_judges_the_median_and_the_slowest_light_as_g8_does() {
        let g = crate::config::load().expect("loads").g8;
        let p = Presses {
            lit_ms: vec![Some(9.0), Some(14.0), Some(11.0)],
            copy_ms: vec![2.0],
            typed_ok: true,
            ..Presses::default()
        };
        let v = summary(&g, OSK, &p, Some(60.0));
        let within = |frames: f64| frames * f64::from(MS_PER_S) / 60.0 + 2.0;
        let (median, slowest) = (part(&v, 0), part(&v, 1));
        assert_eq!((median.0, median.1), (LIT_P50, Some(11.0)));
        assert_eq!((slowest.0, slowest.1), (LIT_WORST, Some(14.0)));
        assert!((median.2 - within(g.highlight_frames)).abs() < 1e-9);
        assert!((slowest.2 - within(g.highlight_worst_frames)).abs() < 1e-9);
        assert_eq!(v["pass"], true);
        let late = |lit: [f64; 3]| Presses {
            lit_ms: lit.map(Some).to_vec(),
            copy_ms: vec![2.0],
            typed_ok: true,
            ..Presses::default()
        };
        let slow_median = late([within(g.highlight_frames) + 1.0; 3]);
        assert_eq!(summary(&g, OSK, &slow_median, Some(60.0))["pass"], false);
        let slow_one = late([9.0, 11.0, within(g.highlight_worst_frames) + 1.0]);
        assert_eq!(summary(&g, OSK, &slow_one, Some(60.0))["pass"], false);
    }

    #[test]
    fn the_summary_fails_a_lost_character_an_unlit_key_or_no_refresh_rate() {
        let g = crate::config::load().expect("loads").g8;
        let mut p = Presses {
            lit_ms: vec![Some(9.0), Some(14.0), Some(11.0)],
            copy_ms: vec![2.0],
            typed_ok: false,
            ..Presses::default()
        };
        assert_eq!(
            summary(&g, OSK, &p, Some(60.0))["pass"],
            false,
            "a lost character"
        );
        p.typed_ok = true;
        assert_eq!(summary(&g, OSK, &p, None)["pass"], false, "no refresh rate");
        p.lit_ms.push(None);
        assert_eq!(
            summary(&g, OSK, &p, Some(60.0))["pass"],
            false,
            "a key that never lit"
        );
    }

    #[test]
    fn the_shipped_osk_settings_name_a_program_and_a_key() {
        let o = crate::config::load().expect("loads").osk;
        assert_eq!(o.check(), Ok(()));
        for blank in [
            Osk {
                program: String::new(),
                ..o.clone()
            },
            Osk {
                key: String::new(),
                ..o.clone()
            },
        ] {
            assert!(blank.check().is_err(), "{blank:?}");
        }
    }
}

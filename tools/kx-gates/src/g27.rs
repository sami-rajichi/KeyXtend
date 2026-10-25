//! G27 (P7): a word typed through the product's first chip, per language. In target-window's box the product's keys type
//! each `[g27]` case's prefix, the first chip that offers it is picked, and the box must then hold the prefix replaced
//! by the chip's word and the gap its script takes. Edge 6 picks the chip after a click moved the caret: nothing types.
//! The result holds each case's language, prefix, chip time and verdict, never the box's text or a word.
#![cfg(windows)]

use kx_mod_predict::script;
use serde_json::{Value, json};

use crate::apps::{self, Ctx};
use crate::g25::part;
use crate::g27type::{self, Typist};
use crate::p7cfg::Case;
use crate::product::{self, Keyboard};
use crate::win::{self, sleep_ms};
use crate::{launch, probe};

/// The text a box holding `typed` holds after a pick of `word`: `typed` less its last `prefix`, then the word and the
/// gap its script takes; none when `typed` does not end with `prefix` or the word has no script.
pub(crate) fn after_pick(typed: &str, prefix: &str, word: &str) -> Option<String> {
    let kept = typed.strip_suffix(prefix)?;
    let gap = script::of(word)?.gap();
    Some(format!("{kept}{word}{gap}"))
}

/// True when `got` is `want` code point for code point: no folding, no normalising.
pub(crate) fn same(got: &str, want: &str) -> bool {
    got.chars().eq(want.chars())
}

/// What one fresh box gave: the drive's result, whether the app's first layout came back, and what was left open.
struct Boxed<T> {
    got: Result<T, String>,
    back: bool,
    left: Vec<String>,
}

/// A fresh target-window in front with its box focused, `drive` run on it, then its first layout put back and the
/// window closed.
fn in_box<T>(ctx: &Ctx, drive: impl FnOnce(&Typist) -> Result<T, String>) -> Boxed<T> {
    let started = launch::start_target(ctx).and_then(|app| launch::to_front(ctx, app));
    let app = match started {
        Ok(app) => app,
        Err(e) => {
            return Boxed {
                got: Err(e),
                back: true,
                left: Vec::new(),
            };
        }
    };
    let typist = g27type::focus_target(ctx, &app).and_then(|()| Typist::new(ctx, &app));
    let (got, back) = match typist {
        Ok(t) => (drive(&t), t.put_back()),
        Err(e) => (Err(e), true),
    };
    let mut left = probe::released();
    left.extend(apps::clean_up(ctx, app));
    Boxed { got, back, left }
}

/// Reads the box until it holds `want` or `[g27] text_wait_ms` passed: the last text read.
fn box_until(ctx: &Ctx, t: &Typist, want: &str) -> Result<String, String> {
    let mut last = t.box_text();
    let holds = |r: &Result<String, String>| r.as_deref().is_ok_and(|got| same(got, want));
    win::poll_until(ctx.cfg.g27.text_wait_ms, ctx.cfg.timing.poll_ms, || {
        last = t.box_text();
        holds(&last).then_some(())
    });
    last
}

/// One case: its layout, its prefix typed, a chip that offers it picked, and the box judged.
fn case(ctx: &Ctx, t: &Typist, c: &Case) -> Result<Value, String> {
    let layout = t.switch(&c.lang)?;
    let before = t.box_text()?;
    let last = t.type_in(layout, &c.prefix)?;
    let (_, chip_ms) = t.offered(c, last)?;
    let typed = t.box_text()?;
    let typed_ok = same(&typed, &format!("{before}{}", c.prefix));
    let word = t.pick()?;
    let want = after_pick(&typed, &c.prefix, &word);
    let got = match &want {
        Some(want) => Some(box_until(ctx, t, want)?),
        None => None,
    };
    let ok = typed_ok && want.zip(got).is_some_and(|(w, g)| same(&g, &w));
    Ok(
        json!({ "lang": c.lang, "prefix": c.prefix, "chip_ms": chip_ms, "prefix_typed": typed_ok, "ok": ok }),
    )
}

/// Edge 6 in a fresh box: the prefix, a click at the box's start, then the same first chip, which must type nothing.
fn edge(ctx: &Ctx, t: &Typist, c: &Case) -> Result<Value, String> {
    let g = &ctx.cfg.g27;
    let layout = t.switch(&c.lang)?;
    let last = t.type_in(layout, &c.prefix)?;
    let (word, chip_ms) = t.offered(c, last)?;
    let typed = t.box_text()?;
    t.click_start()?;
    sleep_ms(g.settle_ms);
    let kept = t.first_word().as_deref() == Some(word.as_str());
    if kept {
        t.click_first()?;
        sleep_ms(g.text_wait_ms);
    }
    let ok = kept && same(&t.box_text()?, &typed);
    Ok(
        json!({ "lang": c.lang, "prefix": c.prefix, "chip_ms": chip_ms, "lane_kept": kept, "ok": ok }),
    )
}

/// True when every result in `parts` passed and there is one.
fn all_ok(parts: &[Value]) -> bool {
    !parts.is_empty() && parts.iter().all(|p| p["ok"] == true)
}

/// Runs G27 on the product, called `name`, which must be running: every case in one box, then edge 6 in another.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name != product::NAME {
        return Err(format!("G27 runs on {}, not {name}", product::NAME));
    }
    Keyboard::find()?;
    let g = &ctx.cfg.g27;
    let edge_case = g.case(&g.edge).ok_or("g27.edge names no case")?;
    let each = |t: &Typist| Ok(g.cases.iter().map(|c| part(case(ctx, t, c))).collect());
    let words: Boxed<Vec<Value>> = in_box(ctx, each);
    let edged = in_box(ctx, |t| edge(ctx, t, edge_case));
    let cases = words.got.unwrap_or_else(|e| vec![part(Err(e))]);
    let edge = part(edged.got);
    let back = words.back && edged.back;
    let left: Vec<String> = words.left.into_iter().chain(edged.left).collect();
    let pass = all_ok(&cases) && all_ok(std::slice::from_ref(&edge)) && back;
    Ok(json!({
        "gate": "G27", "keyboard": name, "cases": cases, "edge": edge, "layout_put_back": back, "clean_up": left,
        "pass": pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prefix_gives_way_to_the_chips_word_and_its_scripts_gap() {
        assert_eq!(after_pick("hel", "hel", "hello").as_deref(), Some("hello "));
        assert_eq!(after_pick("ان", "ان", "أن").as_deref(), Some("أن "));
        assert_eq!(
            after_pick("hello eco", "eco", "école").as_deref(),
            Some("hello école "),
            "the text before stays"
        );
        assert_eq!(after_pick("hel", "eco", "école"), None, "not typed last");
        assert_eq!(after_pick("hel", "hel", "123"), None, "no script");
    }

    #[test]
    fn a_word_whose_script_spaces_nothing_takes_no_gap() {
        let thai = "\u{0E2A}\u{0E27}\u{0E31}\u{0E2A}\u{0E14}\u{0E35}";
        assert_eq!(
            after_pick("\u{0E2A}", "\u{0E2A}", thai).as_deref(),
            Some(thai)
        );
    }

    #[test]
    fn only_the_exact_text_passes_with_no_folding() {
        let want = after_pick("hel", "hel", "hello").expect("a text");
        assert!(same("hello ", &want));
        for wrong in ["hello", "Hello ", "hello  ", "helhello ", ""] {
            assert!(!same(wrong, &want), "{wrong:?}");
        }
        let ar = after_pick("ان", "ان", "أن").expect("a text");
        assert!(same("أن ", &ar));
        assert!(!same("ان ", &ar), "no hamza folding");
        assert!(
            !same("\u{0627}\u{0654}\u{0646} ", &ar),
            "a decomposed hamza is other code points"
        );
        assert!(!same("e\u{0301}cole ", "école "), "no normalising");
    }

    #[test]
    fn g27_passes_only_when_every_part_passed() {
        let good = json!({ "ok": true });
        assert!(all_ok(&[good.clone(), good.clone()]));
        assert!(!all_ok(&[good, json!({ "ok": false })]));
        assert!(!all_ok(&[]));
    }
}

//! G27 (P7): a word typed through the product's first chip, per language, on copies of the product G27 starts with
//! learning paused. In target-window's box the product's keys type each `[g27]` case's prefix, the first chip that
//! offers it is picked, and the box must then hold the prefix replaced by the chip's word and its script's gap. Edge 6
//! picks the chip after a click moved the caret, with the mouse module off: nothing types. The result holds each
//! case's language, prefix, chip time and verdict, never the box's text or a word.
#![cfg(windows)]

use kx_mod_predict::script;
use kx_module_api::{ModuleId, Redacted};
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::copyend::{self, Ended};
use crate::g25::part;
use crate::g27type::Typist;
use crate::owncopy::Session;
use crate::p7cfg::Case;
use crate::product;
use crate::typebox::{self, Boxed};
use crate::win::{self, sleep_ms};

/// The text a box holding `typed` holds after a pick of `word`: `typed` less its last `prefix`, then the word and the
/// gap its script takes; none when `typed` does not end with `prefix` or the word has no script.
pub(crate) fn after_pick(typed: &str, prefix: &str, word: &str) -> Option<String> {
    let kept = typed.strip_suffix(prefix)?;
    let gap = script::of(word)?.gap();
    Some(format!("{kept}{word}{gap}"))
}

/// Edge 6's verdict when the pick typed nothing.
const EDGE_PASS: &str = "typed nothing";
/// Edge 6's verdict when the pick changed the text.
const EDGE_TYPED: &str = "typed";
/// Edge 6's verdict when chip-0 left the lane after the click, so the pick check was never reached.
const EDGE_SETUP: &str = "setup failed: chip-0 left the lane after the click";
/// The copies G27 starts: one for the cases, one for edge 6 with the mouse module off.
pub const COPIES: usize = 2;
/// Why edge 6 did not run.
const EDGE_SKIPPED: &str = "edge 6 did not run: the first copy could not be ended";

/// Edge 6's verdict: whether the lane `kept` chip-0 after the click, and whether the text stayed `unchanged`.
pub(crate) fn edge_verdict(kept: bool, unchanged: bool) -> &'static str {
    match (kept, unchanged) {
        (false, _) => EDGE_SETUP,
        (true, true) => EDGE_PASS,
        (true, false) => EDGE_TYPED,
    }
}

/// True when `got` is `want` code point for code point: no folding, no normalising.
pub(crate) fn same(got: &str, want: &str) -> bool {
    got.chars().eq(want.chars())
}

/// What one copy's boxes gave: each box's result line, whether every layout came back, and what clean-up left.
struct Boxes {
    parts: Vec<Value>,
    back: bool,
    left: Vec<String>,
}

impl Boxes {
    /// The boxes `run` gave, each judged.
    fn of(run: Vec<Boxed<Value>>) -> Self {
        let back = run.iter().all(|b| b.back);
        let left = run.iter().flat_map(|b| b.left.clone()).collect();
        let parts = run.into_iter().map(|b| part(b.got)).collect();
        Self { parts, back, left }
    }

    /// No box ran, for `why`.
    fn failed(why: String) -> Self {
        Self {
            parts: vec![part(Err(why))],
            back: true,
            left: Vec::new(),
        }
    }
}

/// `boxes` run on a fresh copy of the product, with learning paused and the modules `off` switched off: what they
/// gave, and how the copy went.
fn on_copy(
    session: &Session,
    off: &[ModuleId],
    boxes: impl FnOnce() -> Vec<Boxed<Value>>,
) -> (Boxes, Option<Ended>) {
    let (got, ended) = session.with_copy(off, || Ok(boxes()));
    (got.map_or_else(Boxes::failed, Boxes::of), ended)
}

/// Reads the box until it holds `want` or `[g27] text_wait_ms` passed: the last text read.
fn box_until(ctx: &Ctx, t: &Typist, want: &str) -> Result<Redacted<String>, String> {
    let mut last = t.box_text();
    let holds =
        |r: &Result<Redacted<String>, String>| r.as_ref().is_ok_and(|got| same(got.expose(), want));
    win::poll_until(ctx.cfg.g27.text_wait_ms, ctx.cfg.timing.poll_ms, || {
        last = t.box_text();
        holds(&last).then_some(())
    });
    last
}

/// One case in a fresh box: its layout, its prefix typed, a chip that offers it picked, and the box judged.
fn case(ctx: &Ctx, t: &Typist, c: &Case) -> Result<Value, String> {
    let layout = t.switch(&c.lang)?;
    let before = t.box_text()?;
    let last = t.type_in(layout, &c.prefix)?;
    let (_, chip_ms) = t.offered(c, last)?;
    let typed = t.box_text()?;
    let typed_ok = same(typed.expose(), &format!("{}{}", before.expose(), c.prefix));
    let word = t.pick()?;
    let want = after_pick(typed.expose(), &c.prefix, word.expose()).map(Redacted::new);
    let got = match &want {
        Some(want) => Some(box_until(ctx, t, want.expose())?),
        None => None,
    };
    let ok = typed_ok
        && want
            .zip(got)
            .is_some_and(|(w, g)| same(g.expose(), w.expose()));
    Ok(
        json!({ "lang": c.lang, "prefix": c.prefix, "chip_ms": chip_ms, "prefix_typed": typed_ok, "ok": ok }),
    )
}

/// Edge 6 in a fresh box on a copy with the mouse module off: the prefix, a click at the box's start the copy never
/// hears, then the same first chip, whose pick check must type nothing.
fn edge(ctx: &Ctx, t: &Typist, c: &Case) -> Result<Value, String> {
    let g = &ctx.cfg.g27;
    let layout = t.switch(&c.lang)?;
    let last = t.type_in(layout, &c.prefix)?;
    let (word, chip_ms) = t.offered(c, last)?;
    let typed = t.box_text()?;
    t.click_start()?;
    sleep_ms(g.settle_ms);
    let kept = t.first_word().as_ref() == Some(&word);
    let unchanged = if kept {
        t.click_first()?;
        sleep_ms(g.text_wait_ms);
        same(t.box_text()?.expose(), typed.expose())
    } else {
        false
    };
    let verdict = edge_verdict(kept, unchanged);
    Ok(json!({
        "lang": c.lang, "prefix": c.prefix, "chip_ms": chip_ms, "verdict": verdict, "ok": verdict == EDGE_PASS,
    }))
}

/// True when every result in `parts` passed and there is one.
fn all_ok(parts: &[Value]) -> bool {
    !parts.is_empty() && parts.iter().all(|p| p["ok"] == true)
}

/// Runs G27 on the product, called `name`, which must be closed: each case in a fresh box on a copy of its own, then
/// edge 6 on a copy with the mouse module off, unless the first copy could not be ended. Every copy ends before the
/// owner's file comes back.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name != product::NAME {
        return Err(format!("G27 runs on {}, not {name}", product::NAME));
    }
    let g = &ctx.cfg.g27;
    let edge_case = g.case(&g.edge).ok_or("g27.edge names no case")?;
    let session = Session::open(ctx)?;
    let each = || {
        g.cases
            .iter()
            .map(|c| typebox::in_target(ctx, &[], |t| case(ctx, t, c)))
            .collect()
    };
    let (cases, first) = on_copy(&session, &[], each);
    let edged = || vec![typebox::in_target(ctx, &[], |t| edge(ctx, t, edge_case))];
    let (edge, second) = if copyend::may_follow(first.as_ref().map(|e| e.gone)) {
        on_copy(&session, &[kx_mod_mouse::ID], edged)
    } else {
        (Boxes::failed(EDGE_SKIPPED.to_string()), None)
    };
    let settings = session.close();
    let back = cases.back && edge.back;
    let typed = all_ok(&cases.parts) && all_ok(&edge.parts);
    let copies: [Option<Ended>; COPIES] = [first, second];
    let ended = copies.iter().all(|c| c.as_ref().is_some_and(Ended::ok));
    let left = [cases.left, edge.left].concat();
    Ok(json!({
        "gate": "G27", "keyboard": name, "cases": cases.parts, "edge": edge.parts, "layout_put_back": back,
        "clean_up": left, "copies": copies.map(|c| c.map(|e| e.line())),
        "settings_put_back": settings.is_ok(), "settings_error": settings.as_ref().err(),
        "pass": typed && back && ended && settings.is_ok(),
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
    fn edge_6_tells_a_setup_failure_from_a_typing_fail() {
        assert_eq!(edge_verdict(true, true), EDGE_PASS);
        assert_eq!(edge_verdict(true, false), EDGE_TYPED);
        assert_eq!(edge_verdict(false, true), EDGE_SETUP, "no chip-0 to click");
        assert_eq!(edge_verdict(false, false), EDGE_SETUP);
        assert!(EDGE_SETUP.starts_with("setup failed"), "{EDGE_SETUP}");
    }

    #[test]
    fn g27_passes_only_when_every_part_passed() {
        let good = json!({ "ok": true });
        assert!(all_ok(&[good.clone(), good.clone()]));
        assert!(!all_ok(&[good, json!({ "ok": false })]));
        assert!(!all_ok(&[]));
    }
}

//! G6's cases on one window, each a verdict and a note of numbers only: the aim and the speeds, up, Stop, the
//! press-stop, sideways where the window overflows sideways, and the end.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde_json::{Value, json};

use crate::g6judge::{self, Look, Pair};
use crate::g6scene::{Phase, Scene, between};
use crate::keys;
use crate::p5cfg::G6;
use crate::simuser::keys_ours;
use crate::win::{self, sleep_ms};

/// Why a case that needs the pad still scrolling failed: the document was too short for it.
const STOPPED_ITSELF: &str = "the pad stopped itself first";
/// The note's key for such a fault.
const SIZING: &str = "sizing_fault";
/// Why Stop failed: the pad stopped itself on the way up, before Stop went out, which no sizing explains.
const UP_FIRST: &str = "the pad stopped itself on the way up, before Stop";
/// The note's key for that.
const EARLY: &str = "stopped_early";
/// Why press-stop is not judged on Explorer: a click elsewhere could open a file or sort a column.
const NO_SPOT: &str =
    "Explorer showed no spot UI Automation reads as no file or column, so G6 did not click";
/// The press-stop case's name.
const PRESS: &str = "press_stop";

/// One case's verdict: none when it could not be judged, and its note says why.
pub struct Case {
    /// The case's name in the report.
    pub name: &'static str,
    /// Passed, failed, or not judged.
    pub ok: Option<bool>,
    /// Its numbers, or why it was not judged.
    pub note: Value,
}

impl Case {
    /// A judged case.
    fn judged(name: &'static str, ok: bool, note: Value) -> Self {
        Self {
            name,
            ok: Some(ok),
            note,
        }
    }

    /// A case left unjudged, and why.
    fn unjudged(name: &'static str, why: &str) -> Self {
        Self {
            name,
            ok: None,
            note: json!({ "unjudged": why }),
        }
    }

    /// A case judged by `still` on `look` when there was one, else failed as the pad stopped itself.
    fn stilled(name: &'static str, look: Option<Look>, min: f64) -> Self {
        match look {
            Some(l) => Self::judged(name, l.still(min), json!({ "share": l.share })),
            None => Self::too_short(name),
        }
    }

    /// A case that failed as the pad met the document's end before it: a sizing fault, never a pass.
    fn too_short(name: &'static str) -> Self {
        Self::judged(name, false, json!({ SIZING: STOPPED_ITSELF }))
    }

    /// The Stop case, judged by `still` on `look` when Stop went out, else failed as the pad stopped itself first.
    fn stopped(look: Option<Look>, min: f64) -> Self {
        match look {
            Some(l) => Self::judged("stop", l.still(min), json!({ "share": l.share })),
            None => Self::judged("stop", false, json!({ EARLY: UP_FIRST })),
        }
    }

    /// The case's report line.
    pub fn json(&self) -> Value {
        json!({ "case": self.name, "ok": self.ok, "note": self.note })
    }
}

/// The names of `cases` that failed as the pad met the document's end first.
pub fn sizing_faults(cases: &[Case]) -> Vec<&'static str> {
    cases
        .iter()
        .filter(|c| c.note.get(SIZING).is_some())
        .map(|c| c.name)
        .collect()
}

/// Glides onto the pad and taps down three times, slow, fast, then stopped: the aim, down, the ratio and the third tap.
pub fn speeds(s: &Scene, want: f64) -> Result<Vec<Case>, String> {
    let g = &s.ctx.cfg.g6;
    let kept = s.glide_in()?;
    let came = s.came_from(kept);
    s.arrow(Dir::Down)?;
    let slow = s.phase(Dir::Down, g.phase_ms)?;
    s.arrow(Dir::Down)?;
    let fast = s.phase(Dir::Down, g.phase_ms)?;
    let third = if s.idle() {
        None
    } else {
        s.third(Dir::Down)?;
        Some(s.look(g.still_ms)?)
    };
    let min = g.min_changed;
    let down = slow.look.moved(Dir::Down, min) && fast.look.moved(Dir::Down, min);
    let shares = json!({ "slow": slow.look.share, "fast": fast.look.share });
    Ok(vec![
        Case::judged("aim", came && down, json!({ "pointer_came_from_it": came })),
        Case::judged("down", down, shares),
        ratio(&slow, &fast, want, g),
        Case::stilled("third_tap", third, min),
    ])
}

/// The fast ÷ slow ratio against the test build's own, from the two speeds' percent slopes.
fn ratio(slow: &Phase, fast: &Phase, want: f64, g: &G6) -> Case {
    let n = g.slope_reads;
    let (s, f) = (g6judge::rate(&slow.reads, n), g6judge::rate(&fast.reads, n));
    let (Some(s), Some(f)) = (s, f) else {
        return Case::unjudged(
            "ratio",
            "UI Automation read too few percents short of the ends",
        );
    };
    let slack = g.ratio_slack;
    let ok = g6judge::ratio_ok(s, f, want, slack);
    let note = json!({ "slow_pct_s": s, "fast_pct_s": f, "want": want, "slack": slack });
    Case::judged("ratio", ok, note)
}

/// Taps up, then Stop: it went up, and Stop stopped it.
pub fn up_and_stop(s: &Scene) -> Result<Vec<Case>, String> {
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    s.arrow(Dir::Up)?;
    let up = s.look(g.move_ms)?;
    let stopped = if s.stop()? {
        Some(s.look(g.still_ms)?)
    } else {
        None
    };
    Ok(vec![
        Case::judged("up", up.moved(Dir::Up, min), json!({ "share": up.share })),
        Case::stopped(stopped, min),
    ])
}

/// Taps down, clicks the window's safe spot outside our windows while the pad still scrolls, then glides back: the
/// click stopped it. Unjudged, with Stop tapped, when the window shows no safe spot (`Scene::spot`).
pub fn press_stop(s: &Scene) -> Result<Case, String> {
    let Some(spot) = s.spot() else {
        return Ok(Case::unjudged(PRESS, NO_SPOT));
    };
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    s.arrow(Dir::Down)?;
    let going = s.look(g.move_ms)?;
    if s.idle() {
        return Ok(Case::too_short(PRESS));
    }
    if !s.click_out(spot)? {
        s.stop()?;
        return Ok(Case::unjudged(PRESS, NO_SPOT));
    }
    let after = s.look(g.still_ms)?;
    s.glide_in()?;
    let ok = going.moved(Dir::Down, min) && after.still(min);
    let note = json!({ "going": going.share, "after": after.share });
    Ok(Case::judged(PRESS, ok, note))
}

/// Right, then left for `[pad] undo_div` times less, each ended by Stop, where the window overflows sideways or is
/// `wide` by make; unjudged elsewhere. Without a horizontal percent, the picture judges.
pub fn sideways(s: &Scene, wide: bool) -> Result<Case, String> {
    if !wide && !g6judge::sideways(s.pair()) {
        return Ok(Case::unjudged("sideways", "no sideways overflow"));
    }
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    let runs = [
        (Dir::Right, g.move_ms),
        (Dir::Left, s.ctx.cfg.pad.undone(g.move_ms)),
    ];
    let (mut ok, mut shares) = (true, Vec::new());
    for (dir, ms) in runs {
        s.arrow(dir)?;
        let look = s.look(ms)?;
        s.stop()?;
        ok &= look.moved(dir, min);
        shares.push(look.share);
    }
    Ok(Case::judged("sideways", ok, json!({ "shares": shares })))
}

/// From the window's end, up for `back_ms` and Stop, then down until the pad stops by itself; the end case and the
/// percents read at the end.
pub fn end(s: &Scene) -> Result<(Case, Pair), String> {
    let (g, t) = (&s.ctx.cfg.g6, &s.ctx.cfg.timing);
    keys_ours(s.app)?;
    keys::combo(&s.ctx.cfg.edit_keys.doc_end)?;
    sleep_ms(s.ctx.cfg.probes.settle_ms);
    let bottom = s.snap()?;
    s.arrow(Dir::Up)?;
    sleep_ms(g.back_ms);
    s.stop()?;
    let from = s.snap()?;
    s.arrow(Dir::Down)?;
    let idle = win::poll_until(g.end_wait_ms, t.poll_ms, || s.idle().then_some(())).is_some();
    let now = s.snap()?;
    if !idle {
        s.stop()?;
    }
    let (down, back) = (between(&from, &now), between(&bottom, &now));
    let ok = g6judge::end_ok(idle, &down, &back, g.min_changed);
    let note = json!({ "stopped_itself": idle, "down_share": down.share, "end_share": back.share });
    Ok((Case::judged("end", ok, note), bottom.pair))
}

#[cfg(test)]
mod tests;

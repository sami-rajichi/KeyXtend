//! G6's cases on one window, each a verdict and a note of numbers only: the aim and the speeds, up, Stop, the
//! press-stop, sideways where the window overflows sideways, and the end.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde_json::{Value, json};

use crate::g6judge::{self, Look, Pair};
use crate::g6scene::{Phase, Scene, between};
use crate::g6spot::Tried;
use crate::ownkeys::{self, Real};
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
/// The note's key for what UI Automation showed at each spot tried.
const TRIED: &str = "tried";
/// The note's key for a still look's percents, before and after.
const PAIRS: &str = "pairs";
/// The sideways case's name.
const SIDE: &str = "sideways";
/// Why sideways is not judged: the scroll pattern says the window does not scroll sideways.
const NO_OVERFLOW: &str = "no sideways overflow";
/// Why sideways failed on a page G6 makes wide: its scroll pattern says it does not scroll sideways.
const NOT_WIDE: &str =
    "the scroll pattern says this page, made wide by G6, does not scroll sideways";
/// The note's key for that.
const UNSCROLLABLE: &str = "unscrollable";
/// Why the end case is not judged on Explorer: Ctrl+End would not reach its list.
const NO_FOCUS: &str =
    "UI Automation could not put the focus on Explorer's list, so Ctrl+End would miss it";

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
            Some(l) => Self::judged(name, l.still(min), still_note(&l)),
            None => Self::too_short(name),
        }
    }

    /// The press-stop case left unjudged as no spot tried was safe, with what each `tried` showed.
    fn unspotted(tried: &[Tried]) -> Self {
        let tried: Vec<Value> = tried.iter().map(Tried::json).collect();
        Self {
            name: PRESS,
            ok: None,
            note: json!({ "unjudged": NO_SPOT, TRIED: tried }),
        }
    }

    /// The press-stop case: it was `going` down, then stood still `after` the click.
    fn pressed(going: &Look, after: &Look, min: f64) -> Self {
        let ok = going.moved(Dir::Down, min) && after.still(min);
        let pairs = [after.before, after.after];
        let note = json!({ "going": going.share, "after": after.share, PAIRS: pairs });
        Self::judged(PRESS, ok, note)
    }

    /// A case that failed as the pad met the document's end before it: a sizing fault, never a pass.
    fn too_short(name: &'static str) -> Self {
        Self::judged(name, false, json!({ SIZING: STOPPED_ITSELF }))
    }

    /// The Stop case, judged by `still` on `look` when Stop went out, else failed as the pad stopped itself first.
    fn stopped(look: Option<Look>, min: f64) -> Self {
        match look {
            Some(l) => Self::judged("stop", l.still(min), still_note(&l)),
            None => Self::judged("stop", false, json!({ EARLY: UP_FIRST })),
        }
    }

    /// The case's report line.
    pub fn json(&self) -> Value {
        json!({ "case": self.name, "ok": self.ok, "note": self.note })
    }
}

/// A still look's note: its changed share and its percents before and after.
fn still_note(l: &Look) -> Value {
    json!({ "share": l.share, PAIRS: [l.before, l.after] })
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
        Some(s.still_look()?)
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
        Some(s.still_look()?)
    } else {
        None
    };
    Ok(vec![
        Case::judged("up", up.moved(Dir::Up, min), json!({ "share": up.share })),
        Case::stopped(stopped, min),
    ])
}

/// Taps down, clicks the window's safe spot outside our windows while the pad still scrolls, then glides back: the
/// click stopped it. Unjudged when the window shows no safe spot (`Scene::spot`); Stop is tapped when the spot turns
/// unsafe mid-run.
pub fn press_stop(s: &Scene) -> Result<Case, String> {
    let spot = match s.spot() {
        Ok(spot) => spot,
        Err(tried) => return Ok(Case::unspotted(&tried)),
    };
    s.arrow(Dir::Down)?;
    let going = s.look(s.ctx.cfg.g6.move_ms)?;
    if s.idle() {
        return Ok(Case::too_short(PRESS));
    }
    if let Some(refused) = s.click_out(spot)? {
        s.stop()?;
        return Ok(Case::unspotted(&[refused]));
    }
    let after = s.still_look()?;
    s.glide_in()?;
    Ok(Case::pressed(&going, &after, s.ctx.cfg.g6.min_changed))
}

/// The sideways case when it does not run: failed where the page is `wide` by make yet its pattern says it cannot
/// scroll sideways, unjudged where it is not wide and the pattern does not say `scrollable`; none when it runs.
fn unrun(scrollable: Option<bool>, wide: bool) -> Option<Case> {
    if g6judge::sideways(scrollable) || (wide && scrollable.is_none()) {
        return None;
    }
    Some(if wide {
        Case::judged(SIDE, false, json!({ UNSCROLLABLE: NOT_WIDE }))
    } else {
        Case::unjudged(SIDE, NO_OVERFLOW)
    })
}

/// Right, then left for `[pad] undo_div` times less, each ended by Stop, where the scroll pattern says the window
/// scrolls sideways or it is `wide` by make (`unrun`). Without a horizontal percent, the picture judges.
pub fn sideways(s: &Scene, wide: bool) -> Result<Case, String> {
    if let Some(case) = unrun(s.sideways(), wide) {
        return Ok(case);
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
    Ok(Case::judged(SIDE, ok, json!({ "shares": shares })))
}

/// From the window's end, up for `back_ms` and Stop, then down until the pad stops by itself; the end case and the
/// percents read at the end. Unjudged, with none read, when the focus cannot reach Explorer's list for Ctrl+End.
pub fn end(s: &Scene) -> Result<(Case, Pair), String> {
    let (g, t) = (&s.ctx.cfg.g6, &s.ctx.cfg.timing);
    keys_ours(s.app)?;
    if !s.focus_list()? {
        return Ok((Case::unjudged("end", NO_FOCUS), None));
    }
    ownkeys::send(&Real, s.app, &s.ctx.cfg.edit_keys.doc_end)?;
    sleep_ms(s.ctx.cfg.probes.settle_ms);
    let bottom = s.snap()?;
    s.arrow(Dir::Up)?;
    sleep_ms(g.back_ms);
    s.stop()?;
    let from = s.snap()?;
    s.to_end()?;
    let idle = win::poll_until(g.end_wait_ms, t.poll_ms, || s.idle().then_some(())).is_some();
    let now = s.snap()?;
    if !idle {
        s.stop()?;
    }
    let (down, back) = (between(&from, &now), between(&bottom, &now));
    let ok = g6judge::end_ok(idle, &down, &back, g.min_changed);
    let note = json!({
        "stopped_itself": idle, "down_share": down.share, "end_share": back.share, "stopped_pair": now.pair,
    });
    Ok((Case::judged("end", ok, note), bottom.pair))
}

#[cfg(test)]
mod tests;

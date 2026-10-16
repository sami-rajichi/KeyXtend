//! G6's cases on one window, each a verdict and a note of numbers only: the aim and the speeds, up, Stop, the
//! press-stop, sideways where the window overflows sideways, and the end.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use serde_json::{Value, json};

use crate::g6judge::{self, Pair};
use crate::g6scene::{Phase, Scene};
use crate::keys;
use crate::simuser::keys_ours;
use crate::win::{self, sleep_ms};

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

    /// The case's report line.
    pub fn json(&self) -> Value {
        json!({ "case": self.name, "ok": self.ok, "note": self.note })
    }
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
    s.arrow(Dir::Down)?;
    let third = s.look(g.still_ms)?;
    let min = g.min_changed;
    let down = slow.look.moved(Dir::Down, min) && fast.look.moved(Dir::Down, min);
    let shares = json!({ "slow": slow.look.share, "fast": fast.look.share });
    Ok(vec![
        Case::judged("aim", came && down, json!({ "pointer_came_from_it": came })),
        Case::judged("down", down, shares),
        ratio(&slow, &fast, want, g.ratio_slack),
        Case::judged(
            "third_tap",
            third.still(min),
            json!({ "share": third.share }),
        ),
    ])
}

/// The fast ÷ slow ratio against the test build's own, from the two speeds' percent slopes.
fn ratio(slow: &Phase, fast: &Phase, want: f64, slack: f64) -> Case {
    let (s, f) = (g6judge::rate(&slow.reads), g6judge::rate(&fast.reads));
    let (Some(s), Some(f)) = (s, f) else {
        return Case::unjudged(
            "ratio",
            "UI Automation read too few percents short of the ends",
        );
    };
    let ok = g6judge::ratio_ok(s, f, want, slack);
    let note = json!({ "slow_pct_s": s, "fast_pct_s": f, "want": want, "slack": slack });
    Case::judged("ratio", ok, note)
}

/// Taps up, then Stop: it went up, and Stop stopped it.
pub fn up_and_stop(s: &Scene) -> Result<Vec<Case>, String> {
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    s.arrow(Dir::Up)?;
    let up = s.look(g.move_ms)?;
    s.stop()?;
    let stopped = s.look(g.still_ms)?;
    Ok(vec![
        Case::judged("up", up.moved(Dir::Up, min), json!({ "share": up.share })),
        Case::judged(
            "stop",
            stopped.still(min),
            json!({ "share": stopped.share }),
        ),
    ])
}

/// Taps down, clicks the window outside our windows, then glides back: the click stopped it.
pub fn press_stop(s: &Scene) -> Result<Case, String> {
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    s.arrow(Dir::Down)?;
    let going = s.look(g.move_ms)?;
    s.click_out()?;
    let after = s.look(g.still_ms)?;
    s.glide_in()?;
    let ok = going.moved(Dir::Down, min) && after.still(min);
    let note = json!({ "going": going.share, "after": after.share });
    Ok(Case::judged("press_stop", ok, note))
}

/// Right then left, each ended by Stop, where the window overflows sideways; unjudged elsewhere.
pub fn sideways(s: &Scene) -> Result<Case, String> {
    if !g6judge::sideways(s.pair()) {
        return Ok(Case::unjudged("sideways", "no sideways overflow"));
    }
    let (g, min) = (&s.ctx.cfg.g6, s.ctx.cfg.g6.min_changed);
    let (mut ok, mut shares) = (true, Vec::new());
    for dir in [Dir::Right, Dir::Left] {
        s.arrow(dir)?;
        let look = s.look(g.move_ms)?;
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
    let bottom = s.pair();
    s.arrow(Dir::Up)?;
    sleep_ms(g.back_ms);
    s.stop()?;
    s.arrow(Dir::Down)?;
    let idle = win::poll_until(g.end_wait_ms, t.poll_ms, || s.idle().then_some(())).is_some();
    let now = s.pair();
    let at_end = now.is_none() || g6judge::ended(Dir::Down, now);
    if !idle {
        s.stop()?;
    }
    let note = json!({ "stopped_itself": idle, "at_end": at_end });
    Ok((Case::judged("end", idle && at_end, note), bottom))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::g6judge::Look;

    /// The shipped `[g6] ratio_slack`.
    const SLACK: f64 = 0.3;

    /// A phase whose percent climbs `per_s` a second, read every 200 ms.
    fn phase(per_s: f64) -> Phase {
        let reads = (0..10_u32)
            .map(|i| (u64::from(i) * 200, 10.0 + per_s * f64::from(i) / 5.0))
            .collect();
        let look = Look {
            share: None,
            before: None,
            after: None,
        };
        Phase { look, reads }
    }

    #[test]
    fn the_ratio_is_judged_from_both_slopes_and_left_unjudged_without_one() {
        let want = 10.0 / 3.0;
        let near = ratio(&phase(3.0), &phase(10.0), want, SLACK);
        assert_eq!(near.ok, Some(true), "{}", near.note);
        assert_eq!(ratio(&phase(3.0), &phase(3.0), want, SLACK).ok, Some(false));
        let few = Phase {
            reads: vec![(0, 10.0)],
            ..phase(3.0)
        };
        let unread = ratio(&few, &phase(10.0), want, SLACK);
        assert_eq!(unread.ok, None);
        assert_eq!(unread.json()["case"], "ratio");
    }
}

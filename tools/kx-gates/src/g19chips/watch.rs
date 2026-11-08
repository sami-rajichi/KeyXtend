//! The product's own selection watch, run in the gate with the module's defaults around a terminal case: what one look
//! finds before and after the gate's drag, the changes heard between, and the gate's own read of the selection's boxes.
//! Numbers and kinds only, never text, so a missed strip says whether the selection, its event or the look failed.
#![cfg(windows)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use kx_mod_selection::SelSettings;
use kx_module_api::ports::{SelError, SelRun, SelSpot, SelectionWatch};
use kx_platform_windows::WinSelectionWatch;
use serde_json::{Value, json};

use crate::g19chips::gate::Gate;
use crate::win::{self, ms_since};

/// A running watch, and how many settled changes it told.
struct Watch {
    run: Box<dyn SelRun>,
    told: Arc<AtomicUsize>,
}

impl Watch {
    /// A watch with the module's settings `sel`, counting each change it tells.
    fn start(sel: &SelSettings) -> Result<Self, String> {
        let told = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&told);
        let tell = Box::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
        });
        let run = (WinSelectionWatch.watch(sel.sel_config(), tell)).map_err(|e| e.to_string())?;
        Ok(Self { run, told })
    }

    /// The changes told so far.
    fn told(&self) -> usize {
        self.told.load(Ordering::Relaxed)
    }

    /// True once the watch told its first change, the box in front, within `wait_ms`, so a look is not timed from its
    /// start.
    fn ready(&self, wait_ms: u64, poll_ms: u64) -> bool {
        win::poll_until(wait_ms, poll_ms, || (self.told() > 0).then_some(())).is_some()
    }

    /// One look now, and the ms it took.
    fn look(&mut self) -> Value {
        let t = Instant::now();
        let got = self.run.look();
        spot_json(got, ms_since(t))
    }
}

/// What a look found after `ms`: the box's kind, whether its selection holds text and has a line box; or the error.
pub fn spot_json(got: Result<SelSpot, SelError>, ms: u64) -> Value {
    match got {
        Ok(s) => {
            let kind = format!("{:?}", s.kind);
            json!({ "ms": ms, "kind": kind, "selected": s.selected, "line": s.last.is_some() })
        }
        Err(e) => json!({ "ms": ms, "error": format!("{e:?}") }),
    }
}

/// The gate's own read of the focused box's selection: how many line boxes it has, and the ms it took.
fn boxes(gate: &Gate) -> Value {
    let t = Instant::now();
    let got = gate.boxes();
    let ms = ms_since(t);
    got.map_or_else(
        |e| json!({ "ms": ms, "error": e }),
        |b| json!({ "ms": ms, "boxes": b.len() }),
    )
}

/// `act` with the watch around it: a look once the watch is ready, the changes heard during `act`, a look after and
/// the gate's read.
pub fn around<T>(
    gate: &Gate,
    act: impl FnOnce() -> Result<T, String>,
) -> Result<(T, Value), String> {
    let t = &gate.ctx.cfg.timing;
    let watch = Watch::start(&gate.sel).map(|mut w| {
        let ready = w.ready(t.read_wait_ms, t.poll_ms);
        let before = (ready, w.look(), w.told());
        (w, before)
    });
    let got = act()?;
    let seen = match watch {
        Ok((mut w, (ready, look, told))) => {
            let heard = w.told().saturating_sub(told);
            let after = w.look();
            json!({ "ready": ready, "before": look, "heard": heard, "after": after, "read": boxes(gate) })
        }
        Err(e) => json!({ "error": e, "read": boxes(gate) }),
    };
    Ok((got, seen))
}

#[cfg(test)]
mod tests {
    use kx_module_api::ports::{Rect, Screen, SelKind};

    use super::*;

    #[test]
    fn a_look_is_told_by_its_kind_flags_and_time_never_its_boxes() {
        let line = Rect {
            x: 4321,
            y: 8765,
            w: 99,
            h: 21,
        };
        let spot = SelSpot {
            box_id: 7,
            kind: SelKind::Terminal,
            selected: true,
            first: Some(line),
            last: Some(line),
            caret: None,
            screen: Screen::default(),
        };
        let v = spot_json(Ok(spot), 30);
        assert_eq!(
            v,
            json!({ "ms": 30, "kind": "Terminal", "selected": true, "line": true })
        );
        assert!(!v.to_string().contains("4321"), "no box: {v}");
        let late = spot_json(Err(SelError::TimedOut), 250);
        assert_eq!(late, json!({ "ms": 250, "error": "TimedOut" }));
    }
}

//! G19 in the gate's own Notepad document: its line boxes read once, as Cut empties a line and UI Automation then
//! skips it; a selection by keys and one by a drag, then Copy, Cut and Paste, with a latched Shift that stays latched.
#![cfg(windows)]

use kx_mod_keyboard::Latch;
use kx_module_api::ports::{Rect, SelChip};
use kx_platform_windows::screen::edges;
use kx_platform_windows::uia;
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;

use crate::apps::Opened;
use crate::board;
use crate::g19chips::docs::{Line, edited, lines_of};
use crate::g19chips::gate::Gate;
use crate::g19chips::place;
use crate::g19chips::strip::Shown;
use crate::modekey::ModeKey;
use crate::win;

/// The acts `edits` takes: two to select by keys, twice, a drag, three keycaps, and Shift latched and let go.
pub const ACTS: usize = 10;

/// The rows of `parts`: each line's parts joined, top to bottom.
pub fn rows(parts: &[Rect]) -> Vec<Rect> {
    let mut out: Vec<Rect> = Vec::new();
    for at in 0..parts.len() {
        if let Some(row) = place::line(parts, at).filter(|r| r.w > 0 && !out.contains(r)) {
            out.push(row);
        }
    }
    out.sort_by_key(|r| r.y);
    out
}

/// The document's text lines on screen, and how far past a line's end a click lands.
pub struct Lines {
    rows: Vec<Rect>,
    inset: i32,
}

impl Lines {
    /// The lines of `app`'s document once UI Automation shows every text line.
    pub fn read(gate: &Gate, app: &Opened) -> Result<Self, String> {
        let t = &gate.ctx.cfg.timing;
        let want = Line::Caret.at();
        let read = || {
            let parts = uia::lines(&gate.uia().text(app.hwnd)?).ok()?;
            let rows = rows(&parts.iter().map(place::rect).collect::<Vec<_>>());
            (rows.len() >= want).then_some(rows)
        };
        let rows = win::poll_until(t.read_wait_ms, t.poll_ms, read)
            .ok_or("Notepad's lines did not show through UI Automation")?;
        Ok(Self {
            rows,
            inset: gate.ctx.cfg.probes.inset_px,
        })
    }

    /// The same lines once their window moved by `dx`, `dy`.
    pub fn shifted(&self, dx: i32, dy: i32) -> Self {
        let rows = (self.rows.iter())
            .map(|r| Rect {
                x: r.x.saturating_add(dx),
                y: r.y.saturating_add(dy),
                ..*r
            })
            .collect();
        Self {
            rows,
            inset: self.inset,
        }
    }

    /// The box of `line`; the empty caret line sits one line under the Arabic one.
    pub fn of(&self, line: Line) -> Rect {
        let at = |l: Line| self.rows.get(l.at()).copied().unwrap_or_default();
        match line {
            Line::Caret => {
                let a = at(Line::Arabic);
                Rect {
                    y: a.y.saturating_add(a.h),
                    ..a
                }
            }
            other => at(other),
        }
    }

    /// A point just past the end of `line`, where a click leaves the caret at its end.
    pub fn end(&self, line: Line) -> POINT {
        let [_, top, right, bottom] = edges(self.of(line));
        POINT {
            x: right.saturating_add(self.inset),
            y: i32::midpoint(top, bottom),
        }
    }

    /// The point at the start of `line`.
    pub fn start(&self, line: Line) -> POINT {
        let [left, top, _, bottom] = edges(self.of(line));
        POINT {
            x: left,
            y: i32::midpoint(top, bottom),
        }
    }

    /// The point inside the caret line, where a click leaves a bare caret.
    pub fn caret(&self) -> POINT {
        let r = self.of(Line::Caret);
        POINT {
            x: r.x.saturating_add(self.inset),
            y: r.y.saturating_add(r.h / 2),
        }
    }
}

/// Selects `line` with a click past its end and Shift+Home, then judges case `name`.
pub fn by_keys(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    line: Line,
    name: &str,
) -> Result<(Value, Option<Shown>), String> {
    let before = Gate::before();
    gate.acts.click(app, lines.end(line))?;
    gate.acts.keys(app, &gate.g().select_home)?;
    gate.case(name, &before, &gate.g().editable_chips)
}

/// Clicks `chip` on `shown` while `app`'s tab is in front: the window in front stays, `done` holds for the clipboard's
/// number before the click, and the strip goes once its tick has shown.
pub fn act(
    gate: &Gate,
    app: &Opened,
    shown: Option<Shown>,
    chip: SelChip,
    done: impl FnOnce(u32) -> bool,
) -> Result<Value, String> {
    let Some(s) = shown else {
        return Ok(json!({ "ok": false, "error": "no strip" }));
    };
    let (focus, seq) = gate.tap(app, &s, chip)?;
    let done = done(seq);
    let went = gate.strip.goes(|_| true)?;
    Ok(json!({ "focus": focus, "done": done, "went": went, "ok": focus && done && went }))
}

/// Copy, Cut and Paste in Notepad, Paste with Shift latched; `cases` gathers each selection's case.
pub fn edits(
    gate: &Gate,
    app: &Opened,
    lines: &Lines,
    cases: &mut Vec<Value>,
) -> Result<Value, String> {
    let g = gate.g();
    let (case, shown) = by_keys(gate, app, lines, Line::Copy, "notepad_keys")?;
    cases.push(case);
    let copy = act(gate, app, shown, SelChip::Copy, |seq| {
        gate.clip_holds(seq, &g.copy_line)
    })?;
    let before = Gate::before();
    gate.acts
        .drag(app, lines.start(Line::Cut), lines.end(Line::Cut))?;
    let (case, shown) = gate.case("notepad_drag", &before, &g.editable_chips)?;
    cases.push(case);
    let cut = act(gate, app, shown, SelChip::Cut, |seq| {
        gate.clip_holds(seq, &g.cut_line)
    })?;
    let (case, shown) = by_keys(gate, app, lines, Line::Paste, "notepad_paste")?;
    cases.push(case);
    let shift = board::latch_id(Latch::Shift)?;
    gate.acts.key(app, &shift)?;
    let paste = act(gate, app, shown, SelChip::Paste, |_| true)?;
    let latched = ModeKey::by_id(shift.clone())?.on()?;
    if latched {
        gate.acts.key(app, &shift)?;
    }
    let max = i32::try_from(gate.ctx.cfg.g1.max_read_chars).unwrap_or(i32::MAX);
    let doc = gate
        .uia()
        .doc_text(app.hwnd, max)
        .is_some_and(|t| lines_of(&t) == edited(g));
    let ok = [&copy, &cut, &paste].iter().all(|v| v["ok"] == true) && doc && latched;
    Ok(
        json!({ "copy": copy, "cut": cut, "paste": paste, "doc": doc, "shift_latched": latched, "ok": ok }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn a_lines_parts_join_into_one_row_and_rows_run_top_to_bottom() {
        let parts = [
            r(10, 40, 50, 20),
            r(10, 10, 80, 20),
            r(60, 41, 30, 19),
            r(10, 70, 0, 20),
        ];
        assert_eq!(
            rows(&parts),
            [r(10, 10, 80, 20), r(10, 40, 80, 20)],
            "an empty part has no row"
        );
        let lines = Lines {
            rows: rows(&parts),
            inset: 4,
        };
        assert_eq!(lines.end(Line::Copy), POINT { x: 94, y: 20 });
        assert_eq!(lines.start(Line::Cut), POINT { x: 10, y: 50 });
        let moved = lines.shifted(5, -7);
        assert_eq!(moved.end(Line::Copy), POINT { x: 99, y: 13 });
        assert_eq!(moved.of(Line::Cut), r(15, 33, 80, 20));
    }
}

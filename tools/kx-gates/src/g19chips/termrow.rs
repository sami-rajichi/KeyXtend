//! A terminal's top text row through UI Automation: from its focused text area while its window is in front, as a
//! terminal's text area is no document, else from a text document of the window.
#![cfg(windows)]

use kx_module_api::ports::Rect;
use kx_platform_windows::uia;
use kx_platform_windows::window::foreground;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::apps::Opened;
use crate::g19chips::gate::Gate;
use crate::g19chips::notepad::usable;
use crate::g19chips::place;
use crate::g19chips::term::Kind;
use crate::win;

/// One look at a text box: its line parts, or none when no box was found.
type Look = Option<Result<Vec<Rect>, String>>;

/// The top text row of the first of `looks` that shows one; else each named look's failed step.
fn first_row(looks: &[(&str, &dyn Fn() -> Look)]) -> Result<Rect, String> {
    let mut why = Vec::new();
    for (name, look) in looks {
        match usable(look(), 1) {
            Ok(rows) => return rows.first().copied().ok_or_else(|| "no row".into()),
            Err(e) => why.push(format!("{name}: {e}")),
        }
    }
    Err(why.join("; "))
}

/// The line parts of text box `el`.
fn parts(el: &IUIAutomationElement) -> Result<Vec<Rect>, String> {
    let boxes = uia::lines(el).map_err(|e| e.to_string())?;
    Ok(boxes.iter().map(place::rect).collect())
}

/// The top text row of terminal `kind`'s window `app`, once UI Automation shows one.
pub fn row(gate: &Gate, kind: Kind, app: &Opened) -> Result<Rect, String> {
    let (t, u) = (&gate.ctx.cfg.timing, gate.uia());
    let front = || win::key(foreground()) == win::key(app.hwnd);
    let focus = || u.focused().ok().filter(|_| front()).map(|f| parts(&f));
    let doc = || u.text(app.hwnd).map(|d| parts(&d));
    let mut last = String::new();
    let read = || {
        first_row(&[("focus", &focus), ("document", &doc)])
            .map_err(|e| last = e)
            .ok()
    };
    let got = win::poll_until(t.read_wait_ms, t.poll_ms, read);
    got.ok_or_else(|| {
        let focused = gate.kind();
        format!(
            "{} shows no text row: {last}; focus on {focused}",
            kind.name()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_look_that_shows_a_row_wins_else_each_says_its_step() {
        let r = |y| Rect {
            x: 0,
            y,
            w: 50,
            h: 20,
        };
        let none = || -> Look { None };
        let empty = || -> Look { Some(Ok(Vec::new())) };
        let two = || -> Look { Some(Ok(vec![r(40), r(10)])) };
        assert_eq!(
            first_row(&[("focus", &two), ("document", &none)]),
            Ok(r(10))
        );
        assert_eq!(
            first_row(&[("focus", &none), ("document", &two)]),
            Ok(r(10))
        );
        let e = first_row(&[("focus", &empty), ("document", &none)]).expect_err("no row");
        assert!(
            e.starts_with("focus: ") && e.contains("; document: "),
            "{e}"
        );
        assert!(e.contains("0 of 1") && e.contains("not found"), "{e}");
    }
}

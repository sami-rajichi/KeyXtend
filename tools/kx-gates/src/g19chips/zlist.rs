//! A z-order snapshot for G19's hung step, in classes and numbers only: each top-level window from the top down to a
//! few rows below our keyboard, and the fields that tell the window guard's suspected causes apart.
#![cfg(windows)]

use serde::Serialize;
use serde_json::{Value, json};

/// Whose window a row is: the product copy's, target-window's, the gate's Notepad's, or another program's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Who {
    /// The product copy's.
    Ours,
    /// target-window's.
    Target,
    /// The gate's Notepad's.
    Notepad,
    /// Another program's.
    Other,
}

/// The processes the rows are told by.
#[derive(Debug, Clone, Copy)]
pub struct Pids {
    /// The product copy's.
    pub ours: u32,
    /// target-window's.
    pub target: u32,
    /// The gate's Notepad's.
    pub notepad: u32,
}

/// One top-level window at its place in the z-order; never its title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Its class name.
    pub class: String,
    /// Whose it is.
    pub who: Who,
    /// The thread that made it.
    pub thread: u32,
    /// Its owner window's place, when it has one.
    pub owner: Option<usize>,
    /// Flagged topmost.
    pub top: bool,
    /// Visible.
    pub shown: bool,
    /// Hidden by the compositor.
    pub cloaked: bool,
    /// Its z-order band, when Windows tells it.
    pub band: Option<u32>,
}

/// What tells the causes apart, read over the whole z-order.
#[derive(Debug, PartialEq, Eq)]
pub struct Verdict {
    /// Above the keyboard and past a topmost window, the first one out of the topmost band, not ours, not Notepad's.
    pub stray: Option<usize>,
    /// A shown window of another program, out of the band, above the keyboard with no topmost window between.
    pub kb_in_normal_run: bool,
}

/// The columns of a listed row, in the order `cells` writes them.
pub const COLS: [&str; 10] = [
    "i", "class", "who", "thread", "owner", "top", "shown", "cloaked", "band", "kb",
];

impl Pids {
    /// Whose a window of process `pid` is.
    pub fn who(&self, pid: u32) -> Who {
        match pid {
            p if p == self.ours => Who::Ours,
            p if p == self.target => Who::Target,
            p if p == self.notepad => Who::Notepad,
            _ => Who::Other,
        }
    }
}

/// True when the row at `i` is the row at `head` or owned by it through its owners; an owner loop ends the walk.
fn in_group(rows: &[Row], i: usize, head: usize) -> bool {
    let owners = std::iter::successors(Some(i), |&at| rows.get(at)?.owner);
    owners
        .take(rows.len().saturating_add(1))
        .any(|at| at == head)
}

/// Above the keyboard at `kb` and past the first topmost row, the first row out of the band that is neither ours nor in
/// the group of Notepad's window at `notepad`.
fn stray(rows: &[Row], kb: usize, notepad: Option<usize>) -> Option<usize> {
    let above = rows.get(..kb)?;
    let first_top = above.iter().position(|r| r.top)?;
    let notepads = |i: usize| notepad.is_some_and(|n| in_group(rows, i, n));
    (first_top..kb).find(|&i| !above[i].top && above[i].who != Who::Ours && !notepads(i))
}

/// True when a shown row of another program, out of the band, sits above the keyboard at `kb` with no topmost row
/// between them.
fn in_normal_run(rows: &[Row], kb: usize) -> bool {
    let above = rows.get(..kb).unwrap_or_default();
    let mut run = above.iter().rev().take_while(|r| !r.top);
    run.any(|r| r.shown && r.who != Who::Ours)
}

/// The verdict over `rows`, with the keyboard at `kb` and Notepad's window at `notepad`.
pub fn verdict(rows: &[Row], kb: usize, notepad: Option<usize>) -> Verdict {
    Verdict {
        stray: stray(rows, kb, notepad),
        kb_in_normal_run: in_normal_run(rows, kb),
    }
}

/// Row `r` at place `i` in `COLS`' order; `kb` marks the keyboard's.
fn cells(i: usize, r: &Row, kb: bool) -> Value {
    json!([
        i, r.class, r.who, r.thread, r.owner, r.top, r.shown, r.cloaked, r.band, kb
    ])
}

/// One snapshot's note: the rows from the top down to `below` past the keyboard at `kb`, and the verdict `v`.
pub fn note(rows: &[Row], kb: usize, below: usize, v: &Verdict) -> Value {
    let end = kb.saturating_add(below).saturating_add(1);
    let listed = rows.iter().enumerate().take(end);
    let listed: Vec<Value> = listed.map(|(i, r)| cells(i, r, i == kb)).collect();
    json!({
        "kb": kb, "total": rows.len(), "stray": v.stray, "kb_in_normal_run": v.kb_in_normal_run,
        "cols": COLS, "rows": listed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row of `who`, topmost or not, shown or hidden, with no owner.
    fn row(class: &str, who: Who, top: bool, shown: bool) -> Row {
        let (thread, owner, cloaked, band) = (1, None, false, None);
        Row {
            class: class.into(),
            who,
            thread,
            owner,
            top,
            shown,
            cloaked,
            band,
        }
    }

    /// Row `r`, owned by the row at `by`.
    fn owned(mut r: Row, by: usize) -> Row {
        r.owner = Some(by);
        r
    }

    #[test]
    fn a_window_is_told_by_its_process() {
        let p = Pids {
            ours: 1,
            target: 2,
            notepad: 3,
        };
        let got = [1, 2, 3, 4].map(|pid| p.who(pid));
        assert_eq!(got, [Who::Ours, Who::Target, Who::Notepad, Who::Other]);
    }

    #[test]
    fn a_hidden_window_out_of_the_band_between_topmost_ones_is_the_stray() {
        let rows = [
            row("Shell_TrayWnd", Who::Other, true, true),
            row("Notepad", Who::Notepad, false, true),
            row("IME", Who::Other, false, false),
            row("tooltips_class32", Who::Other, true, false),
            row("Qt", Who::Ours, true, true),
        ];
        let want = Verdict {
            stray: Some(2),
            kb_in_normal_run: false,
        };
        assert_eq!(
            verdict(&rows, 4, Some(1)),
            want,
            "Notepad landed on the stray"
        );
    }

    #[test]
    fn notepads_own_group_and_ours_are_never_the_stray_but_its_loose_windows_are() {
        let rows = vec![
            row("Shell_TrayWnd", Who::Other, true, true),
            owned(row("PopupHost", Who::Notepad, false, false), 2),
            row("Notepad", Who::Notepad, false, true),
            row("QtShadow", Who::Ours, false, false),
            row("Qt", Who::Ours, true, true),
        ];
        assert_eq!(verdict(&rows, 4, Some(2)).stray, None);
        let mut loose = rows;
        loose[1].owner = None;
        assert_eq!(
            verdict(&loose, 4, Some(2)).stray,
            Some(1),
            "another thread's bridge"
        );
    }

    #[test]
    fn a_window_out_of_the_band_with_no_topmost_one_above_is_no_stray() {
        let rows = [
            row("IME", Who::Other, false, false),
            row("tooltips_class32", Who::Other, true, false),
            row("Qt", Who::Ours, true, true),
        ];
        assert_eq!(verdict(&rows, 2, None).stray, None);
        assert_eq!(verdict(&rows, 0, None).stray, None, "nothing above");
    }

    #[test]
    fn a_shown_window_of_another_program_right_above_the_keyboard_puts_it_in_the_normal_run() {
        let rows = vec![
            row("Shell_TrayWnd", Who::Other, true, true),
            row("Notepad", Who::Notepad, false, true),
            row("IME", Who::Other, false, false),
            row("Qt", Who::Ours, true, true),
        ];
        assert!(verdict(&rows, 3, Some(1)).kb_in_normal_run);
        let mut between = rows.clone();
        between[2].top = true;
        assert!(
            !verdict(&between, 3, Some(1)).kb_in_normal_run,
            "a topmost one between"
        );
        let mut ours = rows;
        ours[1].who = Who::Ours;
        assert!(
            !verdict(&ours, 3, None).kb_in_normal_run,
            "only ours and hidden ones"
        );
    }

    #[test]
    fn an_owner_loop_ends_outside_the_group() {
        let rows = [
            row("Shell_TrayWnd", Who::Other, true, true),
            owned(row("A", Who::Notepad, false, false), 2),
            owned(row("B", Who::Notepad, false, false), 1),
            row("Notepad", Who::Notepad, false, true),
            row("Qt", Who::Ours, true, true),
        ];
        assert!(!in_group(&rows, 1, 3));
        assert_eq!(verdict(&rows, 4, Some(3)).stray, Some(1));
    }

    #[test]
    fn the_note_lists_down_to_below_rows_past_the_keyboard_and_marks_it() {
        let rows: Vec<Row> = (0..6)
            .map(|i| row(&format!("C{i}"), Who::Other, true, true))
            .collect();
        let v = verdict(&rows, 2, None);
        let n = note(&rows, 2, 2, &v);
        let listed = n["rows"].as_array().expect("rows");
        assert_eq!(
            (listed.len(), &n["kb"], &n["total"]),
            (5, &json!(2), &json!(6))
        );
        assert!(
            listed
                .iter()
                .all(|r| r.as_array().is_some_and(|c| c.len() == COLS.len()))
        );
        let kb = COLS.iter().position(|c| *c == "kb").expect("a kb column");
        assert_eq!(listed[2][kb], true);
        assert_eq!(listed[1][kb], false);
        assert_eq!(listed[3][0], 3, "each row's place");
        assert_eq!(
            (&n["stray"], &n["kb_in_normal_run"]),
            (&json!(null), &json!(false))
        );
        assert_eq!(
            note(&rows, 2, 10, &v)["rows"].as_array().map(Vec::len),
            Some(6),
            "no more than there are"
        );
    }
}

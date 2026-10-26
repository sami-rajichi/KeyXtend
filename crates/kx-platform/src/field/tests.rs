//! The shared read rules over a scripted reader: reads come from the focused box, our own focus reads nothing, a late
//! read never starts, and the cut at the read cap and the deadline.

use super::*;
use crate::deadline;
use kx_module_api::Redacted;
use kx_module_api::ports::{FieldKind, FieldRead};
use std::cell::Cell;
use std::sync::mpsc;
use std::time::Duration;

/// A scripted element: whether it is ours, and its box.
type El = (bool, FieldChange);

/// A reader whose focus is scripted, counting what it was asked.
struct Desk {
    focus: Result<El, FieldError>,
    looks: Cell<usize>,
    reads: Cell<usize>,
}

impl Desk {
    /// A reader whose focus is `focus`.
    fn on(focus: Result<El, FieldError>) -> Self {
        Self {
            focus,
            looks: Cell::new(0),
            reads: Cell::new(0),
        }
    }
}

impl Probe for Desk {
    type El = El;

    fn ours(&self, el: &El) -> bool {
        el.0
    }

    fn change(&self, el: &El) -> FieldChange {
        el.1
    }
}

impl Reader for Desk {
    fn focused(&self) -> Result<El, FieldError> {
        self.looks.set(self.looks.get() + 1);
        self.focus
    }

    fn before_cursor(&self, el: &El, max_words: u8, _: Instant) -> Result<FieldRead, FieldError> {
        self.reads.set(self.reads.get() + 1);
        let text = Some(Redacted::new(format!("{max_words} words")));
        Ok(FieldRead {
            kind: el.1.kind,
            text,
        })
    }

    fn ends_with(&self, _: &El, text: &str, _: Instant) -> Result<Option<bool>, FieldError> {
        self.reads.set(self.reads.get() + 1);
        Ok(Some(text == "ap"))
    }
}

/// A text box of another app.
fn theirs() -> El {
    let change = FieldChange {
        kind: FieldKind::Text,
        box_id: 1,
    };
    (false, change)
}

/// A minute from now, which no test reaches.
fn far() -> Instant {
    deadline(Duration::from_secs(60))
}

/// What `serve` answers to two words before the caret, read by `by`.
fn before(desk: &Desk, by: Instant) -> Result<Option<(FieldKind, String)>, FieldError> {
    let (reply, answer) = mpsc::channel();
    let max_words = 2;
    serve(
        desk,
        Job(Ask::Before {
            max_words,
            by,
            reply,
        }),
    );
    let read = answer.recv().unwrap()?;
    Ok(read.map(|r| (r.kind, r.text.map(Redacted::into_inner).unwrap_or_default())))
}

/// What `serve` answers to whether the text ends with `ap`, read by `by`.
fn ends(desk: &Desk, by: Instant) -> Result<Option<bool>, FieldError> {
    let (reply, answer) = mpsc::channel();
    let text = Redacted::new(String::from("ap"));
    serve(desk, Job(Ask::Ends { text, by, reply }));
    answer.recv().unwrap()
}

#[test]
fn a_read_comes_back_from_the_focused_box() {
    let desk = Desk::on(Ok(theirs()));
    assert_eq!(
        before(&desk, far()),
        Ok(Some((FieldKind::Text, "2 words".to_owned())))
    );
    assert_eq!(ends(&desk, far()), Ok(Some(true)));
    assert_eq!(desk.reads.get(), 2);
}

#[test]
fn focus_in_our_own_process_reads_as_none_and_reads_nothing() {
    let desk = Desk::on(Ok((true, theirs().1)));
    assert_eq!(before(&desk, far()), Ok(None));
    assert_eq!(ends(&desk, far()), Ok(None));
    assert_eq!(desk.reads.get(), 0);
}

#[test]
fn a_late_read_times_out_before_it_looks_at_the_focus() {
    let desk = Desk::on(Ok(theirs()));
    let past = Instant::now()
        .checked_sub(Duration::from_millis(1))
        .unwrap();
    assert_eq!(before(&desk, past), Err(FieldError::TimedOut));
    assert_eq!(ends(&desk, past), Err(FieldError::TimedOut));
    assert_eq!((desk.looks.get(), desk.reads.get()), (0, 0));
}

#[test]
fn a_refused_focus_lookup_passes_its_error_on() {
    let desk = Desk::on(Err(FieldError::Refused));
    assert_eq!(before(&desk, far()), Err(FieldError::Refused));
    assert_eq!(desk.reads.get(), 0);
}

#[test]
fn the_deadline_is_later_and_never_overflows() {
    assert_eq!(late(far()), Ok(()));
    assert!(deadline(Duration::MAX) <= Instant::now());
}

#[test]
fn the_cut_at_the_cap_drops_the_first_word_it_may_have_split() {
    assert_eq!(after_first("ord two three"), " two three");
    assert_eq!(
        after_first(" two three"),
        " two three",
        "a cut between words keeps all"
    );
    assert_eq!(after_first("word"), "");
    assert_eq!(after_first("لمة كلمتان"), " كلمتان");
}

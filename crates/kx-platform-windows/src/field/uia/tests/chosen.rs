//! Which range a read and a check use: `TextPattern2`'s caret first, a selection that holds text over it, and an
//! unsure caret when `GetSelection` fails or is missing; and whether a lone range holds text when its end check fails.

use std::cell::Cell;

use super::super::caret::{Caret, Held, Sel, chosen, held_by};
use kx_module_api::ports::FieldError;

/// What `GetSelection` told in a test: a range named by a word.
type Told = Result<Sel<&'static str>, FieldError>;

/// True when the caret `chosen` takes from `caret` and `sel` is `range`, with its selection `held`.
fn takes(caret: Option<&'static str>, sel: Told, range: &'static str, held: Held) -> bool {
    chosen(caret, sel) == Ok(Some(Caret { range, held }))
}

#[test]
fn the_text_pattern_2_caret_comes_first_and_a_selection_that_holds_text_wins() {
    let caret = Some("caret");
    assert!(takes(
        caret,
        Ok(Sel::Range("sel", Held::No)),
        "caret",
        Held::No
    ));
    assert!(takes(caret, Ok(Sel::Empty), "caret", Held::No));
    assert!(takes(
        caret,
        Ok(Sel::Range("sel", Held::Yes)),
        "sel",
        Held::Yes
    ));
    assert!(takes(
        None,
        Ok(Sel::Range("sel", Held::Yes)),
        "sel",
        Held::Yes
    ));
    assert!(takes(
        None,
        Ok(Sel::Range("sel", Held::No)),
        "sel",
        Held::No
    ));
}

#[test]
fn a_failed_or_missing_selection_keeps_the_caret_but_leaves_it_unsure() {
    let failed = [Err(FieldError::Refused), Err(FieldError::TimedOut)];
    for sel in failed.into_iter().chain([Ok(Sel::Missing)]) {
        assert!(takes(Some("caret"), sel, "caret", Held::Unknown));
    }
    let refused = chosen::<&str>(None, Err(FieldError::Refused));
    assert_eq!(refused, Err(FieldError::Refused), "no caret to keep");
    assert_eq!(chosen::<&str>(None, Ok(Sel::Missing)), Ok(None));
    assert_eq!(chosen::<&str>(None, Ok(Sel::Empty)), Ok(None));
}

#[test]
fn an_unsure_selection_range_stays_unsure_with_or_without_a_caret() {
    let unsure = || Ok(Sel::Range("sel", Held::Unknown));
    assert!(takes(None, unsure(), "sel", Held::Unknown));
    assert!(takes(Some("caret"), unsure(), "caret", Held::Unknown));
}

#[test]
fn a_working_end_check_tells_the_selection_and_never_asks_for_boxes() {
    let asked = Cell::new(0);
    let boxes = || {
        asked.set(asked.get() + 1);
        Some(1)
    };
    assert_eq!(held_by(Ok(-120), boxes), Ok(Held::Yes));
    assert_eq!(held_by(Ok(3), boxes), Ok(Held::Yes));
    assert_eq!(held_by(Ok(0), boxes), Ok(Held::No));
    assert_eq!(asked.get(), 0, "a working check asks for no box");
}

#[test]
fn a_refused_end_check_takes_the_line_boxes_and_a_timeout_stays_an_error() {
    let refused = || Err(FieldError::Refused);
    let terminal = held_by(refused(), || Some(1));
    assert_eq!(terminal, Ok(Held::Yes), "Windows Terminal with a selection");
    assert_eq!(
        held_by(refused(), || Some(0)),
        Ok(Held::Unknown),
        "no box: unsure"
    );
    assert_eq!(
        held_by(refused(), || None),
        Err(FieldError::Refused),
        "boxes unread"
    );
    let asked = Cell::new(0);
    let boxes = || {
        asked.set(asked.get() + 1);
        Some(1)
    };
    for e in [FieldError::TimedOut, FieldError::Unavailable] {
        assert_eq!(held_by(Err(e), boxes), Err(e));
    }
    assert_eq!(asked.get(), 0, "a timeout asks for no box");
}

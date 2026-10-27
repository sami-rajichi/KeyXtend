//! Which range a read and a check use: `TextPattern2`'s caret first, a selection that holds text over it, and an
//! unsure caret when `GetSelection` fails or is missing.

use super::super::caret::{Caret, Held, Sel, chosen};
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
        Ok(Sel::Range("sel", false)),
        "caret",
        Held::No
    ));
    assert!(takes(caret, Ok(Sel::Empty), "caret", Held::No));
    assert!(takes(caret, Ok(Sel::Range("sel", true)), "sel", Held::Yes));
    assert!(takes(None, Ok(Sel::Range("sel", true)), "sel", Held::Yes));
    assert!(takes(None, Ok(Sel::Range("sel", false)), "sel", Held::No));
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

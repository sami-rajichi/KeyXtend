//! The shared kind order: a password box first, unknown for a missing fact or a hiding window, no box without text,
//! and a read-only box unknown, probed only when nothing else decided.

use super::*;
use std::cell::Cell;

/// A text box every fact of which was read.
fn text_box() -> BoxFacts {
    BoxFacts {
        password: Some(false),
        known: true,
        hidden: false,
        text: true,
        read_only: false,
    }
}

/// The kind of `facts` with a probe that answers `fixed`, and whether it was asked.
fn kind_of(facts: BoxFacts, fixed: bool) -> (FieldKind, bool) {
    let asked = Cell::new(false);
    let got = kind(facts, || {
        asked.set(true);
        fixed
    });
    (got, asked.get())
}

#[test]
fn a_password_box_wins_over_every_other_fact() {
    let facts = BoxFacts {
        password: Some(true),
        known: false,
        hidden: true,
        text: false,
        read_only: true,
    };
    assert_eq!(kind_of(facts, true), (FieldKind::Password, false));
}

#[test]
fn a_missing_fact_or_a_hiding_window_is_unknown_unprobed() {
    let unread = BoxFacts {
        password: None,
        ..text_box()
    };
    let unknown = BoxFacts {
        known: false,
        ..text_box()
    };
    let hidden = BoxFacts {
        hidden: true,
        ..text_box()
    };
    for facts in [unread, unknown, hidden] {
        assert_eq!(
            kind_of(facts, false),
            (FieldKind::Unknown, false),
            "{facts:?}"
        );
    }
}

#[test]
fn no_text_is_no_box_and_read_only_text_is_unknown() {
    let plain = BoxFacts {
        text: false,
        ..text_box()
    };
    assert_eq!(kind_of(plain, false), (FieldKind::None, false));
    let fixed = BoxFacts {
        read_only: true,
        ..text_box()
    };
    assert_eq!(kind_of(fixed, false), (FieldKind::Unknown, false));
    assert_eq!(
        kind_of(text_box(), true),
        (FieldKind::Unknown, true),
        "the probe says read-only"
    );
    assert_eq!(kind_of(text_box(), false), (FieldKind::Text, true));
}

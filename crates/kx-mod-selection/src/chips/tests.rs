//! The keycaps per kind and per setting, at a selection and at a caret.

use super::*;
use SelChip::{Copy, Cut, Paste, Translate};

/// Every switch on.
const ALL_ON: Rules = Rules {
    on: true,
    copy: true,
    cut: true,
    paste: true,
    translate: true,
    caret_paste: true,
};

/// A caret with something on the clipboard.
const CARET: At = At::Caret { clip: true };

#[test]
fn each_kind_shows_its_keycaps_on_a_selection() {
    let at = |kind| chips(&ALL_ON, kind, At::Selection);
    assert_eq!(at(SelKind::Editable), [Copy, Cut, Paste, Translate]);
    assert_eq!(at(SelKind::ReadOnly), [Copy, Translate]);
    assert_eq!(at(SelKind::Terminal), [Copy, Paste, Translate], "never Cut");
    assert!(at(SelKind::Closed).is_empty());
}

#[test]
fn each_keycap_needs_its_own_setting() {
    let off = |rules: Rules| chips(&rules, SelKind::Editable, At::Selection);
    assert_eq!(
        off(Rules {
            copy: false,
            ..ALL_ON
        }),
        [Cut, Paste, Translate]
    );
    assert_eq!(
        off(Rules {
            cut: false,
            ..ALL_ON
        }),
        [Copy, Paste, Translate]
    );
    assert_eq!(
        off(Rules {
            paste: false,
            ..ALL_ON
        }),
        [Copy, Cut, Translate]
    );
    assert_eq!(
        off(Rules {
            translate: false,
            ..ALL_ON
        }),
        [Copy, Cut, Paste]
    );
    let term = Rules {
        paste: false,
        ..ALL_ON
    };
    assert_eq!(
        chips(&term, SelKind::Terminal, At::Selection),
        [Copy, Translate]
    );
}

#[test]
fn the_helper_off_shows_nothing_anywhere() {
    let off = Rules {
        on: false,
        ..ALL_ON
    };
    for kind in SelKind::ALL {
        assert!(chips(&off, kind, At::Selection).is_empty(), "{kind:?}");
        assert!(chips(&off, kind, CARET).is_empty(), "{kind:?}");
    }
}

#[test]
fn a_caret_shows_paste_alone_in_an_editable_box_with_a_clipboard() {
    assert_eq!(chips(&ALL_ON, SelKind::Editable, CARET), [Paste]);
    let empty = At::Caret { clip: false };
    assert!(
        chips(&ALL_ON, SelKind::Editable, empty).is_empty(),
        "nothing to paste"
    );
    for kind in [SelKind::ReadOnly, SelKind::Terminal, SelKind::Closed] {
        assert!(chips(&ALL_ON, kind, CARET).is_empty(), "{kind:?}");
    }
}

#[test]
fn the_caret_paste_needs_both_its_switch_and_paste() {
    let no_caret = Rules {
        caret_paste: false,
        ..ALL_ON
    };
    assert!(chips(&no_caret, SelKind::Editable, CARET).is_empty());
    let no_paste = Rules {
        paste: false,
        ..ALL_ON
    };
    assert!(chips(&no_paste, SelKind::Editable, CARET).is_empty());
    let selection = chips(&no_caret, SelKind::Editable, At::Selection);
    assert!(selection.contains(&Paste), "a selection keeps its Paste");
}

#[test]
fn the_keycaps_keep_the_strips_order() {
    for kind in SelKind::ALL {
        let shown = chips(&ALL_ON, kind, At::Selection);
        let order: Vec<_> = SelChip::ALL
            .into_iter()
            .filter(|c| shown.contains(c))
            .collect();
        assert_eq!(shown, order, "{kind:?}");
    }
}

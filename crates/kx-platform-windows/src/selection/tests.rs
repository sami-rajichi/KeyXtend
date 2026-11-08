//! The kind table and the kind rules: the shipped table and bad ones, and the boxes the chips meet (Notepad's edit, a
//! read-only document, Windows Terminal, the console, a password box, our own window); the walk up to the text is in
//! `climb`, the boxes in `spot`, the rules over any facts in `props`, the handlers in `hear`, where the selection
//! handler sits in `follow`, and the readers on hidden edit boxes in `uia`.

use super::kinds::{Facts, kind};
use super::*;
use crate::field;
use kx_module_api::ports::SelKind;
use std::cell::Cell;

mod climb;
mod follow;
#[cfg(windows)]
mod hear;
mod props;
mod spot;
#[cfg(windows)]
mod uia;

/// UI Automation's Edit, Document, Text and Button control types.
const EDIT: i32 = 50004;
const DOCUMENT: i32 = 50030;
const TEXT: i32 = 50020;
const BUTTON: i32 = 50000;
/// The edit password style bit.
const PW: u32 = 0x0020;

/// A small table with one entry of each list.
const SMALL: &str = r#"
edit_controls = [50004, 50030]
read_controls = [50020]
terminal_classes = ["CASCADIA_HOSTING_WINDOW_CLASS", "ConsoleWindowClass"]
walk_up = 15
"#;

/// The small table, parsed.
fn small() -> Table {
    Table::parse(SMALL).unwrap()
}

/// What UI Automation tells about a writable box of `control` in another app.
fn plain(control: i32) -> Facts {
    Facts {
        control: Some(control),
        password: Some(false),
        class: String::from("RichEditD2DPT"),
        style: 0,
        top: String::from("Notepad"),
        fixed: Some(false),
        ours: false,
    }
}

/// A page in Chrome: a document whose value pattern says read-only.
fn page() -> Facts {
    Facts {
        class: String::from("Chrome_RenderWidgetHostHWND"),
        top: String::from("Chrome_WidgetWin_1"),
        fixed: Some(true),
        ..plain(DOCUMENT)
    }
}

/// A terminal: Windows Terminal's text has no window of its own, and the console's is its window.
fn terminal(class: &str, top: &str) -> Facts {
    Facts {
        class: class.to_owned(),
        top: top.to_owned(),
        fixed: None,
        ..plain(TEXT)
    }
}

/// The kind of `facts` under the shipped field table and `table`, with a range that says `range`, and how often the
/// range was asked.
fn asked(facts: &Facts, table: &Table, range: Option<bool>) -> (SelKind, usize) {
    let asks = Cell::new(0);
    let got = kind(facts, field::table(), table, || {
        asks.set(asks.get() + 1);
        range
    });
    (got, asks.get())
}

/// The kind of `facts` under the small table, with a range that cannot tell.
fn kind_of(facts: &Facts) -> SelKind {
    asked(facts, &small(), None).0
}

#[test]
fn the_shipped_table_parses_with_every_list_filled() {
    let shipped = Table::parse(TABLE).unwrap();
    assert!(shipped.edits(EDIT) && shipped.edits(DOCUMENT) && shipped.reads(TEXT));
    assert!(
        shipped.terminal("CASCADIA_HOSTING_WINDOW_CLASS") && shipped.terminal("consolewindowclass")
    );
    assert!(!shipped.terminal("") && !shipped.edits(BUTTON));
    assert!(table().edits(EDIT), "the shared copy is the shipped one");
}

#[test]
fn a_table_that_does_not_parse_closes_every_box() {
    assert!(!shipped(TABLE).is_closed());
    let closed = shipped("not a table");
    assert!(closed.is_closed());
    let boxes = [
        plain(EDIT),
        plain(DOCUMENT),
        page(),
        terminal("", "ConsoleWindowClass"),
    ];
    for facts in &boxes {
        assert_eq!(
            asked(facts, &closed, Some(false)),
            (SelKind::Closed, 0),
            "{facts:?}"
        );
        let blind = kind(facts, &field::Table::closed(), &small(), || Some(false));
        assert_eq!(blind, SelKind::Closed, "a closed field table: {facts:?}");
    }
}

#[test]
fn a_bad_table_is_an_error() {
    let unknown = format!("{SMALL}\nextra = 1");
    assert!(matches!(Table::parse(&unknown), Err(TableError::Parse(_))));
    let wrong = SMALL.replace("[50020]", "\"text\"");
    assert!(matches!(Table::parse(&wrong), Err(TableError::Parse(_))));
    let missing = SMALL.replace("read_controls = [50020]", "");
    assert!(matches!(Table::parse(&missing), Err(TableError::Parse(_))));
    for empty in [
        SMALL.replace("[50004, 50030]", "[]"),
        SMALL.replace("[50020]", "[]"),
        SMALL.replace("\"ConsoleWindowClass\"", "\" \""),
        SMALL.replace(
            "[\"CASCADIA_HOSTING_WINDOW_CLASS\", \"ConsoleWindowClass\"]",
            "[]",
        ),
    ] {
        assert_eq!(
            Table::parse(&empty).err(),
            Some(TableError::Empty),
            "{empty}"
        );
    }
    let twice = SMALL.replace("[50020]", "[50020, 50030]");
    assert_eq!(Table::parse(&twice).err(), Some(TableError::Twice));
}

#[test]
fn notepads_edit_is_editable_old_and_new() {
    let old = Facts {
        class: String::from("Edit"),
        ..plain(EDIT)
    };
    assert_eq!(kind_of(&old), SelKind::Editable);
    assert_eq!(kind_of(&plain(DOCUMENT)), SelKind::Editable);
}

#[test]
fn a_read_only_document_or_label_is_read_only() {
    assert_eq!(kind_of(&page()), SelKind::ReadOnly);
    let unsaid = Facts {
        fixed: None,
        ..page()
    };
    assert_eq!(asked(&unsaid, &small(), Some(true)), (SelKind::ReadOnly, 1));
    let label = Facts {
        fixed: None,
        ..plain(TEXT)
    };
    assert_eq!(asked(&label, &small(), Some(false)), (SelKind::ReadOnly, 0));
}

#[test]
fn a_writable_range_makes_an_edit_box_editable_whatever_its_value_says() {
    assert_eq!(
        asked(&page(), &small(), Some(false)),
        (SelKind::Editable, 1)
    );
    assert_eq!(
        asked(&plain(EDIT), &small(), Some(true)),
        (SelKind::Editable, 0),
        "the value said writable"
    );
}

#[test]
fn windows_terminal_and_the_console_are_terminals() {
    let wt = terminal("", "CASCADIA_HOSTING_WINDOW_CLASS");
    let console = terminal("ConsoleWindowClass", "ConsoleWindowClass");
    for facts in [wt, console] {
        assert_eq!(
            asked(&facts, &small(), Some(false)),
            (SelKind::Terminal, 0),
            "{facts:?}"
        );
        let any = Facts {
            control: Some(BUTTON),
            ..facts.clone()
        };
        assert_eq!(
            kind_of(&any),
            SelKind::Terminal,
            "any control in a terminal"
        );
    }
}

#[test]
fn a_password_box_is_closed_by_its_flag_or_its_edit_style() {
    let flagged = Facts {
        password: Some(true),
        ..plain(EDIT)
    };
    let styled = Facts {
        class: String::from("Edit"),
        style: PW,
        ..plain(EDIT)
    };
    let unread = Facts {
        password: None,
        ..plain(EDIT)
    };
    let console = Facts {
        password: Some(true),
        ..terminal("ConsoleWindowClass", "ConsoleWindowClass")
    };
    for facts in [flagged, styled, unread, console] {
        assert_eq!(
            asked(&facts, &small(), Some(false)),
            (SelKind::Closed, 0),
            "{facts:?}"
        );
    }
}

#[test]
fn our_own_window_is_closed() {
    let own = Facts {
        ours: true,
        ..plain(EDIT)
    };
    assert_eq!(asked(&own, &small(), Some(false)), (SelKind::Closed, 0));
}

#[test]
fn a_box_of_unknown_kind_is_closed() {
    let unread = Facts {
        control: None,
        ..plain(EDIT)
    };
    let remote = Facts {
        top: String::from("TscShellContainerClass"),
        ..plain(DOCUMENT)
    };
    for facts in [unread, plain(BUTTON), remote, Facts::default()] {
        assert_eq!(
            asked(&facts, &small(), Some(false)),
            (SelKind::Closed, 0),
            "{facts:?}"
        );
    }
}

#[cfg(windows)]
#[test]
fn the_table_numbers_are_windows_own() {
    use windows::Win32::UI::Accessibility::{
        UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_TextControlTypeId,
    };
    use windows::Win32::UI::WindowsAndMessaging::ES_PASSWORD;
    let edits = [UIA_EditControlTypeId.0, UIA_DocumentControlTypeId.0];
    let reads = [UIA_TextControlTypeId.0];
    assert_eq!(table().controls(), (&edits[..], &reads[..]));
    assert_eq!([EDIT, DOCUMENT, TEXT], [edits[0], edits[1], reads[0]]);
    assert_eq!(i64::from(PW), i64::from(ES_PASSWORD));
}

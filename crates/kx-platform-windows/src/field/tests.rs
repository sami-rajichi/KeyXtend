//! The kind table and the kind rules: each rule, the shipped table, bad tables, and the cut at the read cap.

use super::*;
use kx_module_api::ports::FieldKind;
use proptest::prelude::*;

/// UI Automation's Edit, Document and Button control types.
const EDIT: i32 = 50004;
const DOCUMENT: i32 = 50030;
const BUTTON: i32 = 50000;
/// The edit password style bit.
const PW: u32 = 0x0020;

/// A small table with one class of each kind.
const SMALL: &str = r#"
text_controls = [50004, 50030]
password_style = 0x0020
edit_classes = ["Edit", "RichEdit60W"]
opaque_classes = ["TscShellContainerClass", "ConsoleWindowClass"]
"#;

/// The small table, parsed.
fn small() -> Table {
    Table::parse(SMALL).unwrap()
}

/// What UI Automation tells about a plain element of `control` in an ordinary app.
fn plain(control: i32) -> Facts {
    Facts {
        control: Some(control),
        password: Some(false),
        class: String::from("Chrome_RenderWidgetHostHWND"),
        style: 0,
        top: String::from("Chrome_WidgetWin_1"),
        editable: false,
        read_only: false,
    }
}

/// The kind of `facts` under the small table.
fn kind_of(facts: &Facts) -> FieldKind {
    kind(facts, &small())
}

#[test]
fn the_shipped_table_parses_with_every_list_filled() {
    let shipped = Table::parse(TABLE).unwrap();
    assert!(shipped.hides("Edit", PW) && shipped.hides("RichEdit50W", PW));
    assert!(
        shipped.opaque("TscShellContainerClass") && shipped.opaque("CASCADIA_HOSTING_WINDOW_CLASS")
    );
    assert!(shipped.holds_text(EDIT) && shipped.holds_text(DOCUMENT));
    assert!(
        table().holds_text(EDIT),
        "the shared copy is the shipped one"
    );
}

#[test]
fn a_table_that_does_not_parse_makes_every_box_unknown_but_a_password_box() {
    assert!(!shipped(TABLE).is_closed());
    let closed = shipped("not a table");
    assert!(closed.is_closed());
    let custom = Facts {
        editable: true,
        ..plain(BUTTON + 1)
    };
    for facts in [plain(EDIT), plain(DOCUMENT), plain(BUTTON), custom] {
        assert_eq!(kind(&facts, &closed), FieldKind::Unknown, "{facts:?}");
    }
    let secret = Facts {
        password: Some(true),
        ..plain(EDIT)
    };
    assert_eq!(kind(&secret, &closed), FieldKind::Password);
}

#[test]
fn a_bad_table_is_an_error() {
    let unknown = format!("{SMALL}\nextra = 1");
    assert!(matches!(Table::parse(&unknown), Err(TableError::Parse(_))));
    let wrong = SMALL.replace("0x0020", "\"bit\"");
    assert!(matches!(Table::parse(&wrong), Err(TableError::Parse(_))));
    let missing = SMALL.replace("password_style = 0x0020", "");
    assert!(matches!(Table::parse(&missing), Err(TableError::Parse(_))));
    for empty in [
        SMALL.replace("0x0020", "0"),
        SMALL.replace("[50004, 50030]", "[]"),
        SMALL.replace("\"Edit\", ", "\" \", "),
        SMALL.replace("[\"TscShellContainerClass\", \"ConsoleWindowClass\"]", "[]"),
    ] {
        assert_eq!(
            Table::parse(&empty).err(),
            Some(TableError::Empty),
            "{empty}"
        );
    }
}

#[test]
fn uia_saying_password_makes_a_password_box() {
    let facts = Facts {
        password: Some(true),
        ..plain(EDIT)
    };
    assert_eq!(kind_of(&facts), FieldKind::Password);
}

#[test]
fn an_edit_class_with_the_password_style_is_a_password_box_even_when_uia_says_no() {
    for class in ["Edit", "richedit60w"] {
        let facts = Facts {
            class: class.to_owned(),
            style: PW,
            ..plain(EDIT)
        };
        assert_eq!(kind_of(&facts), FieldKind::Password, "{class}");
    }
    let opaque = Facts {
        class: String::from("Edit"),
        style: PW,
        top: String::from("ConsoleWindowClass"),
        password: None,
        ..plain(EDIT)
    };
    assert_eq!(kind_of(&opaque), FieldKind::Password, "the style wins");
}

#[test]
fn the_password_style_means_nothing_outside_an_edit_class() {
    let facts = Facts {
        style: PW,
        ..plain(EDIT)
    };
    assert_eq!(kind_of(&facts), FieldKind::Text);
}

#[test]
fn an_element_whose_properties_cannot_be_read_is_unknown() {
    let no_type = Facts {
        control: None,
        ..plain(EDIT)
    };
    let no_flag = Facts {
        password: None,
        ..plain(EDIT)
    };
    assert_eq!(kind_of(&no_type), FieldKind::Unknown);
    assert_eq!(kind_of(&no_flag), FieldKind::Unknown);
    assert_eq!(kind_of(&Facts::default()), FieldKind::Unknown);
}

#[test]
fn an_opaque_class_from_the_table_is_unknown_on_the_box_or_its_window() {
    let own = Facts {
        class: String::from("tscshellcontainerclass"),
        ..plain(DOCUMENT)
    };
    let top = Facts {
        top: String::from("ConsoleWindowClass"),
        editable: true,
        ..plain(DOCUMENT)
    };
    assert_eq!(kind_of(&own), FieldKind::Unknown);
    assert_eq!(kind_of(&top), FieldKind::Unknown);
}

#[test]
fn a_read_only_box_is_unknown() {
    for control in [EDIT, DOCUMENT] {
        let facts = Facts {
            read_only: true,
            ..plain(control)
        };
        assert_eq!(kind_of(&facts), FieldKind::Unknown, "{control}");
    }
}

#[test]
fn edit_and_document_types_and_editable_patterns_are_text() {
    assert_eq!(kind_of(&plain(EDIT)), FieldKind::Text);
    assert_eq!(kind_of(&plain(DOCUMENT)), FieldKind::Text);
    let custom = Facts {
        editable: true,
        ..plain(BUTTON + 1)
    };
    assert_eq!(kind_of(&custom), FieldKind::Text);
}

#[test]
fn a_button_or_the_desktop_is_no_box() {
    assert_eq!(kind_of(&plain(BUTTON)), FieldKind::None);
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

#[cfg(windows)]
#[test]
fn the_table_numbers_are_windows_own() {
    use windows::Win32::UI::Accessibility::{UIA_DocumentControlTypeId, UIA_EditControlTypeId};
    use windows::Win32::UI::WindowsAndMessaging::ES_PASSWORD;
    let table = table();
    assert_eq!(i64::from(table.password_style()), i64::from(ES_PASSWORD));
    let types = [UIA_EditControlTypeId.0, UIA_DocumentControlTypeId.0];
    assert_eq!(table.text_controls(), types);
    assert_eq!([EDIT, DOCUMENT], types);
}

/// Any facts: a few control types, both flags and unreadable ones, and classes from every list.
fn any_facts() -> impl Strategy<Value = Facts> {
    let class = prop::sample::select(vec!["", "Edit", "Static", "ConsoleWindowClass"]);
    (
        prop::option::of(prop::sample::select(vec![EDIT, DOCUMENT, BUTTON])),
        prop::option::of(any::<bool>()),
        (class.clone(), any::<u32>(), class),
        (any::<bool>(), any::<bool>()),
    )
        .prop_map(
            |(control, password, (class, style, top), (editable, read_only))| Facts {
                control,
                password,
                class: class.to_owned(),
                style,
                top: top.to_owned(),
                editable,
                read_only,
            },
        )
}

proptest! {
    #[test]
    fn a_box_is_text_only_when_every_fact_was_read_and_says_so(facts in any_facts()) {
        let table = small();
        let got = kind(&facts, &table);
        if facts.password == Some(true) {
            prop_assert_eq!(got, FieldKind::Password);
        }
        if got == FieldKind::Text {
            prop_assert!(facts.control.is_some() && facts.password == Some(false));
            prop_assert!(!table.hides(&facts.class, facts.style));
            prop_assert!(!table.opaque(&facts.class) && !table.opaque(&facts.top));
            prop_assert!(!facts.read_only);
        }
    }
}

//! The kind rules over any facts: text only when every fact says so, and read-only only narrowing a text box.

use super::*;
use proptest::prelude::*;

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
        let got = kind(&facts, &table, || false);
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

    #[test]
    fn read_only_only_turns_text_into_unknown_and_is_probed_only_there(facts in any_facts()) {
        let open = Facts { read_only: false, ..facts.clone() };
        let fixed = Facts { read_only: true, ..facts };
        let (base, asked) = probed(&open, false);
        let narrowed = [probed(&fixed, false), probed(&fixed, true), probed(&open, true)];
        prop_assert_eq!(asked, usize::from(base == FieldKind::Text));
        prop_assert_eq!(narrowed[0].1 + narrowed[1].1, 0, "a read-only value is never probed");
        let want = if base == FieldKind::Text { FieldKind::Unknown } else { base };
        for (got, _) in narrowed {
            prop_assert_eq!(got, want);
        }
    }
}

//! The rules over any input: a box is never Editable unless every fact allows it and never shown when closed; a line
//! box holds its part and every part on its line, and never panics on any numbers; a read cut by a cap never makes a
//! character that was not there.

use super::super::spot::{line, whole};
use super::*;
use crate::screen::edges;
use kx_module_api::ports::Rect;
use proptest::prelude::*;

/// Any facts: a few control types, both flags and unread ones, and classes from every list.
fn any_facts() -> impl Strategy<Value = Facts> {
    let class = prop::sample::select(vec![
        "",
        "Edit",
        "RichEditD2DPT",
        "ConsoleWindowClass",
        "CASCADIA_HOSTING_WINDOW_CLASS",
        "TscShellContainerClass",
    ]);
    let flag = || prop::option::of(any::<bool>());
    (
        prop::option::of(prop::sample::select(vec![EDIT, DOCUMENT, TEXT, BUTTON])),
        (flag(), flag()),
        (class.clone(), prop::sample::select(vec![0, PW]), class),
        any::<bool>(),
    )
        .prop_map(
            |(control, (password, fixed), (class, style, top), ours)| Facts {
                control,
                password,
                class: class.to_owned(),
                style,
                top: top.to_owned(),
                fixed,
                ours,
            },
        )
}

/// A box within a few screens, at least a pixel each way.
fn any_rect() -> impl Strategy<Value = Rect> {
    let at = -100_000..100_000;
    let size = 1..10_000;
    (at.clone(), at, size.clone(), size).prop_map(|(x, y, w, h)| Rect { x, y, w, h })
}

/// Whether `outer` holds all of `inner`.
fn holds(outer: Rect, inner: Rect) -> bool {
    let ([ol, ot, or, ob], [il, it, ir, ib]) = (edges(outer), edges(inner));
    ol <= il && ot <= it && or >= ir && ob >= ib
}

proptest! {
    #[test]
    fn a_box_is_open_only_when_every_fact_allows_it(facts in any_facts(), range in prop::option::of(any::<bool>())) {
        let (got, asks) = asked(&facts, &small(), range);
        let boxes = field::table();
        if facts.ours || facts.password != Some(false) || facts.control.is_none() || boxes.hides(&facts.class, facts.style) {
            prop_assert_eq!(got, SelKind::Closed);
        }
        if got == SelKind::Editable {
            prop_assert!(facts.fixed == Some(false) || range == Some(false));
            prop_assert!(facts.control.is_some_and(|c| small().edits(c)));
        }
        prop_assert!(asks <= 1);
        prop_assert!(asks == 0 || facts.fixed != Some(false), "a writable value is never probed");
    }

    #[test]
    fn a_line_box_holds_its_part_and_every_part_on_its_line(rects in prop::collection::vec(any_rect(), 1..8), at in 0..8usize) {
        let at = at % rects.len();
        let got = line(&rects, at).unwrap();
        prop_assert!(holds(got, rects[at]));
        let [_, top, _, bottom] = edges(rects[at]);
        for part in &rects {
            let middle = part.y.saturating_add(part.h / 2);
            if (top..bottom).contains(&middle) {
                prop_assert!(holds(got, *part));
            }
        }
    }

    #[test]
    fn a_line_box_never_panics_on_any_numbers(rects in prop::collection::vec(any::<(i32, i32, i32, i32)>(), 0..6), at in 0..8usize) {
        let rects: Vec<Rect> = rects.into_iter().map(|(x, y, w, h)| Rect { x, y, w, h }).collect();
        let got = line(&rects, at);
        prop_assert_eq!(got.is_some(), at < rects.len());
    }

    #[test]
    fn a_read_cut_by_a_cap_never_makes_a_character_that_was_not_there(text in any::<String>(), cut in any::<usize>(), max in any::<u16>()) {
        let units: Vec<u16> = text.encode_utf16().collect();
        let cut = cut % (units.len() + 1);
        let got = whole(&units[..cut], max);
        prop_assert!(text.starts_with(&got));
        prop_assert!(got.chars().count() <= usize::from(max));
    }
}

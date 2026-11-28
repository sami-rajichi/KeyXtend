//! `[snip] editor` and `editor_full`: each empty or a whole rectangle with a width and a height, and a maximised
//! editor's work area kept only over a box to come back to.

use super::*;

/// The work area a maximised editor filled.
const WORK: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};

/// The defaults with the editor at `PLACED`, maximised over `full`.
fn maximised(full: Rect) -> toml::Table {
    let mut t = with("editor", place(PLACED));
    t.insert("editor_full".into(), place(full).into());
    t
}

#[test]
fn the_editor_is_empty_until_placed_then_its_rectangle() {
    assert_eq!(read(&with("editor", place(PLACED))).editor, Some(PLACED));
    assert_eq!(read(&with("editor", toml::Table::new())).editor, None);
}

#[test]
fn part_of_a_rectangle_or_another_key_does_not_read() {
    let mut part = place(PLACED);
    part.remove("h");
    let mut more = place(PLACED);
    more.insert("z".into(), 1.into());
    for key in ["editor", "editor_full"] {
        for bad in [&part, &more] {
            assert!(unread(&with(key, bad.clone())), "{key}: {bad:?}");
        }
        assert!(unread(&with(key, 5)), "{key}");
    }
}

#[test]
fn an_editor_with_no_width_or_height_is_refused() {
    let flat = Rect { h: 0, ..PLACED };
    let thin = Rect { w: -1, ..PLACED };
    for bad in [flat, thin] {
        assert!(refused(&with("editor", place(bad)), "editor"), "{bad:?}");
    }
}

#[test]
fn the_editor_is_not_maximised_by_default() {
    let s = SnipSettings::from_defaults().unwrap();
    assert_eq!((s.editor, s.editor_full), (None, None));
}

#[test]
fn a_maximised_editor_keeps_its_work_area_over_its_box() {
    let s = read(&maximised(WORK));
    assert_eq!((s.editor, s.editor_full), (Some(PLACED), Some(WORK)));
}

#[test]
fn a_maximised_editor_with_no_box_under_it_is_refused() {
    assert!(refused(&with("editor_full", place(WORK)), "editor_full"));
}

#[test]
fn a_work_area_with_no_width_or_height_is_refused() {
    let flat = Rect { h: 0, ..WORK };
    let thin = Rect { w: -1, ..WORK };
    for bad in [flat, thin] {
        assert!(refused(&maximised(bad), "editor_full"), "{bad:?}");
    }
}

#[test]
fn a_maximised_editor_writes_back_as_it_reads() {
    let mut s = SnipSettings::from_defaults().unwrap();
    s.editor = Some(PLACED);
    s.editor_full = Some(WORK);
    assert_eq!(read(&toml::Table::try_from(&s).unwrap()), s);
}

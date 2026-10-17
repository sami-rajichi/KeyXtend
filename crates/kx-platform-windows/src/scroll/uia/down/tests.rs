//! The walk down on a fake tree, a window holding a toolbar and a pane with a list and a preview: each level takes
//! the first child holding the point, a leaf or a point no child holds ends it, and so does the time.

use std::cell::Cell;

use kx_module_api::ports::{Point, Rect};

use super::{Tree, deepest};
use crate::screen::contains;

/// The window, its toolbar, its pane, the pane's list and preview, and the list's first item, by number.
const BOXES: [Rect; 6] = [
    Rect {
        x: 0,
        y: 0,
        w: 100,
        h: 100,
    },
    Rect {
        x: 0,
        y: 0,
        w: 100,
        h: 10,
    },
    Rect {
        x: 0,
        y: 10,
        w: 100,
        h: 90,
    },
    Rect {
        x: 0,
        y: 10,
        w: 50,
        h: 90,
    },
    Rect {
        x: 50,
        y: 10,
        w: 50,
        h: 90,
    },
    Rect {
        x: 0,
        y: 10,
        w: 50,
        h: 10,
    },
];
/// Each node's children, in order.
const KIDS: [&[usize]; 6] = [&[1, 2], &[], &[3, 4], &[5], &[], &[]];
/// The window.
const WINDOW: usize = 0;
/// The pane below the toolbar.
const PANE: usize = 2;
/// The pane's list.
const LIST: usize = 3;
/// The pane's preview.
const PREVIEW: usize = 4;
/// The list's first item.
const ITEM: usize = 5;

/// The fake tree.
struct Fake;

impl Tree for Fake {
    type Node = usize;

    fn first(&self, node: &usize) -> Option<usize> {
        KIDS[*node].first().copied()
    }

    fn next(&self, node: &usize) -> Option<usize> {
        let siblings = KIDS.iter().find(|k| k.contains(node))?;
        let at = siblings.iter().position(|k| k == node)?;
        siblings.get(at + 1).copied()
    }

    fn holds(&self, node: &usize, p: Point) -> bool {
        contains(BOXES[*node], p)
    }
}

/// Where the walk down from the window ends at `(x, y)`, with time to spare.
fn down_to(x: i32, y: i32) -> usize {
    deepest(&Fake, WINDOW, Point { x, y }, || false)
}

#[test]
fn each_level_takes_the_first_child_holding_the_point() {
    assert_eq!(
        down_to(25, 15),
        ITEM,
        "past the toolbar, down the pane and the list"
    );
    assert_eq!(down_to(75, 50), PREVIEW, "past the list");
}

#[test]
fn a_point_no_child_holds_ends_the_walk_where_it_is() {
    assert_eq!(down_to(25, 50), LIST, "below the item");
    assert_eq!(down_to(200, 200), WINDOW, "off every child");
}

#[test]
fn the_walk_ends_as_deep_as_it_got_once_the_time_is_up() {
    let checks = Cell::new(0);
    let late = || {
        checks.set(checks.get() + 1);
        checks.get() > 2
    };
    let at = deepest(&Fake, WINDOW, Point { x: 25, y: 15 }, late);
    assert_eq!(at, PANE, "the toolbar, then the pane, then time");
}

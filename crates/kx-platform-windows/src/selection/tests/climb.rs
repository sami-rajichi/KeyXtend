//! The walk up from the focused element to the text that holds its selection: the focused element's own pattern
//! wins, the nearest ancestor within the cap serves, none within it keeps the focused element, and a password box, an
//! opaque window or our own never climbs.

use super::super::kinds::{Tree, climb, shut};
use super::*;

/// UI Automation's Group control type, which a web page's block has and the kind table does not list.
const GROUP: i32 = 50026;
/// The depth the shipped table must reach: Zen's pages held the selection 9 and 11 levels above the focus.
const SEEN: u8 = 11;

/// A chain from the focused element (0) up: which levels have a text pattern, whether the focused one is shut, and
/// how many parents were asked.
struct Chain {
    texts: Vec<bool>,
    shut: bool,
    asked: Cell<usize>,
}

impl Tree for Chain {
    type El = usize;

    fn texts(&self, el: &usize) -> bool {
        self.texts.get(*el).copied().unwrap_or_default()
    }

    fn shut(&self, el: &usize) -> bool {
        *el == 0 && self.shut
    }

    fn parent(&self, el: &usize) -> Option<usize> {
        self.asked.set(self.asked.get() + 1);
        let up = el + 1;
        (up < self.texts.len()).then_some(up)
    }
}

/// A chain of `levels` above the focus whose only text pattern is at level `at`, with the focus `shut` or not.
fn chain(at: usize, levels: usize, shut: bool) -> Chain {
    Chain {
        texts: (0..=levels).map(|i| i == at).collect(),
        shut,
        asked: Cell::new(0),
    }
}

/// Where a walk on `chain` under `cap` ends, and how many parents it asked.
fn walked(chain: &Chain, cap: u8) -> (usize, usize) {
    (climb(chain, 0, cap), chain.asked.get())
}

#[test]
fn the_focused_elements_own_text_pattern_wins_and_asks_no_parent() {
    let both = Chain {
        texts: vec![true, true],
        shut: false,
        asked: Cell::new(0),
    };
    assert_eq!(walked(&both, table().walk_up()), (0, 0));
}

#[test]
fn the_nearest_ancestor_with_a_text_pattern_within_the_cap_is_read() {
    let cap = table().walk_up();
    for at in [1, 9, usize::from(SEEN), usize::from(cap)] {
        assert_eq!(walked(&chain(at, 20, false), cap), (at, at), "{at} up");
    }
    let mut two = chain(5, 20, false);
    two.texts[3] = true;
    assert_eq!(walked(&two, cap), (3, 3), "the nearest");
}

#[test]
fn none_within_the_cap_keeps_the_focused_element_and_asks_no_more() {
    let cap = table().walk_up();
    let past = usize::from(cap) + 1;
    assert_eq!(walked(&chain(past, 20, false), cap), (0, usize::from(cap)));
    let top = chain(usize::MAX, 2, false);
    assert_eq!(walked(&top, cap), (0, 3), "the root has no parent");
    assert_eq!(walked(&chain(1, 2, false), 0), (0, 0), "a cap of 0");
}

#[test]
fn a_shut_focused_element_never_climbs() {
    assert_eq!(walked(&chain(1, 2, true), table().walk_up()), (0, 0));
}

/// A web page's group in another app, with no value pattern.
fn group() -> Facts {
    Facts {
        fixed: None,
        ..plain(GROUP)
    }
}

/// Whether `facts` shut the focus under the shipped field table and the small table.
fn shuts(facts: &Facts) -> bool {
    shut(facts, field::table(), &small())
}

#[test]
fn a_page_its_group_or_its_label_climbs() {
    let label = Facts {
        fixed: None,
        ..plain(TEXT)
    };
    for facts in [group(), label, page()] {
        assert!(!shuts(&facts), "{facts:?}");
    }
}

#[test]
fn a_password_box_an_opaque_window_or_our_own_shuts_the_focus_and_a_password_ancestor_is_closed() {
    let closed = [
        Facts {
            password: Some(true),
            ..group()
        },
        Facts {
            password: None,
            ..group()
        },
        Facts {
            class: String::from("Edit"),
            style: PW,
            ..group()
        },
        Facts {
            top: String::from("TscShellContainerClass"),
            ..group()
        },
        Facts {
            ours: true,
            ..group()
        },
        Facts {
            control: None,
            ..group()
        },
    ];
    for facts in &closed {
        assert!(shuts(facts), "{facts:?}");
    }
    let secret = Facts {
        password: Some(true),
        ..page()
    };
    assert_eq!(kind_of(&secret), SelKind::Closed, "a password ancestor");
}

#[test]
fn the_shipped_cap_reaches_the_pages_seen_and_a_closed_table_climbs_nowhere() {
    assert!(table().walk_up() >= SEEN, "{}", table().walk_up());
    assert_eq!(Table::closed().walk_up(), 0);
}

//! The walk up from the focused element to the text that holds its selection: the focused element's own pattern
//! wins, the nearest ancestor within the cap serves, none within it keeps the focused element, a shut focus never
//! climbs, and on the way up only a password box, a hiding or opaque window or our own stops the walk, never an unread
//! fact; a climbed box offers Copy and Translate only.

use super::super::kinds::{Tree, capped, climb, fence, shut};
use super::*;

/// UI Automation's Group control type, which a web page's block has and the kind table does not list.
const GROUP: i32 = 50026;
/// The depth the shipped table must reach: Zen's pages held the selection 9 and 11 levels above the focus.
const SEEN: u8 = 11;
/// A window class `field.toml` lists as hiding its boxes from UI Automation.
const OPAQUE: &str = "TscShellContainerClass";

/// A chain from the focused element (0) up of the facts each level tells, by the real rules: which levels have a
/// text pattern, and how many parents were asked.
struct Chain {
    texts: Vec<bool>,
    facts: Vec<Facts>,
    asked: Cell<usize>,
}

impl Tree for Chain {
    type El = usize;

    fn texts(&self, el: &usize) -> bool {
        self.texts.get(*el).copied().unwrap_or_default()
    }

    fn shut(&self, el: &usize) -> bool {
        shuts(&self.facts[*el])
    }

    fn fenced(&self, el: &usize) -> bool {
        fence(&self.facts[*el], field::table())
    }

    fn parent(&self, el: &usize) -> Option<usize> {
        self.asked.set(self.asked.get() + 1);
        let up = el + 1;
        (up < self.texts.len()).then_some(up)
    }
}

/// A chain of `levels` groups above the focus whose only text pattern is at level `at`, with the focus shut by an
/// unread control type or not.
fn chain(at: usize, levels: usize, shut: bool) -> Chain {
    let mut facts = vec![group(); levels + 1];
    if shut {
        facts[0].control = None;
    }
    Chain {
        texts: (0..=levels).map(|i| i == at).collect(),
        facts,
        asked: Cell::new(0),
    }
}

/// A web page's group that UI Automation calls a password box.
fn secret() -> Facts {
    Facts {
        password: Some(true),
        ..group()
    }
}

/// Where a walk on `chain` under `cap` ends, and how many parents it asked; it says it climbed only off the focus.
fn walked(chain: &Chain, cap: u8) -> (usize, usize) {
    let (el, up) = climb(chain, 0, cap);
    assert_eq!(up, el != 0, "climbed to {el}");
    (el, chain.asked.get())
}

#[test]
fn the_focused_elements_own_text_pattern_wins_and_asks_no_parent() {
    let mut both = chain(0, 1, false);
    both.texts[1] = true;
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

#[test]
fn a_password_ancestor_under_a_text_one_or_with_text_stops_the_walk_at_the_focus() {
    let mut under = chain(5, 20, false);
    under.facts[2] = secret();
    assert_eq!(walked(&under, table().walk_up()), (0, 2));
    let mut with_text = chain(5, 20, false);
    with_text.facts[5] = secret();
    assert_eq!(
        walked(&with_text, table().walk_up()),
        (0, 5),
        "a password text ancestor"
    );
}

#[test]
fn a_password_ancestor_at_the_caps_edge_stops_the_walk_and_one_past_it_is_never_met() {
    let cap = table().walk_up();
    let edge = usize::from(cap);
    let mut at_edge = chain(edge, 20, false);
    at_edge.facts[edge] = secret();
    assert_eq!(walked(&at_edge, cap), (0, edge));
    let mut past = chain(edge, 20, false);
    past.facts[edge + 1] = secret();
    assert_eq!(walked(&past, cap), (edge, edge));
}

#[test]
fn an_unread_fact_on_an_ancestor_never_stops_the_walk_and_the_box_reached_is_still_closed() {
    let mut unread = chain(5, 20, false);
    unread.facts[2].password = None;
    unread.facts[3].control = None;
    unread.facts[5].password = None;
    assert_eq!(walked(&unread, table().walk_up()), (5, 5));
    assert_eq!(
        kind_of(&unread.facts[5]),
        SelKind::Closed,
        "the box reached"
    );
}

#[test]
fn a_hiding_or_opaque_ancestor_window_or_our_own_stops_the_walk_and_one_with_no_window_does_not() {
    let fences = [
        Facts {
            class: String::from(OPAQUE),
            ..group()
        },
        Facts {
            class: String::from("Edit"),
            style: PW,
            ..group()
        },
        Facts {
            ours: true,
            ..group()
        },
    ];
    for facts in fences {
        let mut stopped = chain(5, 20, false);
        stopped.facts[2] = facts.clone();
        assert_eq!(walked(&stopped, table().walk_up()), (0, 2), "{facts:?}");
    }
    let mut windowless = chain(5, 20, false);
    windowless.facts[2] = Facts {
        class: String::new(),
        top: String::from(OPAQUE),
        ..group()
    };
    assert_eq!(
        walked(&windowless, table().walk_up()),
        (5, 5),
        "no window of its own"
    );
}

#[test]
fn a_climbed_box_offers_copy_and_translate_only_and_the_focused_one_keeps_its_kind() {
    use SelKind::{Closed, Editable, ReadOnly, Terminal};
    let climbed = [
        (Editable, ReadOnly),
        (Terminal, Closed),
        (ReadOnly, ReadOnly),
        (Closed, Closed),
    ];
    for (kind, cap) in climbed {
        assert_eq!(capped(kind, true), cap, "{kind:?} climbed");
        assert_eq!(capped(kind, false), kind, "{kind:?} focused");
    }
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
            top: String::from(OPAQUE),
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

#[test]
fn a_walk_up_of_0_or_past_the_deepest_walk_is_refused_and_the_cap_itself_parses() {
    let with = |n: usize| Table::parse(&SMALL.replace("walk_up = 15", &format!("walk_up = {n}")));
    assert_eq!(
        with(0).err(),
        Some(TableError::Depth),
        "0 turns the climb off"
    );
    let cap = with(MAX_DEPTH).map(|t| usize::from(t.walk_up()));
    assert_eq!(cap, Ok(MAX_DEPTH), "the cap");
    assert_eq!(
        with(MAX_DEPTH + 1).err(),
        Some(TableError::Depth),
        "one past it"
    );
}

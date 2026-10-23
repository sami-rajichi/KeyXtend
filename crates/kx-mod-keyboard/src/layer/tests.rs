use super::*;
use crate::KeyboardSettings;
use crate::spec::BoardSpec;

/// The shipped board's layer grid, and the cell of its Shortcuts key, which shows the layer.
pub(super) fn board() -> (LayerGrid, SideCell) {
    let s = BoardSpec::shipped().unwrap();
    let key = s.side.iter().find(|k| k.role == Role::Layer).unwrap();
    let cell = SideCell {
        col: key.col,
        row: key.row,
    };
    (s.layer_grid(), cell)
}

/// `n` ids from `s<first>` on.
pub(super) fn many(first: usize, n: usize) -> Vec<ShortcutId> {
    let id = |i| ShortcutId::new(&format!("s{i}")).unwrap();
    (first..first + n).map(id).collect()
}

/// Every id on `pages`, as the platform's known set.
fn all(pages: &[Vec<ShortcutId>]) -> HashSet<ShortcutId> {
    pages.iter().flatten().cloned().collect()
}

/// A layer over `pages`, every id known.
fn layer(pages: &[Vec<ShortcutId>]) -> Layer {
    Layer::new(board().0, pages, &all(pages))
}

/// The shortcuts `l` draws now, in order.
fn drawn(l: &Layer) -> Vec<ShortcutId> {
    let keys = l.cells().into_iter().map(|c| c.key);
    let ids = keys.filter_map(|k| match k {
        LayerKey::Shortcut(id) => Some(id),
        LayerKey::Next => None,
    });
    ids.collect()
}

/// True when `l` draws the Next key.
fn has_next(l: &Layer) -> bool {
    l.cells().iter().any(|c| c.key == LayerKey::Next)
}

#[test]
fn a_new_layer_is_hidden_on_its_first_page_and_draws_nothing() {
    let l = layer(&[many(0, 3), many(3, 3)]);
    assert!(!l.shown() && l.any());
    assert_eq!((l.page(), l.page_count()), (0, 2));
    assert!(l.cells().is_empty());
}

#[test]
fn toggle_twice_returns_to_the_usual_side_keys() {
    let mut l = layer(&[many(0, 3), many(3, 3)]);
    l.toggle();
    assert!(l.shown());
    assert_eq!(drawn(&l), many(0, 3));
    l.toggle();
    assert!(!l.shown() && l.cells().is_empty());
}

#[test]
fn next_wraps_from_the_last_page_to_the_first() {
    let pages = [many(0, 2), many(2, 2), many(4, 2)];
    let mut l = layer(&pages);
    l.toggle();
    let mut seen = Vec::new();
    for _ in 0..=pages.len() {
        seen.push((l.page(), drawn(&l)));
        l.next_page();
    }
    let want: Vec<_> = (0..pages.len())
        .chain([0])
        .map(|p| (p, pages[p].clone()))
        .collect();
    assert_eq!(seen, want);
}

#[test]
fn next_does_nothing_while_hidden_or_with_one_page() {
    let mut two = layer(&[many(0, 1), many(1, 1)]);
    two.next_page();
    assert_eq!(two.page(), 0, "hidden");
    let mut one = layer(&[many(0, 2)]);
    one.toggle();
    one.next_page();
    assert_eq!((one.page(), drawn(&one)), (0, many(0, 2)));
}

#[test]
fn one_page_holds_14_and_has_no_next_key() {
    let (grid, _) = board();
    let mut l = layer(&[many(0, 14)]);
    l.toggle();
    assert_eq!(drawn(&l), many(0, 14));
    assert!(!has_next(&l));
    let last = l.cells().last().map(|c| c.cell);
    assert_eq!(
        last,
        Some(grid.next()),
        "the 14th takes the Next key's cell"
    );
}

#[test]
fn with_more_pages_a_page_holds_13_and_next_takes_the_corner() {
    let (grid, _) = board();
    let mut l = layer(&[many(0, 13), many(13, 1)]);
    l.toggle();
    let cells = l.cells();
    assert_eq!((drawn(&l), cells.len()), (many(0, 13), 14));
    let next = cells.last().unwrap();
    assert_eq!((next.cell, &next.key), (grid.next(), &LayerKey::Next));
}

#[test]
fn shortcuts_past_the_room_are_dropped_not_drawn() {
    let mut l = layer(&[many(0, 20)]);
    l.toggle();
    assert_eq!(drawn(&l), many(0, 14));
}

#[test]
fn unknown_ids_repeats_and_pages_left_empty_are_dropped() {
    let pages = [many(0, 2), many(10, 1), Vec::new(), many(1, 2)];
    let known: HashSet<ShortcutId> = many(0, 3).into_iter().collect();
    let mut l = Layer::new(board().0, &pages, &known);
    assert_eq!(l.page_count(), 2);
    l.toggle();
    assert_eq!(drawn(&l), many(0, 2));
    l.next_page();
    assert_eq!(drawn(&l), many(2, 1), "s1 came first on page 1");
}

#[test]
fn nothing_to_show_cannot_toggle_on() {
    let none = Layer::new(board().0, &[many(0, 3)], &HashSet::new());
    let mut l = none.clone();
    l.toggle();
    assert!(!l.any() && !l.shown() && l.cells().is_empty());
    assert_eq!((l.page(), l.page_count()), (0, 0));
    let mut empty = layer(&[]);
    empty.toggle();
    assert!(!empty.shown());
}

#[test]
fn a_smaller_list_brings_the_page_back_in_range_and_keeps_it_shown() {
    let mut l = layer(&[many(0, 1), many(1, 1), many(2, 1)]);
    l.toggle();
    l.next_page();
    l.next_page();
    let fewer = [many(0, 1), many(1, 1)];
    l.renew(&fewer, &all(&fewer));
    assert!(l.shown());
    assert_eq!((l.page(), drawn(&l)), (1, many(1, 1)));
    let known: HashSet<ShortcutId> = many(0, 1).into_iter().collect();
    l.renew(&fewer, &known);
    assert_eq!((l.page(), l.page_count(), drawn(&l)), (0, 1, many(0, 1)));
}

#[test]
fn renew_keeps_the_page_in_range_and_nothing_left_hides_the_layer() {
    let pages = [many(0, 1), many(1, 1)];
    let mut l = layer(&pages);
    l.toggle();
    l.next_page();
    let more = [many(0, 1), many(1, 1), many(2, 1)];
    l.renew(&more, &all(&more));
    assert_eq!((l.shown(), l.page()), (true, 1));
    l.renew(&more, &HashSet::new());
    assert!(!l.shown() && !l.any() && l.cells().is_empty());
    l.renew(&pages, &all(&pages));
    assert!(!l.shown(), "a renew never shows the layer");
}

#[test]
fn the_owners_pages_fill_the_side_block_less_the_shortcuts_key_and_next() {
    let (grid, layer_cell) = board();
    let pages = KeyboardSettings::from_defaults().unwrap().shortcut_pages;
    let mut l = Layer::new(grid.clone(), &pages, &all(&pages));
    l.toggle();
    assert_eq!(l.page_count(), 3);
    let cells: Vec<SideCell> = l.cells().iter().map(|c| c.cell).collect();
    let mut want = grid.cells(l.page_count());
    want.push(grid.next());
    assert_eq!(cells, want, "13 shortcuts, then Next");
    assert!(!cells.contains(&layer_cell));
}

#[test]
fn each_key_gives_its_generated_id_and_role() {
    let copy = ShortcutId::new("copy").unwrap();
    let key = LayerKey::Shortcut(copy.clone());
    assert_eq!(
        (key.id(), key.role()),
        (shortcut_key_id(&copy), Role::Shortcut)
    );
    assert_eq!(
        (LayerKey::Next.id(), LayerKey::Next.role()),
        (NEXT_ID.to_owned(), Role::Next)
    );
}

mod props;

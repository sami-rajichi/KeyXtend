//! The layer's rules under any pages, known ids and clicks: cells never clash, Next shows only with pages to turn,
//! the page stays in range, and Next turns full circle.

use super::*;
use proptest::prelude::*;

/// What can happen to the layer.
#[derive(Clone, Debug)]
enum Op {
    /// The Shortcuts key.
    Toggle,
    /// The Next key.
    Next,
    /// New pages or known ids.
    Renew(Vec<Vec<ShortcutId>>, HashSet<ShortcutId>),
}

/// Ids from a small pool, so repeats and unknown ids are common.
fn an_id() -> impl Strategy<Value = ShortcutId> {
    (0usize..30).prop_map(|i| many(i, 1).remove(0))
}

/// Up to ten pages of up to twenty ids.
fn some_pages() -> impl Strategy<Value = Vec<Vec<ShortcutId>>> {
    prop::collection::vec(prop::collection::vec(an_id(), 0..20), 0..10)
}

/// Any subset of the pool.
fn some_known() -> impl Strategy<Value = HashSet<ShortcutId>> {
    prop::collection::hash_set(an_id(), 0..30)
}

fn an_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => Just(Op::Toggle),
        3 => Just(Op::Next),
        1 => (some_pages(), some_known()).prop_map(|(p, k)| Op::Renew(p, k)),
    ]
}

/// Applies `op` to `l`.
fn apply(l: &mut Layer, op: &Op) {
    match op {
        Op::Toggle => l.toggle(),
        Op::Next => l.next_page(),
        Op::Renew(pages, known) => l.renew(pages, known),
    }
}

/// Checks every rule on `l`, whose layer key sits in `layer_cell`.
fn rules(l: &Layer, grid: &LayerGrid, layer_cell: SideCell) -> Result<(), TestCaseError> {
    let cells = l.cells();
    let spots: HashSet<SideCell> = cells.iter().map(|c| c.cell).collect();
    prop_assert_eq!(spots.len(), cells.len(), "a side cell twice");
    prop_assert!(!spots.contains(&layer_cell), "on the Shortcuts key");
    let ids: HashSet<String> = cells.iter().map(|c| c.key.id()).collect();
    prop_assert_eq!(ids.len(), cells.len(), "a key id twice");
    let nexts: Vec<&LayerCell> = cells.iter().filter(|c| c.key == LayerKey::Next).collect();
    let paged = l.shown() && l.page_count() > 1;
    prop_assert_eq!(nexts.len(), usize::from(paged));
    prop_assert!(nexts.iter().all(|c| c.cell == grid.next()));
    prop_assert!(cells.len() - nexts.len() <= grid.capacity(l.page_count()));
    prop_assert!(!l.shown() || (l.any() && !cells.is_empty()));
    prop_assert!(!l.any() || l.page() < l.page_count());
    prop_assert_eq!(l.any(), l.page_count() > 0);
    Ok(())
}

proptest! {
    #[test]
    fn any_clicks_keep_every_rule(pages in some_pages(), known in some_known(), ops in prop::collection::vec(an_op(), 0..40)) {
        let (grid, layer_cell) = board();
        let mut l = Layer::new(grid.clone(), &pages, &known);
        rules(&l, &grid, layer_cell)?;
        for op in &ops {
            apply(&mut l, op);
            rules(&l, &grid, layer_cell)?;
        }
    }

    #[test]
    fn next_page_count_times_comes_back_to_the_start(pages in some_pages(), known in some_known(), turns in 0usize..12) {
        let (grid, _) = board();
        let mut l = Layer::new(grid, &pages, &known);
        l.toggle();
        for _ in 0..turns {
            l.next_page();
        }
        let start = (l.page(), l.cells());
        for _ in 0..l.page_count() {
            l.next_page();
        }
        prop_assert_eq!((l.page(), l.cells()), start);
    }

    #[test]
    fn toggle_twice_always_returns(pages in some_pages(), known in some_known(), ops in prop::collection::vec(an_op(), 0..20)) {
        let (grid, _) = board();
        let mut l = Layer::new(grid, &pages, &known);
        for op in &ops {
            apply(&mut l, op);
        }
        let before = (l.shown(), l.page(), l.cells());
        l.toggle();
        l.toggle();
        prop_assert_eq!((l.shown(), l.page(), l.cells()), before);
    }
}

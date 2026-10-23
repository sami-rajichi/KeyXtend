//! The shortcuts layer on the board: its roles, the Next cell, the cells a page fills and the ids it generates.

use super::*;
use kx_module_api::ports::ShortcutId;

/// The side key `id` of `s`, to change.
fn side_mut<'a>(s: &'a mut BoardSpec, id: &str) -> &'a mut SideSpec {
    s.side.iter_mut().find(|k| k.id == id).unwrap()
}

/// The shipped board with no key that shows the layer.
fn without_layer() -> BoardSpec {
    let mut s = spec();
    side_mut(&mut s, "shortcuts").role = Role::Later;
    s
}

/// Every side cell of `s` in reading order: row by row, left to right.
fn reading(s: &BoardSpec) -> Vec<SideCell> {
    let rows = u16::try_from(s.rows.len()).unwrap();
    let row = |row| (1..=s.side_columns).map(move |col| SideCell { col, row });
    (1..=rows).flat_map(row).collect()
}

/// The cell of the side key `id` of `s`.
fn cell_of(s: &BoardSpec, id: &str) -> SideCell {
    let k = s.side.iter().find(|k| k.id == id).unwrap();
    SideCell {
        col: k.col,
        row: k.row,
    }
}

#[test]
fn the_next_key_sits_in_the_bottom_right_side_cell() {
    let s = spec();
    let rows = u16::try_from(s.rows.len()).unwrap();
    let corner = SideCell {
        col: s.side_columns,
        row: rows,
    };
    assert_eq!(s.layer_grid().next(), corner);
    assert_eq!(s.check(), Ok(()));
}

#[test]
fn a_page_holds_13_shortcuts_or_14_when_it_is_the_only_one() {
    let s = spec();
    assert_eq!(s.check(), Ok(()));
    let g = s.layer_grid();
    assert_eq!((s.side_columns, s.rows.len()), (3, 5), "today's side grid");
    assert_eq!((g.capacity(1), g.capacity(2), g.capacity(8)), (14, 13, 13));
    assert_eq!(g.capacity(0), g.capacity(1), "no pages need no Next key");
}

#[test]
fn the_cells_run_in_reading_order_less_the_layer_key_and_then_next() {
    let s = spec();
    let g = s.layer_grid();
    let layer = cell_of(&s, "shortcuts");
    let alone: Vec<SideCell> = reading(&s).into_iter().filter(|c| *c != layer).collect();
    assert_eq!(g.cells(1), alone);
    let paged: Vec<SideCell> = alone.into_iter().filter(|c| *c != g.next()).collect();
    assert_eq!(g.cells(2), paged);
    assert_eq!(g.capacity(2), paged.len());
}

#[test]
fn without_a_layer_key_every_side_cell_is_the_layers() {
    let s = without_layer();
    assert_eq!(s.layer_grid().cells(1), reading(&s));
}

#[test]
fn the_layer_hides_every_side_key_but_its_own_and_no_main_key() {
    let s = spec();
    let hidden: Vec<&str> = s
        .side
        .iter()
        .map(|k| k.id.as_str())
        .filter(|id| s.layer_hides(id))
        .collect();
    assert_eq!(hidden.len(), s.side.len() - 1, "{hidden:?}");
    assert!(!s.layer_hides("shortcuts") && !s.layer_hides("esc") && !s.layer_hides(NEXT_ID));
    assert!(
        without_layer().layer_hides("shortcuts"),
        "a key that shows no layer hides"
    );
}

#[test]
fn next_must_sit_on_the_side_grid() {
    for next in [[0, 5], [4, 5], [3, 0], [3, 6]] {
        let mut s = spec();
        s.layer.next = next;
        assert_eq!(s.check(), Err(BoardError::Next), "{next:?}");
    }
    let mut s = spec();
    s.layer.next = [1, 2];
    assert_eq!(s.check(), Ok(()), "the layer covers the D-pad too");
}

#[test]
fn next_may_not_take_the_layer_keys_cell() {
    let mut s = spec();
    let layer = cell_of(&s, "shortcuts");
    s.layer.next = [layer.col, layer.row];
    assert_eq!(s.check(), Err(BoardError::Next));
}

#[test]
fn only_one_side_key_shows_the_layer() {
    let mut s = spec();
    side_mut(&mut s, "mic").role = Role::Layer;
    assert_eq!(s.check(), Err(BoardError::Role("mic".into())));
}

/// The shipped Mic key as `board.toml` writes it.
const MIC: &str = "{ id = \"mic\", col = 3, row = 2, role = \"later\" }";

#[test]
fn the_generated_roles_are_refused_in_the_file() {
    assert!(BOARD.contains(MIC));
    for role in ["shortcut", "next"] {
        let text = BOARD.replace(MIC, &MIC.replace("later", role));
        assert_eq!(
            BoardSpec::read(&text),
            Err(BoardError::Role("mic".into())),
            "{role}"
        );
    }
    let mut s = spec();
    named_mut(&mut s, "lang").role = Role::Next;
    assert_eq!(
        s.check(),
        Err(BoardError::Role("lang".into())),
        "a main key"
    );
}

#[test]
fn the_shipped_board_has_one_layer_key_and_one_power_key() {
    let s = spec();
    let with = |r: Role| -> Vec<&str> {
        let keys = s.side.iter().filter(|k| k.role == r);
        keys.map(|k| k.id.as_str()).collect()
    };
    assert_eq!(with(Role::Layer), ["shortcuts"]);
    assert_eq!(with(Role::Power), ["power"]);
    let none = [Role::Layer, Role::Power, Role::Shortcut, Role::Next];
    assert!(none.iter().all(|r| !r.sends()), "no scan code of their own");
}

#[test]
fn the_layer_and_power_roles_belong_to_the_side_block() {
    for role in [Role::Layer, Role::Power] {
        let mut s = spec();
        named_mut(&mut s, "lang").role = role;
        assert_eq!(s.check(), Err(BoardError::Role("lang".into())), "{role:?}");
    }
}

#[test]
fn a_shortcut_keys_id_is_the_prefix_and_its_shortcut() {
    let copy_all = ShortcutId::new("copy-all").unwrap();
    assert_eq!(shortcut_key_id(&copy_all), "sc-copy-all");
    assert_eq!(NEXT_ID, "layer-next");
}

#[test]
fn board_ids_keep_clear_of_the_generated_ids() {
    let copy = shortcut_key_id(&ShortcutId::new("copy").unwrap());
    for id in [NEXT_ID.to_owned(), copy, "sc-".to_owned()] {
        let mut s = spec();
        side_mut(&mut s, "mic").id.clone_from(&id);
        assert_eq!(s.check(), Err(BoardError::Reserved(id.clone())), "{id}");
        let mut s = spec();
        named_mut(&mut s, "menu").id.clone_from(&id);
        assert_eq!(s.check(), Err(BoardError::Reserved(id.clone())), "{id}");
    }
    let mut s = spec();
    side_mut(&mut s, "mic").id = "scan".into();
    assert_eq!(s.check(), Ok(()), "only the prefix and its dash are kept");
}

proptest! {
    #[test]
    fn any_side_key_may_show_the_layer_and_its_cell_is_never_filled(pick in 0usize..9, pages in 0usize..10) {
        let mut s = without_layer();
        let at = pick % s.side.len();
        let layer = SideCell { col: s.side[at].col, row: s.side[at].row };
        prop_assume!([layer.col, layer.row] != s.layer.next);
        s.side[at].role = Role::Layer;
        prop_assert_eq!(s.check(), Ok(()));
        let g = s.layer_grid();
        let cells = g.cells(pages);
        let unique: std::collections::HashSet<SideCell> = cells.iter().copied().collect();
        prop_assert_eq!(unique.len(), cells.len(), "a cell twice");
        prop_assert!(!unique.contains(&layer));
        prop_assert_eq!(unique.contains(&g.next()), pages <= 1);
        prop_assert_eq!(g.capacity(pages), reading(&s).len() - 1 - usize::from(pages > 1));
    }
}

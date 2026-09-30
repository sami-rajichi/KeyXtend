use super::*;
use kx_module_api::ports::{LangLabel, LayoutId};
use proptest::prelude::*;

const ALL: [Nav; 7] = [
    Nav::Up,
    Nav::Down,
    Nav::Left,
    Nav::Right,
    Nav::Enter,
    Nav::Space,
    Nav::Esc,
];

/// A list of seven rows opened on row `at`, as the mock-up's clipboard sample pages them.
fn seven(at: usize) -> (LangList, usize) {
    (LangList::open(at), 7)
}

#[test]
fn up_and_down_move_one_row_and_stop_at_the_ends() {
    let (mut l, len) = seven(0);
    assert_eq!(l.key(Nav::Up, 5, len), Moved::Stay, "already first");
    assert_eq!(l.key(Nav::Down, 5, len), Moved::Hi);
    assert_eq!(l.hi(len), 1);
    let (mut l, len) = seven(6);
    assert_eq!(l.key(Nav::Down, 5, len), Moved::Stay, "already last");
    assert_eq!(l.key(Nav::Up, 5, len), Moved::Hi);
    assert_eq!(l.hi(len), 5);
}

#[test]
fn down_past_a_page_shows_the_next_page_at_the_given_size() {
    let (mut l, len) = seven(4);
    l.key(Nav::Down, 5, len);
    assert_eq!(l.hi(len) / 5, 1, "row 6 is on page 2 of five-row pages");
    let (mut l, len) = seven(1);
    l.key(Nav::Down, 2, len);
    assert_eq!(l.hi(len) / 2, 1, "row 3 is on page 2 of two-row pages");
}

#[test]
fn left_and_right_turn_a_page_to_its_first_row() {
    let (mut l, len) = seven(2);
    assert_eq!(l.key(Nav::Right, 5, len), Moved::Hi);
    assert_eq!(l.hi(len), 5, "page 2 starts at row 6");
    assert_eq!(l.key(Nav::Right, 5, len), Moved::Stay, "the last page");
    assert_eq!(l.key(Nav::Left, 5, len), Moved::Hi);
    assert_eq!(l.hi(len), 0);
    let (mut l, len) = seven(3);
    assert_eq!(l.key(Nav::Right, 3, len), Moved::Hi);
    assert_eq!(l.hi(len), 6, "three-row pages: page 3 starts at row 7");
}

#[test]
fn left_on_the_first_page_goes_to_its_first_row() {
    let (mut l, len) = seven(3);
    assert_eq!(l.key(Nav::Left, 5, len), Moved::Hi);
    assert_eq!(l.hi(len), 0);
}

#[test]
fn enter_picks_the_highlight_esc_closes_and_space_does_nothing() {
    let (mut l, len) = seven(3);
    assert_eq!(l.key(Nav::Enter, 5, len), Moved::Pick(3));
    assert_eq!(l.key(Nav::Esc, 5, len), Moved::Close);
    assert_eq!(l.key(Nav::Space, 5, len), Moved::Stay, "no row to tick");
}

#[test]
fn an_empty_list_only_closes_and_a_zero_page_size_counts_as_one() {
    let mut l = LangList::open(0);
    for nav in [Nav::Up, Nav::Down, Nav::Left, Nav::Right, Nav::Enter] {
        assert_eq!(l.key(nav, 5, 0), Moved::Stay, "{nav:?}");
    }
    assert_eq!(l.key(Nav::Esc, 5, 0), Moved::Close);
    let (mut l, len) = seven(0);
    assert_eq!(l.key(Nav::Right, 0, len), Moved::Hi);
    assert_eq!(l.hi(len), 1);
}

#[test]
fn a_list_that_shrank_keeps_the_highlight_on_its_last_row() {
    let (l, _) = seven(6);
    assert_eq!(l.hi(3), 2);
    assert_eq!(l.hi(0), 0);
}

#[test]
fn a_huge_page_size_from_the_ui_never_overflows() {
    let (mut l, len) = seven(6);
    assert_eq!(l.key(Nav::Right, usize::MAX, len), Moved::Hi);
    assert_eq!(l.hi(len), 0, "one page holds every row");
}

#[test]
fn the_board_names_one_key_for_each_panel_move() {
    let navs = navs(&BoardSpec::shipped().unwrap());
    let ids: Vec<&str> = navs.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(
        ids,
        ["esc", "enter", "up", "space", "left", "down", "right"]
    );
    for nav in ALL {
        assert_eq!(navs.iter().filter(|(_, n)| *n == nav).count(), 1, "{nav:?}");
    }
}

fn label(n: u64, short: &str, rtl: bool) -> LangLabel {
    LangLabel {
        id: LayoutId::new(n),
        short: short.into(),
        name: vec![short.into()],
        rtl,
        arabic: rtl,
    }
}

#[test]
fn the_view_lists_every_layout_and_ticks_the_current_one() {
    let labels = [label(1, "EN", false), label(2, "ع", true)];
    let keys = ["esc".to_owned()];
    let v = list_view(&LangList::open(1), &labels, LayoutId::new(2), &keys);
    assert_eq!(
        (v.hi, v.rows.len(), v.keys.clone()),
        (1, 2, vec!["esc".to_owned()])
    );
    assert_eq!(v.rows[1].icon, Some(CURRENT_ICON));
    assert!(v.rows[1].current && v.rows[1].rtl && v.rows[1].arabic);
    assert_eq!((v.rows[0].icon, v.rows[0].current), (None, false));
}

proptest! {
    #[test]
    fn any_keys_keep_the_highlight_on_a_row(
        len in 1usize..20,
        rows in 0usize..8,
        at in 0usize..20,
        keys in proptest::collection::vec(0usize..7, 0..40),
    ) {
        let mut l = LangList::open(at);
        for k in keys {
            let nav = ALL[k];
            let before = l.hi(len);
            let page = |hi: usize| hi / rows.max(1);
            match l.key(nav, rows, len) {
                Moved::Pick(n) => prop_assert_eq!(n, before),
                Moved::Stay => prop_assert_eq!(l.hi(len), before),
                Moved::Close => prop_assert_eq!(nav, Nav::Esc),
                Moved::Hi => prop_assert_ne!(l.hi(len), before),
            }
            prop_assert!(l.hi(len) < len);
            if matches!(nav, Nav::Left | Nav::Right) {
                prop_assert_eq!(l.hi(len) % rows.max(1), 0, "a page turn lands on a page's first row");
                let last = (len - 1) / rows.max(1);
                let want = if nav == Nav::Left { page(before).saturating_sub(1) } else { (page(before) + 1).min(last) };
                prop_assert_eq!(page(l.hi(len)), want);
            }
        }
    }
}

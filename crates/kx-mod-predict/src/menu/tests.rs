//! The menu's state machine: each tap's effect, and a property that any taps keep the list's page in range.

use proptest::prelude::*;

use super::*;

#[test]
fn forget_then_a_forgotten_chip_shows_undo_until_the_next_key() {
    let mut menu = Menu::default();
    menu.forget();
    assert!(menu.forgetting);
    menu.forgot(Some(2));
    assert_eq!((menu.forgetting, menu.undo), (false, Some(2)));
    menu.key();
    assert!(!menu.undo(), "gone at the next key");
    menu.forget();
    menu.forgot(None);
    assert_eq!(
        (menu.forgetting, menu.undo),
        (false, None),
        "nothing forgotten"
    );
}

#[test]
fn undo_answers_once() {
    let mut menu = Menu::default();
    menu.forgot(Some(0));
    assert!(menu.undo());
    assert!(!menu.undo());
}

#[test]
fn the_list_opens_on_its_first_page_and_turns_within_its_pages() {
    let mut menu = Menu::default();
    menu.turn(1, 5, 12);
    assert_eq!(menu.list, None, "a closed list does not turn");
    menu.forget();
    menu.open(5);
    assert!(!menu.forgetting, "opening the list ends forget mode");
    menu.turn(1, 5, 12);
    menu.turn(1, 5, 12);
    menu.turn(1, 5, 12);
    assert_eq!(menu.list, Some(List { page: 2, rows: 5 }), "three pages");
    menu.turn(-9, 4, 12);
    assert_eq!(menu.list, Some(List { page: 0, rows: 4 }));
    assert_eq!(List { page: 2, rows: 4 }.at(1), 9);
    menu.open(0);
    assert_eq!(menu.list.map(|l| l.rows), Some(1), "at least one row");
}

#[test]
fn cancel_and_off_close_everything() {
    let mut menu = Menu::default();
    menu.open(5);
    menu.cancel();
    assert_eq!(menu.list, None);
    menu.forget();
    menu.forgot(Some(1));
    menu.off();
    assert_eq!(menu, Menu::default());
}

/// One tap on the menu or the list.
#[derive(Clone, Debug)]
enum Tap {
    Forget,
    Cancel,
    Forgot(Option<usize>),
    Key,
    Open(u8),
    Turn(i8, u8),
}

fn tap() -> impl Strategy<Value = Tap> {
    prop_oneof![
        Just(Tap::Forget),
        Just(Tap::Cancel),
        proptest::option::of(0usize..7).prop_map(Tap::Forgot),
        Just(Tap::Key),
        any::<u8>().prop_map(Tap::Open),
        (any::<i8>(), any::<u8>()).prop_map(|(by, rows)| Tap::Turn(by, rows)),
    ]
}

proptest! {
    #[test]
    fn any_taps_keep_the_page_within_the_list_and_rows_above_zero(taps in proptest::collection::vec(tap(), 0..40), count in 0usize..60) {
        let mut menu = Menu::default();
        for tap in taps {
            match tap {
                Tap::Forget => menu.forget(),
                Tap::Cancel => menu.cancel(),
                Tap::Forgot(at) => menu.forgot(at),
                Tap::Key => menu.key(),
                Tap::Open(rows) => menu.open(rows),
                Tap::Turn(by, rows) => menu.turn(by, rows, count),
            }
            if let Some(list) = menu.list {
                prop_assert!(list.rows > 0);
                prop_assert!(!menu.forgetting, "the list and forget mode never both open");
                let pages = count.div_ceil(usize::from(list.rows)).max(1);
                prop_assert!(usize::from(list.page) < pages || list.page == 0);
            }
        }
    }
}

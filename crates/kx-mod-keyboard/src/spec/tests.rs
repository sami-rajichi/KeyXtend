use super::*;
use proptest::prelude::*;

fn spec() -> BoardSpec {
    BoardSpec::shipped().unwrap()
}

/// The named key `id` of `s`.
fn named<'a>(s: &'a BoardSpec, id: &str) -> &'a KeySpec {
    let keys = s.rows.iter().flatten().filter_map(|i| match i {
        RowItem::Key(k) => Some(k),
        RowItem::Chars(_) => None,
    });
    keys.into_iter().find(|k| k.id == id).unwrap()
}

/// The named key `id` of `s`, to change.
fn named_mut<'a>(s: &'a mut BoardSpec, id: &str) -> &'a mut KeySpec {
    let keys = s.rows.iter_mut().flatten().filter_map(|i| match i {
        RowItem::Key(k) => Some(k),
        RowItem::Chars(_) => None,
    });
    keys.into_iter().find(|k| k.id == id).unwrap()
}

fn one_char(code: u16) -> RowItem {
    RowItem::Chars(CharSpec {
        chars: vec![Scan::new(code)],
    })
}

#[test]
fn the_shipped_board_has_48_character_keys_and_9_side_keys() {
    let s = spec();
    let runs = s.rows.iter().flatten().map(|i| match i {
        RowItem::Chars(c) => c.chars.len(),
        RowItem::Key(_) => 0,
    });
    let chars: usize = runs.sum();
    assert_eq!((chars, s.side.len(), s.rows.len()), (48, 9, 5));
    assert_eq!(s.check(), Ok(()));
}

#[test]
fn char_scans_list_the_character_keys_in_row_order() {
    let s = spec();
    let keys = (0..s.rows.len()).flat_map(|r| s.row_keys(r));
    let want: Vec<Scan> = keys
        .filter_map(|k| match k {
            RowKey::Char { sc, .. } => Some(sc),
            RowKey::Named(_) => None,
        })
        .collect();
    assert_eq!(s.char_scans(), want);
    assert_eq!(want.len(), 48);
}

#[test]
fn every_row_spans_the_main_block_and_a_long_row_is_refused() {
    let mut s = spec();
    s.rows[2].push(one_char(0x56));
    let want = BoardError::Row {
        row: 3,
        spans: 68,
        want: 64,
    };
    assert_eq!(s.check(), Err(want));
}

proptest! {
    #[test]
    fn adding_or_dropping_a_row_item_breaks_that_row(row in 0usize..5, pick in 0usize..16, add in any::<bool>()) {
        let mut s = spec();
        let items = &mut s.rows[row];
        let at = pick % items.len();
        if add {
            items.insert(at, one_char(0x56));
        } else {
            items.remove(at);
        }
        let broken = matches!(s.check(), Err(BoardError::Row { row: r, .. }) if r == row + 1);
        prop_assert!(broken, "row {} still passes", row + 1);
    }
}

#[test]
fn a_key_that_is_not_whole_columns_is_refused() {
    let mut s = spec();
    named_mut(&mut s, "esc").w = 1.1;
    assert_eq!(s.check(), Err(BoardError::Width("esc".into())));
    assert_eq!(s.columns(1.75), Some(7));
    assert_eq!(
        (s.columns(0.1), s.columns(0.0), s.columns(-1.0)),
        (None, None, None)
    );
    s.columns_per_unit = 10;
    assert_eq!(
        s.columns(1.1),
        Some(11),
        "a tiny float error still counts as whole"
    );
}

#[test]
fn sizes_must_be_numbers_above_zero() {
    let cases: [fn(&mut BoardSpec); 7] = [
        |s| s.unit_px = 0.0,
        |s| s.row_px = f64::NAN,
        |s| s.columns_per_unit = 0,
        |s| s.side_columns = 0,
        |s| s.main_units = 16.1,
        |s| s.plate_pad_px[1] = -1.0,
        |s| s.dpad.units = f64::INFINITY,
    ];
    for (i, change) in cases.iter().enumerate() {
        let mut s = spec();
        change(&mut s);
        assert_eq!(s.check(), Err(BoardError::Size), "case {i}");
    }
}

#[test]
fn the_dpad_must_sit_on_the_side_grid_in_order() {
    for cell in [
        [0, 2, 2, 4],
        [2, 2, 1, 4],
        [1, 4, 2, 2],
        [1, 2, 9, 4],
        [1, 2, 2, 6],
    ] {
        let mut s = spec();
        s.dpad.cell = cell;
        assert_eq!(s.check(), Err(BoardError::Dpad), "{cell:?}");
    }
}

#[test]
fn a_side_key_off_the_grid_on_the_dpad_or_in_a_taken_cell_is_refused() {
    let mut s = spec();
    s.side[0].row = 2;
    assert_eq!(
        s.check(),
        Err(BoardError::Side("rclick".into())),
        "on the D-pad"
    );
    let mut s = spec();
    (s.side[1].col, s.side[1].row) = (s.side[0].col, s.side[0].row);
    assert_eq!(
        s.check(),
        Err(BoardError::Side("grab".into())),
        "a taken cell"
    );
    let mut s = spec();
    s.side[2].col = 4;
    assert_eq!(
        s.check(),
        Err(BoardError::Side("shortcuts".into())),
        "off the grid"
    );
}

#[test]
fn an_id_used_twice_is_refused() {
    let mut s = spec();
    s.side[3].id = "esc".into();
    assert_eq!(s.check(), Err(BoardError::Twice("esc".into())));
    let mut s = spec();
    named_mut(&mut s, "menu").id = char_id(Scan::new(0x02));
    assert_eq!(s.check(), Err(BoardError::Twice(char_id(Scan::new(0x02)))));
}

#[test]
fn a_role_must_fit_its_key() {
    let role = |id: &str, change: fn(&mut KeySpec)| {
        let mut s = spec();
        change(named_mut(&mut s, id));
        s.check()
    };
    let refused = |id: &str| Err(BoardError::Role(id.into()));
    assert_eq!(
        role("esc", |k| k.role = Role::Char),
        refused("esc"),
        "chars come in runs"
    );
    assert_eq!(
        role("tab", |k| k.sc = None),
        refused("tab"),
        "send needs a code"
    );
    assert_eq!(
        role("lang", |k| k.sc = Some(Scan::new(0x01))),
        refused("lang")
    );
    let mut s = spec();
    s.side[4].role = Role::Send;
    assert_eq!(
        s.check(),
        Err(BoardError::Role("clip".into())),
        "side keys send no code"
    );
}

#[test]
fn the_shipped_roles_latch_the_modifiers_and_wait_on_the_side_block() {
    let s = spec();
    let latches = [
        ("shift", Latch::Shift),
        ("rshift", Latch::Shift),
        ("ctrl", Latch::Ctrl),
        ("alt", Latch::Alt),
        ("altgr", Latch::AltGr),
        ("win", Latch::Win),
    ];
    for (id, l) in latches {
        assert_eq!(named(&s, id).role, Role::Latch(l), "{id}");
    }
    assert_eq!(
        (named(&s, "caps").role, named(&s, "lang").role),
        (Role::Caps, Role::Lang)
    );
    assert_eq!(named(&s, "space").role, Role::Send);
    assert!(s.side.iter().all(|k| k.role == Role::Later) && s.dpad.later);
    let del = named(&s, "del").sc.unwrap();
    assert!(del.is_extended() && del.code() == 0xE053);
}

#[test]
fn a_broken_board_is_an_error_not_a_panic() {
    let text = BOARD.replace("role = \"caps\"", "role = \"fly\"");
    assert!(matches!(BoardSpec::read(&text), Err(BoardError::Parse(_))));
    let text = BOARD.replace("latch = \"win\"", "latch = \"hyper\"");
    assert!(matches!(BoardSpec::read(&text), Err(BoardError::Parse(_))));
    let text = format!("{BOARD}\nspare = 1");
    assert!(matches!(BoardSpec::read(&text), Err(BoardError::Parse(_))));
    for bad in ["", "unit_px = [", "rows = 1"] {
        assert!(
            matches!(BoardSpec::read(bad), Err(BoardError::Parse(_))),
            "{bad:?}"
        );
    }
}

#[test]
fn a_board_error_keeps_its_kind_when_it_stops_the_start() {
    let all = [
        BoardError::Parse(String::new()),
        BoardError::Size,
        BoardError::Width(String::new()),
        BoardError::Row {
            row: 1,
            spans: 1,
            want: 2,
        },
        BoardError::Dpad,
        BoardError::Side(String::new()),
        BoardError::Twice(String::new()),
        BoardError::Role(String::new()),
    ];
    let kinds: std::collections::HashSet<&str> = all.iter().map(BoardError::kind).collect();
    assert_eq!(kinds.len(), all.len(), "each kind names its rule");
    let start = kx_module_api::ModuleError::from(BoardError::Dpad);
    assert_eq!(
        start,
        kx_module_api::ModuleError::Start(BoardError::Dpad.kind())
    );
}

mod ids;

use super::*;
use crate::KeyboardSettings;
use crate::spec::char_id;
use proptest::prelude::*;

/// Allowed rounding error in logical pixels.
const EPS: f64 = 1e-6;
/// The largest key gap the tests try, in logical pixels; the themes use 3 to 5.
const MAX_GAP: f64 = 8.0;

fn spec() -> BoardSpec {
    BoardSpec::shipped().unwrap()
}

/// The smallest and largest sizes from the settings defaults.
fn limits() -> (f64, f64) {
    let s = KeyboardSettings::from_defaults().unwrap();
    (s.size_min, s.size_max)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

fn inside(outer: &Area, inner: &Area) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.x + inner.w <= outer.x + outer.w + EPS
        && inner.y + inner.h <= outer.y + outer.h + EPS
}

#[test]
fn the_mock_up_board_is_about_671_by_196_at_size_1() {
    let b = board(&spec(), 4.0, 1.0);
    assert!(close(b.plate.w, 670.78), "{}", b.plate.w);
    assert!(close(b.plate.h, 196.0), "{}", b.plate.h);
    let count = |kind| b.keys.iter().filter(|k| k.kind == kind).count();
    assert_eq!((count(KeyKind::Char), count(KeyKind::Act)), (47, 9));
    let mut ids: Vec<&str> = b.keys.iter().map(|k| k.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), b.keys.len(), "every id once");
}

#[test]
fn character_keys_carry_their_row_code_and_role() {
    let b = board(&spec(), 4.0, 1.0);
    let first = b.keys.iter().find(|k| k.id == char_id(0, 0)).unwrap();
    assert_eq!(
        (first.sc, first.row, first.role),
        (Some(Scan::new(0x29)), 0, Role::Char)
    );
    let q = b.keys.iter().find(|k| k.id == char_id(1, 0)).unwrap();
    assert_eq!((q.sc, q.row), (Some(Scan::new(0x10)), 1));
    let mic = b.keys.iter().find(|k| k.id == "mic").unwrap();
    assert_eq!(
        (mic.kind, mic.role, mic.sc, mic.row),
        (KeyKind::Act, Role::Later, None, 1)
    );
}

proptest! {
    #[test]
    fn every_row_spans_the_grid_at_any_size(size in limits().0..=limits().1, gap in 0.0..MAX_GAP) {
        let s = spec();
        let b = board(&s, gap, size);
        let [_, _, _, pad_left] = s.plate_pad_px;
        let (start, end) = (pad_left * size, (pad_left + s.main_units * s.unit_px) * size);
        for row in 0..s.rows.len() {
            let cells: Vec<Area> = b.keys.iter()
                .filter(|k| k.row == row && k.kind != KeyKind::Act)
                .map(|k| k.cell)
                .collect();
            prop_assert!(close(cells[0].x, start), "row {row} starts at the block's edge");
            for pair in cells.windows(2) {
                prop_assert!(close(pair[0].x + pair[0].w, pair[1].x), "row {row} has a hole or an overlap");
                prop_assert!(close(pair[0].y, pair[1].y) && close(pair[0].h, pair[1].h));
            }
            let last = cells[cells.len() - 1];
            prop_assert!(close(last.x + last.w, end), "row {row} ends at the block's edge");
        }
    }

    #[test]
    fn keys_dpad_and_bar_stay_inside_the_plate(size in limits().0..=limits().1, gap in 0.0..MAX_GAP) {
        let b = board(&spec(), gap, size);
        prop_assert!(b.keys.iter().all(|k| inside(&b.plate, &k.cap) && inside(&k.cell, &k.cap)));
        prop_assert!(inside(&b.plate, &b.dpad) && inside(&b.plate, &b.bar));
    }

    #[test]
    fn a_larger_size_scales_every_box(size in limits().0..=limits().1) {
        let (one, big) = (board(&spec(), 4.0, 1.0), board(&spec(), 4.0, size));
        let same = |a: Area, b: Area| {
            let s = a.scale(size);
            close(s.x, b.x) && close(s.y, b.y) && close(s.w, b.w) && close(s.h, b.h)
        };
        prop_assert!(same(one.plate, big.plate) && same(one.dpad, big.dpad) && same(one.bar, big.bar));
        prop_assert!(one.keys.iter().zip(&big.keys).all(|(a, b)| same(a.cap, b.cap)));
    }
}

#[test]
fn the_dpad_is_the_smaller_of_its_width_and_height_less_a_gap() {
    let s = spec();
    let b = board(&s, 4.0, 1.0);
    let [_, r0, _, r1] = s.dpad.cell;
    let rows = f64::from(r1 + 1 - r0);
    let d = (s.dpad.units * s.unit_px).min(rows * s.row_px) - 4.0;
    assert!(close(b.dpad.w, d) && close(b.dpad.h, d), "{:?}", b.dpad);
}

#[test]
fn side_keys_sit_in_their_cells_right_of_the_main_block() {
    let s = spec();
    let b = board(&s, 0.0, 1.0);
    let [pad_top, _, _, pad_left] = s.plate_pad_px;
    let side_x = pad_left + s.main_units * s.unit_px + s.body_gap_px;
    let top = pad_top + s.top_bar_px + s.top_gap_px;
    let power = b.keys.iter().find(|k| k.id == "power").unwrap();
    assert!(close(power.cell.x, side_x + 2.0 * s.side_unit * s.unit_px));
    assert!(close(power.cell.y, top + 4.0 * s.row_px) && power.row == 4);
    assert_eq!(power.cell, power.cap, "no gap, so the cap fills the cell");
}

#[test]
fn a_broken_board_never_panics() {
    let mut s = spec();
    s.columns_per_unit = 0;
    s.side[0].col = 0;
    s.dpad.cell = [3, 4, 1, 2];
    let b = board(&s, 4.0, 1.0);
    assert_eq!(
        b.keys.len(),
        board(&spec(), 4.0, 1.0).keys.len(),
        "every key still listed"
    );
}

#[test]
fn a_box_grows_and_scales() {
    let a = Area {
        x: 1.0,
        y: 2.0,
        w: 3.0,
        h: 4.0,
    };
    let grown = Area {
        x: 0.0,
        y: 1.0,
        w: 5.0,
        h: 6.0,
    };
    assert_eq!(a.grow(1.0), grown);
    assert_eq!(a.grow(1.0).grow(-1.0), a);
    assert_eq!(
        a.scale(2.0),
        Area {
            x: 2.0,
            y: 4.0,
            w: 6.0,
            h: 8.0
        }
    );
}

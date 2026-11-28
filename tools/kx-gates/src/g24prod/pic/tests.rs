//! Tests of `pic`.

use super::*;

/// A DIB header of `w` by `h` at `bits` per pixel with `compression`, sized by its rows.
fn header(w: i32, h: i32, bits: u16, compression: u32) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend(40u32.to_le_bytes());
    b.extend(w.to_le_bytes());
    b.extend(h.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(bits.to_le_bytes());
    b.extend(compression.to_le_bytes());
    b.extend([0u8; 20]);
    b
}

/// A 2 by 2, 24-bit DIB whose rows are given top first, laid out `up` (bottom row first) or top-down.
fn dib24(rows: [[[u8; 3]; 2]; 2], up: bool) -> Vec<u8> {
    let mut b = header(2, if up { 2 } else { -2 }, 24, 0);
    let order: Vec<usize> = if up { vec![1, 0] } else { vec![0, 1] };
    for r in order {
        for px in rows[r] {
            b.extend(px);
        }
        b.extend([0u8; 2]);
    }
    b
}

const A: [u8; 3] = [1, 2, 3];
const B: [u8; 3] = [4, 5, 6];
const C: [u8; 3] = [7, 8, 9];
const D: [u8; 3] = [10, 11, 12];

#[test]
fn a_bottom_up_or_top_down_dib_reads_top_left_first() {
    for up in [true, false] {
        let p = Pic::from_dib(&dib24([[A, B], [C, D]], up)).expect("reads");
        assert_eq!(p.size(), (2, 2));
        assert_eq!(
            [p.at(0, 0), p.at(1, 0), p.at(0, 1), p.at(1, 1)],
            [Some(A), Some(B), Some(C), Some(D)],
            "{up}"
        );
        assert_eq!(p.at(2, 0), None);
    }
}

#[test]
fn a_32_bit_dib_with_or_without_its_masks_drops_its_fourth_byte() {
    for (compression, masks) in [(0u32, 0usize), (3, 12)] {
        let mut b = header(1, 1, 32, compression);
        b.extend(vec![0u8; masks]);
        b.extend([1, 2, 3, 255]);
        let p = Pic::from_dib(&b).expect("reads");
        assert_eq!(p.at(0, 0), Some(A), "{compression}");
    }
}

#[test]
fn a_dib_that_is_short_paletted_empty_or_packed_is_refused() {
    let mut short = dib24([[A, B], [C, D]], true);
    short.pop();
    assert!(Pic::from_dib(&short).is_err(), "short");
    assert!(Pic::from_dib(&header(2, 2, 8, 0)).is_err(), "paletted");
    assert!(Pic::from_dib(&header(0, 2, 24, 0)).is_err(), "empty");
    assert!(Pic::from_dib(&header(1, 1, 24, 1)).is_err(), "packed");
    assert!(Pic::from_dib(&[0u8; 12]).is_err(), "no header");
}

#[test]
fn a_screen_grab_keeps_its_place_and_drops_its_alpha() {
    let shot = Shot {
        left: 10,
        top: 20,
        width: 2,
        height: 1,
        bgra: vec![1, 2, 3, 0, 4, 5, 6, 255],
    };
    let p = Pic::from_shot(&shot);
    assert_eq!((p.left, p.top, p.size()), (10, 20, (2, 1)));
    assert_eq!([p.at(10, 20), p.at(11, 20)], [Some(A), Some(B)]);
}

/// A 2 by 2 picture at `left`, `top`, its rows top first.
fn pic(left: i32, top: i32, rows: [[[u8; 3]; 2]; 2]) -> Pic {
    Pic::from_dib(&dib24(rows, true))
        .expect("reads")
        .placed(left, top)
}

/// A box from its edges.
fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

#[test]
fn the_share_counts_differing_pixels_both_hold_in_the_box() {
    let a = pic(0, 0, [[A, B], [C, D]]);
    let all = rect(0, 0, 2, 2);
    assert_eq!(differing(&a, &a, &all), Some(0.0));
    assert_eq!(
        differing(&a, &pic(0, 0, [[A, B], [C, A]]), &all),
        Some(0.25)
    );
    let moved = pic(1, 0, [[B, A], [D, A]]);
    assert_eq!(
        differing(&a, &moved, &all),
        Some(0.0),
        "only the column both hold"
    );
    assert_eq!(differing(&a, &pic(5, 5, [[A, A], [A, A]]), &all), None);
    assert_eq!(differing(&a, &a, &rect(1, 1, 1, 2)), None, "an empty box");
}

#[test]
fn a_picture_all_one_colour_is_uniform_as_a_blank_drawing_is() {
    assert!(pic(0, 0, [[A, A], [A, A]]).uniform());
    assert!(!pic(0, 0, [[A, A], [A, B]]).uniform());
}

#[test]
fn the_middle_of_a_box_is_its_half_each_way_which_a_round_window_fills() {
    let m = middle(&rect(10, 20, 50, 60));
    assert_eq!((m.left, m.top, m.right, m.bottom), (20, 30, 40, 50));
}

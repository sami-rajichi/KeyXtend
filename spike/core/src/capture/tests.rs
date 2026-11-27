use super::*;

/// A size cap far above the tiny test pictures.
const CAP: usize = 1 << 20;

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// A 3×2 copy at (10, 20) whose pixel bytes count up from 0.
fn shot() -> Shot {
    Shot {
        left: 10,
        top: 20,
        width: 3,
        height: 2,
        bgra: (0..24).collect(),
    }
}

#[test]
fn header_describes_a_top_down_32_bit_picture() {
    let b = header(3, 2, 3 * 2 * 4);
    assert_eq!(b.len(), 54);
    assert_eq!(&b[..2], b"BM");
    assert_eq!(u32_at(&b, 2), 54 + 3 * 2 * 4);
    assert_eq!(u32_at(&b, 10), 54);
    assert_eq!(u32_at(&b, 14), 40);
    assert_eq!(u32_at(&b, 18), 3);
    assert_eq!(u32_at(&b, 22) as i32, -2);
    assert_eq!(u16::from_le_bytes([b[28], b[29]]), 32);
    assert_eq!(u32_at(&b, 34), 3 * 2 * 4);
}

#[test]
fn crop_keeps_the_overlap_only() {
    let s = shot();
    let r = RECT {
        left: 11,
        top: 21,
        right: 99,
        bottom: 99,
    };
    let c = s.crop(&r).expect("overlaps");
    assert_eq!((c.left, c.top, c.width, c.height), (11, 21, 2, 1));
    assert_eq!(c.bgra, (16..24).collect::<Vec<u8>>());
    let away = RECT {
        left: 0,
        top: 0,
        right: 10,
        bottom: 20,
    };
    assert!(s.crop(&away).is_none());
}

#[test]
fn a_bmp_is_the_header_then_the_pixels() {
    let s = shot();
    let b = s.to_bmp().expect("fits");
    let at = PIXELS_AT as usize;
    assert_eq!(b[..at], header(3, 2, 3 * 2 * 4)[..]);
    assert_eq!(b[at..], s.bgra[..]);
}

#[test]
fn sizes_that_are_empty_or_pass_the_cap_are_refused() {
    assert_eq!(byte_len(3, 2, 24), Some(24));
    assert_eq!(byte_len(3, 2, 23), None);
    assert_eq!(byte_len(0, 2, CAP), None);
    assert_eq!(byte_len(-3, 2, CAP), None);
    assert_eq!(byte_len(3, -2, CAP), None);
}

#[test]
fn grab_refuses_a_bad_size_before_copying() {
    assert!(grab(0, 0, 0, 5, CAP).is_err());
    assert!(grab(0, 0, -3, 5, CAP).is_err());
    assert!(grab(0, 0, 4, 4, 4 * 4 * 4 - 1).is_err());
}

#[test]
fn rgba_swaps_blue_and_red_and_sets_alpha() {
    let s = Shot {
        left: 0,
        top: 0,
        width: 1,
        height: 1,
        bgra: vec![1, 2, 3, 0],
    };
    assert_eq!(s.rgba(), vec![3, 2, 1, 255]);
}

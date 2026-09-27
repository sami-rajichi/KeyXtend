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
fn a_region_is_the_same_whatever_corner_comes_first() {
    let (a, b) = (Pt { x: 5, y: 9 }, Pt { x: 2, y: 3 });
    let want = RECT {
        left: 2,
        top: 3,
        right: 6,
        bottom: 10,
    };
    assert_eq!(region(a, b), want);
    assert_eq!(region(b, a), want);
    assert_eq!(region(Pt { x: 2, y: 9 }, Pt { x: 5, y: 3 }), want);
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
fn a_bmp_reads_back_the_same_pixels() {
    let s = shot();
    let back = from_bmp(&s.to_bmp().expect("fits"), CAP).expect("reads");
    assert_eq!((back.width, back.height, back.bgra), (3, 2, s.bgra));
}

#[test]
fn a_bad_or_short_bmp_is_refused() {
    let mut b = shot().to_bmp().expect("fits");
    assert!(from_bmp(&b[..60], CAP).is_err(), "short");
    b[0] = b'X';
    assert!(from_bmp(&b, CAP).is_err(), "bad magic");
    assert!(from_bmp(&[], CAP).is_err());
}

#[test]
fn a_bottom_up_or_oversize_bmp_is_refused() {
    let good = shot().to_bmp().expect("fits");
    let mut up = good.clone();
    up[22..26].copy_from_slice(&2i32.to_le_bytes());
    assert!(from_bmp(&up, CAP).is_err(), "bottom-up");
    assert!(from_bmp(&good, 3 * 2 * 4 - 1).is_err(), "over the cap");
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

//! The wiped buffer keeps every item through each growth, stops doubling at its limit, and clears to empty.

use super::*;

#[test]
fn items_survive_each_growth_in_order() {
    let mut buf = Buf::new();
    for n in 0_i16..1000 {
        buf.push(n).unwrap();
    }
    buf.extend(&[7, 8, 9]).unwrap();
    let want: Vec<i16> = (0..1000).chain([7, 8, 9]).collect();
    assert_eq!(&*buf, &want[..]);
}

#[test]
fn doubling_stops_at_the_limit_unless_more_is_asked() {
    let mut buf: Buf<u8> = Buf::with_limit(10);
    buf.extend(&[1; 8]).unwrap();
    buf.push(2).unwrap();
    assert!(buf.items.capacity() <= 10, "{}", buf.items.capacity());
    buf.extend(&[3; 20]).unwrap();
    assert_eq!(buf.len(), 29);
}

#[test]
fn a_reserve_past_memory_is_a_memory_fault() {
    let mut buf: Buf<i16> = Buf::new();
    buf.push(1).unwrap();
    assert_eq!(buf.reserve(usize::MAX), Err(VoiceFault::Memory));
    assert_eq!(buf.reserve(usize::MAX / 4), Err(VoiceFault::Memory));
    assert_eq!(&*buf, &[1]);
}

#[test]
fn a_collect_past_memory_is_a_memory_fault() {
    let huge = std::iter::repeat_n(0_i16, usize::MAX / 4);
    assert!(matches!(Buf::collect(huge), Err(VoiceFault::Memory)));
    let buf = Buf::collect([1_i16, 2].into_iter()).unwrap();
    assert_eq!((&*buf, buf.items.capacity()), (&[1, 2][..], 2));
}

#[test]
fn clear_empties_and_keeps_the_room() {
    let mut buf = Buf::collect([1.0_f32, 2.0, 3.0].into_iter()).unwrap();
    let room = buf.items.capacity();
    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.items.capacity(), room);
}

#[test]
fn drain_front_keeps_the_rest_in_order() {
    let mut buf = Buf::collect([1_u8, 2, 3, 4].into_iter()).unwrap();
    buf.drain_front(3);
    assert_eq!(&*buf, &[4]);
    buf.drain_front(9);
    assert!(buf.is_empty());
}

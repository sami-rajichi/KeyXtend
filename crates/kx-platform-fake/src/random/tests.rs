//! The seeded random source: the same seed gives the same bytes, clones share one stream, a failure comes on cue, and
//! the bytes spread over every value.

use super::*;
use std::collections::HashSet;

/// Draws `n` bytes from `source`.
fn draw(source: &FakeRandom, n: usize) -> Vec<u8> {
    let mut buf = vec![0; n];
    source.fill(&mut buf).unwrap();
    buf
}

#[test]
fn one_seed_gives_one_stream_and_another_seed_another() {
    assert_eq!(draw(&FakeRandom::new(7), 40), draw(&FakeRandom::new(7), 40));
    assert_ne!(draw(&FakeRandom::new(7), 40), draw(&FakeRandom::new(8), 40));
}

#[test]
fn the_stream_reads_the_same_in_any_cuts() {
    let whole = draw(&FakeRandom::new(3), 37);
    let cut = FakeRandom::new(3);
    let parts = [draw(&cut, 5), draw(&cut, 0), draw(&cut, 13), draw(&cut, 19)].concat();
    assert_eq!(parts, whole);
}

#[test]
fn clones_share_one_stream() {
    let a = FakeRandom::new(11);
    let b = a.clone();
    let both = [draw(&a, 9), draw(&b, 9)].concat();
    assert_eq!(both, draw(&FakeRandom::new(11), 18));
}

#[test]
fn a_failure_comes_once_on_cue_and_leaves_the_buffer_and_stream() {
    let source = FakeRandom::new(5);
    source.fail_next();
    let mut buf = [9u8; 4];
    assert_eq!(source.fill(&mut buf), Err(RandomError));
    assert_eq!(buf, [9; 4]);
    assert_eq!(draw(&source, 8), draw(&FakeRandom::new(5), 8));
    assert_eq!(source.fills(), 2);
}

#[test]
fn every_byte_value_turns_up() {
    let seen: HashSet<u8> = draw(&FakeRandom::new(1), 8192).into_iter().collect();
    assert_eq!(seen.len(), usize::from(u8::MAX) + 1);
}

#[test]
fn it_works_as_the_port() {
    let source: Arc<dyn Random> = Arc::new(FakeRandom::new(2));
    let mut buf = [0u8; 16];
    source.fill(&mut buf).unwrap();
    assert_ne!(buf, [0; 16]);
}

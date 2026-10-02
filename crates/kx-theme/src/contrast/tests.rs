//! Contrast tests: the WCAG maths, the user's minimum, and the rings on a plain window of their mode.

use super::*;
use crate::Themes;
use crate::drawn::WINDOWS;

fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

#[test]
fn every_ring_stands_out_on_a_plain_window_of_its_mode() {
    for t in load().list {
        let id = &t.look.id;
        assert!(
            contrast(t.light.ring, WINDOWS[0]) >= WCAG_SHAPES,
            "{id} light"
        );
        assert!(
            contrast(t.dark.ring, WINDOWS[1]) >= WCAG_SHAPES,
            "{id} dark"
        );
    }
}

#[test]
fn contrast_follows_wcag() {
    let (black, white) = (Rgba::parse("#000000"), Rgba::parse("#FFFFFF"));
    let (black, white) = (black.expect("black"), white.expect("white"));
    assert!((contrast(black, white) - 21.0).abs() < 0.01);
    assert!(
        (contrast(white, black) - 21.0).abs() < 0.01,
        "order does not matter"
    );
    assert!((contrast(white, white) - 1.0).abs() < 0.01);
}

#[test]
fn a_user_minimum_starts_at_wcag_aa_and_ends_at_black_on_white() {
    let (black, white) = (Rgba::parse("#000000"), Rgba::parse("#FFFFFF"));
    let (black, white) = (black.expect("black"), white.expect("white"));
    assert!((contrast(black, white) - MAX_CONTRAST).abs() < 0.01);
    assert!((MIN_CONTRAST.start() - WCAG_AA).abs() < f32::EPSILON);
    assert!(MIN_CONTRAST.contains(&MAX_CONTRAST));
    for bad in [4.4, MAX_CONTRAST + 1.0, f32::NAN] {
        assert!(!MIN_CONTRAST.contains(&bad), "{bad}");
    }
}

#[test]
fn the_better_of_two_inks_is_the_one_further_from_the_background() {
    let (black, white) = (Rgba::BLACK, Rgba::WHITE);
    assert_eq!(readable(black, white, white), black);
    assert_eq!(readable(black, white, black), white);
}

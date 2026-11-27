//! What a click reaches, from the theme's sizes and its slop at the scale shown: the Eraser's aim at each kind of
//! drawing, only on what shows, and two clicks within the slop as one spot.

use super::*;

/// The picture shown at `shown` window pixels for every `Scale::PER` of its own.
fn scale(shown: u32) -> Scale {
    Scale::new(shown).unwrap()
}

/// The scales the tests click at: twice, once and half the picture's size.
fn scales() -> [u32; 3] {
    [Scale::PER * 2, Scale::PER, Scale::PER / 2]
}

/// The slop at `shown`, in the picture's pixels; whole at the test's scales.
fn slop(shown: u32) -> i32 {
    let (per, shown) = (
        i32::try_from(Scale::PER).unwrap(),
        i32::try_from(shown).unwrap(),
    );
    i32::from(SLOP) * per / shown
}

/// The thickness step new drawings take.
fn middle() -> Thickness {
    THICK[THICK.len() / 2]
}

/// An editor with one drawing of `tool` from `from` to `to`, drawn at its own size, then the Eraser picked.
fn drawn_with(tool: Tool, from: Point, to: Point) -> Canvas {
    let mut c = with(tool);
    two(&mut c, from, to);
    assert_eq!(drawn(&c).len(), 1, "{tool:?} drew");
    c.tool(Tool::Eraser);
    c
}

#[test]
fn a_click_reaches_half_a_line_and_the_slop_at_each_scale() {
    for shown in scales() {
        let edge = i32::from(middle().pen / 2) + slop(shown);
        let mut c = drawn_with(Tool::Line, pt(10, 40), pt(110, 40));
        let at = scale(shown);
        assert_eq!(
            c.press(pt(60, 40 + edge + 1), at),
            Did::Stay,
            "past the reach at {shown}"
        );
        assert_eq!(
            c.press(pt(60, 40 + edge), at),
            Did::Change,
            "at its edge at {shown}"
        );
    }
}

#[test]
fn the_highlighter_reaches_by_its_own_width() {
    let mut c = with(Tool::Highlighter);
    two(&mut c, pt(10, 40), pt(110, 40));
    c.tool(Tool::Eraser);
    let edge = i32::from(middle().marker / 2) + slop(Scale::PER);
    assert_eq!(c.press(pt(60, 40 + edge + 1), SAME), Did::Stay);
    assert_eq!(c.press(pt(60, 40 + edge), SAME), Did::Change);
}

#[test]
fn an_arrows_head_reaches_further_than_a_line() {
    let reach = i32::from(middle().head + middle().pen / 2) + slop(Scale::PER);
    let beside_tip = pt(80, 40 + reach);
    let mut line = drawn_with(Tool::Line, pt(10, 40), pt(80, 40));
    assert_eq!(
        line.press(beside_tip, SAME),
        Did::Stay,
        "a line has no head"
    );
    let mut arrow = drawn_with(Tool::Arrow, pt(10, 40), pt(80, 40));
    assert_eq!(arrow.press(pt(80, 40 + reach + 1), SAME), Did::Stay);
    assert_eq!(
        arrow.press(beside_tip, SAME),
        Did::Change,
        "its head is reached"
    );
}

#[test]
fn a_steps_disc_is_reached_across_its_size() {
    let mut c = with(Tool::Steps);
    c.press(pt(60, 40), SAME);
    c.tool(Tool::Eraser);
    let edge = i32::from(middle().disc / 2) + slop(Scale::PER);
    assert_eq!(c.press(pt(60 + edge + 1, 40), SAME), Did::Stay);
    assert_eq!(c.press(pt(60 + edge, 40), SAME), Did::Change);
}

#[test]
fn a_text_is_as_wide_as_its_letters_not_its_marks() {
    // Five letters with four marks: the box takes five letter widths, leftward as it reads right to left.
    let size = TEXT[TEXT.len() / 2];
    let mut c = with(Tool::Text);
    c.press(pt(100, 30), SAME);
    c.write("مَرْحَبًا");
    c.end();
    c.tool(Tool::Eraser);
    let left = 100 - 5 * i32::from(size.letter);
    let below = 30 + i32::from(size.height);
    let edge = slop(Scale::PER);
    assert_eq!(
        c.press(pt(left - edge - 1, 40), SAME),
        Did::Stay,
        "past its first letter"
    );
    assert_eq!(
        c.press(pt(60, below + edge + 1), SAME),
        Did::Stay,
        "under its line"
    );
    assert_eq!(c.press(pt(left - edge, 40), SAME), Did::Change);
}

#[test]
fn the_eraser_takes_only_what_shows() {
    let mut c = with(Tool::Line);
    // Six wide, the line on row 15 ends above the crop's first row, and the one on row 68 reaches into its last.
    two(&mut c, pt(10, 15), pt(100, 15));
    two(&mut c, pt(10, 68), pt(100, 68));
    c.tool(Tool::Crop);
    cut(&mut c, pt(10, 20), pt(100, 66));
    c.tool(Tool::Eraser);
    assert_eq!(
        c.press(pt(50, 20), SAME),
        Did::Stay,
        "the line above the crop does not show"
    );
    assert_eq!(
        c.press(pt(50, 66), SAME),
        Did::Change,
        "the line below shows its edge"
    );
    assert_eq!(marks(&c), [shape(Shape::Line, pt(10, 15), pt(100, 15))]);
}

#[test]
fn two_clicks_within_the_slop_are_one_spot_at_each_scale() {
    for tool in [Tool::Box, Tool::Line, Tool::Crop] {
        for shown in scales() {
            // The farthest diagonal step still within the slop, and the next one.
            let mut near = 0;
            while f64::from(near + 1) * std::f64::consts::SQRT_2 <= f64::from(slop(shown)) {
                near += 1;
            }
            let mut c = with(tool);
            let whole = c.view().crop;
            let at = scale(shown);
            c.press(pt(30, 30), at);
            assert_eq!(
                c.press(pt(30 + near, 30 + near), at),
                Did::Show,
                "{tool:?} at {shown}"
            );
            assert!(
                drawn(&c).is_empty() && c.view().crop == whole,
                "{tool:?} at {shown}"
            );
            assert_eq!(c.view().frame, None, "{tool:?} at {shown}");
            c.press(pt(30, 30), at);
            let apart = pt(30 + near + 1, 30 + near + 1);
            // Crop's second click shows its frame, which its ✓ cuts to.
            let (want, framed) = if tool == Tool::Crop {
                (Did::Show, true)
            } else {
                (Did::Change, false)
            };
            assert_eq!(c.press(apart, at), want, "{tool:?} at {shown}");
            assert_eq!(c.view().frame.is_some(), framed, "{tool:?} at {shown}");
        }
    }
}

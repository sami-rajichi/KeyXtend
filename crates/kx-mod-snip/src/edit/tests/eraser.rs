//! The Eraser: a click removes just the top drawing under it, judged by its shape and size; an empty spot does nothing
//! (edge 18); an erase is undone in its place.

use super::*;

/// An editor with lines across the picture at each row `ys`, then the Eraser picked.
fn lines(ys: &[i32]) -> Canvas {
    let mut c = with(Tool::Line);
    for y in ys {
        two(&mut c, pt(10, *y), pt(100, *y));
    }
    c.tool(Tool::Eraser);
    c
}

#[test]
fn the_eraser_removes_just_the_drawing_clicked() {
    let mut c = lines(&[10, 60]);
    assert_eq!(c.press(pt(50, 11), SAME), Did::Change);
    assert_eq!(marks(&c), [shape(Shape::Line, pt(10, 60), pt(100, 60))]);
}

#[test]
fn edge_18_the_eraser_on_an_empty_spot_does_nothing() {
    let mut c = lines(&[10, 60]);
    let before = drawn(&c);
    assert_eq!(c.press(pt(50, 35), SAME), Did::Stay, "between the lines");
    assert_eq!(c.press(pt(50, -30), SAME), Did::Stay, "outside the picture");
    assert_eq!(drawn(&c), before);
    c.undo();
    assert_eq!(
        drawn(&c),
        before[..1],
        "Undo takes back the last line, as no erase was done"
    );
}

#[test]
fn the_eraser_takes_the_top_drawing_first() {
    let mut c = with(Tool::Line);
    two(&mut c, pt(10, 10), pt(100, 80));
    two(&mut c, pt(10, 80), pt(100, 10));
    c.tool(Tool::Eraser);
    c.press(pt(55, 45), SAME);
    assert_eq!(
        marks(&c),
        [shape(Shape::Line, pt(10, 10), pt(100, 80))],
        "the later one went"
    );
}

#[test]
fn an_erase_is_undone_in_its_place_and_steps_count_from_the_highest() {
    let mut c = with(Tool::Steps);
    for x in [10, 50, 90] {
        c.press(pt(x, 40), SAME);
    }
    let three = drawn(&c);
    c.tool(Tool::Eraser);
    c.press(pt(50, 40), SAME);
    assert_eq!(c.undo(), Did::Change);
    assert_eq!(
        drawn(&c),
        three,
        "the middle step is back between the others"
    );
    c.redo();
    c.tool(Tool::Steps);
    c.press(pt(50, 70), SAME);
    assert_eq!(
        marks(&c).last(),
        Some(&Mark::Step {
            at: pt(50, 70),
            n: 4
        })
    );
}

/// An editor with one drawing of `tool` from `from` to `to`, then the Eraser picked; Steps and Text take `from` alone.
fn alone(tool: Tool, from: Point, to: Point) -> Canvas {
    let mut c = with(tool);
    c.press(from, SAME);
    match tool {
        Tool::Text => {
            c.write("Hello");
            c.end();
        }
        Tool::Steps => {}
        _ => {
            c.press(to, SAME);
        }
    }
    assert_eq!(drawn(&c).len(), 1, "{tool:?} drew");
    c.tool(Tool::Eraser);
    c
}

#[test]
fn the_eraser_reaches_each_drawing_by_its_shape() {
    // Each drawing alone, then a spot beside it and a spot on it.
    let cases = [
        (Tool::Box, [pt(20, 20), pt(100, 80)], pt(60, 50), pt(60, 21)),
        (
            Tool::Cover,
            [pt(40, 30), pt(80, 60)],
            pt(10, 85),
            pt(60, 45),
        ),
        (
            Tool::Circle,
            [pt(20, 20), pt(100, 80)],
            pt(60, 50),
            pt(99, 50),
        ),
        (
            Tool::Arrow,
            [pt(10, 10), pt(100, 10)],
            pt(55, 50),
            pt(55, 14),
        ),
        (Tool::Pen, [pt(10, 40), pt(50, 80)], pt(10, 80), pt(30, 61)),
        (
            Tool::Steps,
            [pt(60, 45), pt(60, 45)],
            pt(100, 10),
            pt(62, 47),
        ),
        (Tool::Text, [pt(20, 30), pt(20, 30)], pt(60, 85), pt(25, 35)),
    ];
    for (tool, [from, to], beside, on) in cases {
        let mut c = alone(tool, from, to);
        assert_eq!(c.press(beside, SAME), Did::Stay, "beside the {tool:?}");
        assert_eq!(c.press(on, SAME), Did::Change, "on the {tool:?}");
    }
}

#[test]
fn right_to_left_text_is_reached_on_the_left_of_its_point() {
    let mut c = with(Tool::Text);
    c.press(pt(100, 30), SAME);
    c.write("مرحبا");
    c.end();
    c.tool(Tool::Eraser);
    assert_eq!(
        c.press(pt(115, 35), SAME),
        Did::Stay,
        "nothing reads right of its point"
    );
    assert_eq!(c.press(pt(90, 35), SAME), Did::Change);
}

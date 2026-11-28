//! Each tool's click, move, click: a drawing is added only when it ends, takes the ink and size picked, and a drawing
//! begun outside the picture is clipped (edge 18).

use super::*;

/// What a pen's path makes.
type Path = fn(Vec<Point>) -> Mark;

#[test]
fn the_pen_and_highlighter_go_down_move_and_lift() {
    let pens: [(Tool, Path); 2] = [(Tool::Pen, Mark::Pen), (Tool::Highlighter, Mark::Highlight)];
    for (tool, mark) in pens {
        let mut c = with(tool);
        assert_eq!(c.press(pt(10, 10), SAME), Did::Show, "{tool:?} goes down");
        assert_eq!(c.view().pending, Some(pt(10, 10)));
        assert!(
            drawn(&c).is_empty(),
            "{tool:?} adds nothing before it lifts"
        );
        let path = [pt(12, 11), pt(15, 14), pt(20, 20)];
        assert_eq!(c.stroke(&path), Did::Change, "{tool:?} lifts");
        let points = vec![pt(10, 10), pt(12, 11), pt(15, 14), pt(20, 20)];
        assert_eq!(marks(&c), [mark(points)]);
        assert_eq!(c.view().pending, None);
    }
}

#[test]
fn a_path_with_no_pen_down_is_ignored() {
    let mut c = with(Tool::Pen);
    assert_eq!(c.stroke(&[pt(1, 1), pt(5, 5)]), Did::Stay);
    let mut c = with(Tool::Box);
    c.press(pt(1, 1), SAME);
    assert_eq!(c.stroke(&[pt(5, 5)]), Did::Stay, "only a pen lifts");
    assert!(drawn(&c).is_empty());
}

#[test]
fn shapes_take_a_start_then_an_end() {
    let shapes = [
        (Tool::Box, Shape::Box),
        (Tool::Circle, Shape::Circle),
        (Tool::Arrow, Shape::Arrow),
        (Tool::Line, Shape::Line),
        (Tool::Cover, Shape::Cover),
    ];
    assert_eq!(shapes.len(), Shape::ALL.len(), "every shape");
    for (tool, kind) in shapes {
        let mut c = with(tool);
        assert_eq!(c.press(pt(30, 20), SAME), Did::Show);
        assert_eq!(
            c.view().pending,
            Some(pt(30, 20)),
            "{tool:?} draws on to the pointer"
        );
        assert!(drawn(&c).is_empty());
        assert_eq!(c.press(pt(60, 50), SAME), Did::Change);
        assert_eq!(marks(&c), [shape(kind, pt(30, 20), pt(60, 50))]);
    }
}

#[test]
fn crop_keeps_the_part_between_its_two_clicks() {
    let mut c = with(Tool::Crop);
    assert_eq!(two(&mut c, pt(49, 59), pt(10, 20)), (Did::Show, Did::Show));
    assert_eq!(c.crop(CropKey::Ok), Did::Change, "once its ✓ is clicked");
    let kept = Rect {
        x: 10,
        y: 20,
        w: 40,
        h: 40,
    };
    assert_eq!(
        c.view().crop.rect(),
        kept,
        "both corner pixels, clicked in any order"
    );
    assert!(drawn(&c).is_empty(), "a crop is no drawing");
}

#[test]
fn the_same_spot_twice_or_a_flat_shape_adds_nothing() {
    for tool in [Tool::Pen, Tool::Box, Tool::Arrow, Tool::Crop] {
        let mut c = with(tool);
        let whole = c.view().crop;
        assert_eq!(two(&mut c, pt(5, 5), pt(5, 5)), (Did::Show, Did::Show));
        assert!(drawn(&c).is_empty(), "{tool:?}");
        let view = c.view();
        assert_eq!((view.crop, view.pending, view.frame), (whole, None, None));
    }
    for tool in [Tool::Box, Tool::Circle, Tool::Cover] {
        let mut c = with(tool);
        assert_eq!(two(&mut c, pt(5, 5), pt(5, 40)).1, Did::Show);
        assert!(drawn(&c).is_empty(), "a {tool:?} with no width");
    }
    let mut c = with(Tool::Line);
    assert_eq!(
        two(&mut c, pt(5, 5), pt(5, 40)).1,
        Did::Change,
        "a line may stand"
    );
}

#[test]
fn steps_drop_the_next_number_each_click() {
    let mut c = with(Tool::Steps);
    for x in [10, 30, 50] {
        assert_eq!(c.press(pt(x, 40), SAME), Did::Change);
    }
    let steps = [(10, 1), (30, 2), (50, 3)].map(|(x, n)| Mark::Step { at: pt(x, 40), n });
    assert_eq!(marks(&c), steps);
}

#[test]
fn a_drawing_takes_the_ink_and_thickness_picked() {
    let mut c = with(Tool::Line);
    assert_eq!(c.ink(Ink::Blue), Did::Show);
    assert_eq!(c.ink(Ink::Blue), Did::Stay, "the same ink again");
    let start = c.view().size;
    assert_eq!(c.size(true), Did::Show);
    two(&mut c, pt(1, 1), pt(9, 9));
    let line = &drawn(&c)[0];
    assert_eq!((line.ink, line.size), (Ink::Blue, start + 1));
}

#[test]
fn sizes_start_at_the_middle_and_step_between_the_lists_ends() {
    let (thick, text) = (steps(THICK.len()), steps(TEXT.len()));
    let mut c = canvas();
    assert_eq!(
        c.view().size,
        thick / 2,
        "the thickness starts at its list's middle"
    );
    for _ in 0..thick {
        c.size(true);
    }
    assert_eq!(c.view().size, thick - 1, "it stops at the thickest");
    assert_eq!(c.size(true), Did::Stay);
    for _ in 0..thick {
        c.size(false);
    }
    assert_eq!(c.view().size, 0, "and at the thinnest");
    assert_eq!(c.size(false), Did::Stay);
    c.tool(Tool::Text);
    assert_eq!(c.view().size, text / 2, "Text shows its own size");
    c.size(true);
    assert_eq!(c.view().size, text / 2 + 1);
    c.tool(Tool::Pen);
    assert_eq!(c.view().size, 0, "the thickness kept its step");
}

#[test]
fn a_shorter_list_keeps_each_step_or_stops_at_its_end() {
    let mut c = canvas();
    for _ in 0..THICK.len() {
        c.size(true);
    }
    let shorter = SnipFit {
        thick: THICK[..3].to_vec(),
        ..fit()
    };
    assert_eq!(c.fit(shorter.clone()), Did::Show);
    assert_eq!(c.view().size, steps(shorter.thick.len()) - 1);
    assert_eq!(c.fit(shorter), Did::Stay, "the same lists again");
}

#[test]
fn picking_a_tool_drops_a_gesture_under_way() {
    let mut c = with(Tool::Box);
    c.press(pt(5, 5), SAME);
    assert_eq!(c.tool(Tool::Line), Did::Show);
    assert_eq!(c.view().pending, None);
    c.press(pt(20, 20), SAME);
    assert!(drawn(&c).is_empty(), "the next click starts afresh");
    assert_eq!(
        c.tool(Tool::Line),
        Did::Show,
        "the same tool again drops it too"
    );
    assert_eq!(
        c.tool(Tool::Line),
        Did::Stay,
        "with nothing under way, it changes nothing"
    );
}

#[test]
fn edge_18_a_drawing_begun_outside_the_picture_is_clipped() {
    let far = far();
    let mut c = with(Tool::Line);
    c.press(pt(-40, 30), SAME);
    assert_eq!(
        c.view().pending,
        Some(pt(0, 30)),
        "the start moves to the left edge"
    );
    c.press(pt(50, 500), SAME);
    assert_eq!(marks(&c), [shape(Shape::Line, pt(0, 30), pt(50, far.y))]);

    let mut c = with(Tool::Pen);
    c.press(pt(10, -5), SAME);
    c.stroke(&[pt(400, 10), pt(20, 20)]);
    let path = vec![pt(10, 0), pt(far.x, 10), pt(20, 20)];
    assert_eq!(
        marks(&c),
        [Mark::Pen(path)],
        "every point of the path stays inside"
    );

    let mut c = with(Tool::Steps);
    c.press(pt(-3, 900), SAME);
    assert_eq!(
        marks(&c),
        [Mark::Step {
            at: pt(0, far.y),
            n: 1
        }]
    );

    let mut c = with(Tool::Box);
    two(&mut c, pt(-10, 5), pt(-20, 40));
    assert!(drawn(&c).is_empty(), "flat on the edge, so nothing");
}

#[test]
fn after_a_crop_drawings_and_crops_stay_inside_what_shows() {
    let mut c = with(Tool::Crop);
    cut(&mut c, pt(10, 10), pt(59, 49));
    c.tool(Tool::Steps);
    c.press(pt(0, 100), SAME);
    assert_eq!(
        marks(&c),
        [Mark::Step {
            at: pt(10, 49),
            n: 1
        }]
    );
    c.tool(Tool::Crop);
    cut(&mut c, pt(0, 0), pt(30, 30));
    let kept = Rect {
        x: 10,
        y: 10,
        w: 21,
        h: 21,
    };
    assert_eq!(
        c.view().crop.rect(),
        kept,
        "a crop keeps a part of what shows"
    );
}

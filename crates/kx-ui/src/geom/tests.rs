use super::*;
use kx_module_api::ports::BASE_DPI;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

fn at(x: i32, y: i32) -> Point {
    Point { x, y }
}

fn screen(r: Rect, dpi: u32, primary: bool) -> Screen {
    Screen {
        rect: r,
        dpi,
        primary,
    }
}

#[test]
fn the_bubble_sits_in_a_bottom_corner_of_the_work_area() {
    let work = rect(0, 0, 1920, 1020);
    assert_eq!(bubble(work, Corner::Right, 46, 12), at(1862, 962));
    assert_eq!(bubble(work, Corner::Left, 46, 12), at(12, 962));
    let second = rect(1920, 0, 1920, 1080);
    assert_eq!(bubble(second, Corner::Right, 46, 12), at(3782, 1022));
    let tiny = rect(0, 0, 20, 20);
    assert_eq!(
        bubble(tiny, Corner::Right, 46, 12),
        at(0, 0),
        "a screen smaller than the bubble"
    );
}

#[test]
fn odd_screens_never_panic() {
    let odd = rect(i32::MAX - 5, i32::MIN, -10, i32::MAX);
    let _ = bubble(odd, Corner::Left, i32::MAX, i32::MIN);
    let _ = bubble(odd, Corner::Right, i32::MIN, i32::MAX);
}

#[test]
fn a_box_is_covered_outwards_in_physical_pixels() {
    let a = Area {
        x: 10.2,
        y: 0.0,
        w: 20.5,
        h: 9.9,
    };
    assert_eq!(outward(&a, 1.5), rect(15, 0, 32, 15));
    assert_eq!(outward(&a, 1.0), rect(10, 0, 21, 10));
}

#[test]
fn a_bad_scale_or_box_gives_an_empty_box_and_never_panics() {
    let a = Area {
        x: f64::NAN,
        y: f64::INFINITY,
        w: -5.0,
        h: 1e30,
    };
    let r = outward(&a, 2.0);
    assert!(r.w >= 0 && r.h >= 0, "{r:?}");
    assert_eq!(outward(&Area::default(), f64::NAN), rect(0, 0, 0, 0));
}

#[test]
fn boxes_read_from_qml_json() {
    let boxes: Vec<Area> = serde_json::from_str(r#"[{"x":0,"y":0,"w":10,"h":5.5}]"#).unwrap();
    assert_eq!(boxes.len(), 1);
    assert!((boxes[0].h - 5.5).abs() < f64::EPSILON);
    assert!(serde_json::from_str::<Vec<Area>>(r#"[{"x":0}]"#).is_err());
}

#[test]
fn the_screen_under_a_point_else_the_primary_else_the_first() {
    let main = screen(rect(0, 0, 1920, 1040), BASE_DPI, true);
    let right = screen(rect(1920, 0, 2560, 1400), 144, false);
    let both = [main, right];
    assert_eq!(screen_at(at(2000, 10), &both), Some(&right));
    assert_eq!(screen_at(at(1919, 1039), &both), Some(&main));
    assert_eq!(
        screen_at(at(-50, -50), &both),
        Some(&main),
        "off every screen"
    );
    let second_first = [right, screen(rect(0, 0, 10, 10), BASE_DPI, false)];
    assert_eq!(screen_at(at(-1, 0), &second_first), Some(&right));
    assert_eq!(screen_at(at(0, 0), &[]), None);
}

#[test]
fn a_real_rounds_to_a_whole_pixel_and_saturates() {
    assert_eq!((whole(10.5), whole(-3.4), whole(899.6)), (11, -3, 900));
    assert_eq!(
        (whole(f64::NAN), whole(1e12), whole(-1e12)),
        (0, i32::MAX, i32::MIN)
    );
}

const GAP: [i32; 2] = [10, 8];

fn size(w: i32, h: i32) -> Size {
    Size { w, h }
}

#[test]
fn a_pop_up_sits_above_the_keyboard_else_below_else_at_the_top() {
    let work = rect(0, 0, 1920, 1040);
    let kb = rect(460, 600, 1000, 400);
    assert_eq!(beside(size(300, 100), 700, kb, work, GAP), at(700, 490));
    let high = rect(460, 60, 1000, 400);
    assert_eq!(
        beside(size(300, 100), 700, high, work, GAP),
        at(700, 470),
        "below"
    );
    let tall = rect(0, 60, 1920, 900);
    assert_eq!(
        beside(size(300, 100), 700, tall, work, GAP),
        at(700, 8),
        "no room: the top"
    );
}

#[test]
fn a_pop_up_stays_inside_the_work_area() {
    let work = rect(1920, 0, 1920, 1040);
    let kb = rect(1920, 600, 1000, 400);
    assert_eq!(
        beside(size(300, 100), 1800, kb, work, GAP),
        at(1928, 490),
        "the left edge"
    );
    assert_eq!(
        beside(size(300, 100), 3800, kb, work, GAP),
        at(3532, 490),
        "the right edge"
    );
    let huge = size(5000, 5000);
    assert_eq!(
        beside(huge, 3000, kb, work, GAP),
        at(1928, 8),
        "too big: the top-left"
    );
}

#[test]
fn with_no_keyboard_a_pop_up_sits_at_the_bottom_centre() {
    let work = rect(0, 0, 1920, 1040);
    assert_eq!(home(size(400, 60), work, 8), at(760, 972));
    assert_eq!(home(size(4000, 6000), work, 8), at(8, 8), "too big");
}

#[test]
fn a_menu_opens_up_and_right_from_its_anchor_inside_the_work_area() {
    let work = rect(0, 0, 1920, 1032);
    assert_eq!(
        menu(size(200, 90), at(600, 500), work, 8),
        at(600, 410),
        "its bottom-left at an anchor inside"
    );
    assert_eq!(
        menu(size(200, 90), at(1700, 1050), work, 8),
        at(1700, 934),
        "taskbar at the bottom"
    );
    assert_eq!(
        menu(size(200, 90), at(1900, 1050), work, 8),
        at(1712, 934),
        "the right edge"
    );
    let below_top_bar = rect(0, 48, 1920, 1032);
    assert_eq!(
        menu(size(200, 90), at(1700, 20), below_top_bar, 8),
        at(1700, 56),
        "taskbar at the top"
    );
}

#[test]
fn the_nearest_screen_holds_a_point_outside_every_work_area() {
    let main = screen(rect(0, 0, 1920, 1032), BASE_DPI, true);
    let right = screen(rect(1920, 0, 2560, 1392), 144, false);
    let both = [main, right];
    assert_eq!(
        screen_near(at(3000, 1420), &both),
        Some(&right),
        "the second screen's taskbar"
    );
    assert_eq!(screen_near(at(100, 1060), &both), Some(&main));
    assert_eq!(screen_near(at(2000, 10), &both), Some(&right), "inside");
    assert_eq!(screen_near(at(0, 0), &[]), None);
}

#[test]
fn the_main_screen_is_the_primary_else_the_first_and_a_box_has_a_centre() {
    let side = screen(rect(1920, 0, 2560, 1392), 144, false);
    let main = screen(rect(0, 0, 1920, 1032), BASE_DPI, true);
    assert_eq!(main_screen(&[side, main]), Some(&main));
    assert_eq!(main_screen(&[side]), Some(&side));
    assert_eq!(main_screen(&[]), None);
    assert_eq!(centre(rect(460, 600, 1000, 400)), at(960, 800));
    assert_eq!(
        centre(rect(i32::MAX, i32::MAX, i32::MAX, 2)),
        at(i32::MAX, i32::MAX),
        "saturates"
    );
}

#[test]
fn odd_pop_ups_never_panic() {
    let odd = rect(i32::MAX - 5, i32::MIN, -10, i32::MAX);
    let _ = beside(
        size(i32::MAX, i32::MIN),
        i32::MIN,
        odd,
        odd,
        [i32::MAX, i32::MIN],
    );
    let _ = home(size(i32::MIN, i32::MAX), odd, i32::MAX);
    let _ = menu(
        size(i32::MAX, i32::MAX),
        at(i32::MIN, i32::MAX),
        odd,
        i32::MIN,
    );
    let _ = screen_near(at(i32::MIN, i32::MAX), &[screen(odd, 0, false)]);
}

proptest::proptest! {
    #[test]
    fn a_pop_up_that_fits_stays_inside_the_work_area(
        x in -4000i32..4000, y in -4000i32..4000, w in 400i32..4000, h in 400i32..4000,
        pw in 1i32..300, ph in 1i32..300, ax in -8000i32..8000, ay in -8000i32..8000, edge in 0i32..50,
    ) {
        let (work, pop) = (rect(x, y, w, h), size(pw, ph));
        let inside = |p: Point| {
            p.x >= x + edge && p.y >= y + edge && p.x + pw <= x + w - edge && p.y + ph <= y + h - edge
        };
        proptest::prop_assert!(inside(menu(pop, at(ax, ay), work, edge)));
        proptest::prop_assert!(inside(home(pop, work, edge)));
        proptest::prop_assert!(inside(beside(pop, ax, rect(ax, ay, 100, 100), work, [10, edge])));
    }
}

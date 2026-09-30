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

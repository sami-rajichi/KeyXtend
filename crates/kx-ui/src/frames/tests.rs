use super::*;

/// The keyboard window and another of ours, by Qt's window handle.
const KB: usize = 1;
const TIP: usize = 2;

/// A presented frame whose steps follow `from`, `gap` µs apart.
fn frame(from: i64, gap: i64) -> Frame {
    Frame {
        update: from,
        begin: from + gap,
        sync: from + 2 * gap,
        rendered: from + 3 * gap,
        swap: from + 4 * gap,
        done: from + 5 * gap,
    }
}

#[test]
fn a_frame_line_splits_its_cpu_time_into_steps() {
    let mut log = Log::new();
    let (line, press) = log.framed(KB, frame(1_000, 10));
    assert_eq!(press, None, "no press is open");
    assert_eq!(
        line,
        FrameTimes {
            at_us: 1_000,
            polish_us: Some(10),
            wait_us: Some(10),
            render_us: Some(10),
            present_us: Some(10),
            busy_us: Some(50),
        }
    );
}

#[test]
fn a_frame_drawn_without_an_update_request_starts_at_its_begin() {
    let mut log = Log::new();
    let f = Frame {
        update: 0,
        ..frame(1_000, 10)
    };
    let (line, _) = log.framed(KB, f);
    assert_eq!((line.at_us, line.polish_us), (1_010, None));
    assert_eq!(line.busy_us, Some(40));
}

#[test]
fn after_typing_the_press_line_waits_for_its_frame() {
    let mut log = Log::new();
    assert_eq!(log.pressed(KB, 100, 150), None);
    for (mark, at) in [(Mark::Start, 200), (Mark::Sent, 900), (Mark::End, 1_200)] {
        assert_eq!(
            log.mark(mark, at),
            None,
            "{mark:?}: the frame is not out yet"
        );
    }
    let (_, press) = log.framed(KB, frame(5_000, 100));
    let p = press.expect("the press frame ends the line");
    assert_eq!((p.at_us, p.qt_us, p.start_us), (100, Some(50), Some(100)));
    assert_eq!((p.sent_us, p.end_us), (Some(800), Some(1_100)));
    assert_eq!(
        (p.update_us, p.begin_us, p.sync_us),
        (Some(4_900), Some(5_000), Some(5_100))
    );
    assert_eq!(
        (p.rendered_us, p.swap_us, p.done_us),
        (Some(5_200), Some(5_300), Some(5_400))
    );
}

#[test]
fn a_press_drawn_first_ends_its_line_when_its_action_ends() {
    let mut log = Log::new();
    log.pressed(KB, 100, 150);
    let (_, press) = log.framed(KB, frame(300, 10));
    assert_eq!(press, None, "the action has not run");
    log.mark(Mark::Start, 400);
    log.mark(Mark::Sent, 500);
    let p = log.mark(Mark::End, 600).expect("the end ends the line");
    assert_eq!(
        (p.swap_us, p.start_us, p.end_us),
        (Some(240), Some(300), Some(500))
    );
}

#[test]
fn only_a_presented_frame_of_the_pressed_window_after_the_press_is_its_frame() {
    let mut log = Log::new();
    log.pressed(KB, 100, 150);
    log.mark(Mark::End, 200);
    assert_eq!(log.framed(TIP, frame(300, 10)).1, None, "another window");
    assert_eq!(
        log.framed(KB, frame(140, 1)).1,
        None,
        "asked before the press"
    );
    let skipped = Frame {
        swap: 0,
        ..frame(400, 10)
    };
    assert_eq!(log.framed(KB, skipped).1, None, "never presented");
    let p = log
        .framed(KB, frame(500, 10))
        .1
        .expect("the next frame is it");
    assert_eq!(p.update_us, Some(400));
}

#[test]
fn a_new_press_brings_out_the_last_one_with_what_it_has() {
    let mut log = Log::new();
    log.pressed(TIP, 0, 150);
    let p = log
        .pressed(KB, 1_000, 1_050)
        .expect("the last press comes out");
    assert_eq!(
        (p.at_us, p.qt_us, p.start_us, p.swap_us),
        (150, None, None, None),
        "with no message seen, times count from Qt's press"
    );
}

#[test]
fn marks_outside_a_press_and_twice_in_one_change_nothing() {
    let mut log = Log::new();
    assert_eq!(log.mark(Mark::Sent, 50), None, "no press is open");
    log.pressed(KB, 100, 150);
    log.mark(Mark::Start, 200);
    log.mark(Mark::Start, 300);
    log.mark(Mark::End, 400);
    let p = log.framed(KB, frame(500, 10)).1.expect("a line");
    assert_eq!((p.start_us, p.sent_us), (Some(100), None));
}

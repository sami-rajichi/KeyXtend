//! The dictation bar's wave (design/voice-bar.css `.wave`): its travel, its amounts and the plain meter.

use super::*;

#[test]
fn the_voice_wave_travels_as_the_mock_up_and_stops_with_reduced_motion() {
    let m = load().expect("motion.toml loads");
    let linear = [0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    let travel = Timed {
        ms: 900,
        bez: linear,
    };
    assert_eq!(m.full.moves["wave"], travel, "sin(s * 7): 2π / 7 s a turn");
    assert_eq!(m.reduced.moves["wave"].ms, 0, "a plain meter instead");
    let a = m.full.amount;
    assert_eq!(
        (a.wave_floor, a.wave_step),
        (0.4, 0.9),
        ".4 + .6 * |sin(n * .9 …)|"
    );
    let bad = text().replacen("wave_floor = 0.4", "wave_floor = 1.5", 1);
    assert!(parse(&bad).expect_err("a share").contains("wave_floor"));
    let bad = text().replacen("wave_step = 0.9", "wave_step = nan", 1);
    assert!(parse(&bad).expect_err("a number").contains("wave_step"));
}

#[test]
fn compares_writing_line_slides_as_the_mock_up_and_stands_still_with_reduced_motion() {
    let m = load().expect("motion.toml loads");
    let run = &m.full.moves["run"];
    assert_eq!(run.ms, 1000, ".prog.run: --shimmer-ms each way");
    assert_eq!(
        run.bez, m.full.moves["led"].bez,
        "ease-in-out, as the key's pulse"
    );
    assert_eq!(m.reduced.moves["run"].ms, 0, ".still: it stands still");
}

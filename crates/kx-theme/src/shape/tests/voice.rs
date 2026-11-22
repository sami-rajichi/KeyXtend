//! The dictation bar's and the Voice panel's own sizes (design/voice-bar.html `--voice-*`), and the bar's padding.

use super::pad::refused_in;
use super::*;

#[test]
fn the_voice_bar_takes_the_mock_up_sizes() {
    let s = load().expect("shape.toml loads");
    let v = &s.voice;
    assert_eq!(
        v.bar_px,
        [500.0, 650.0],
        "--voice-bar-w, and 10 past --voice-bar-w-text for each service's whole name"
    );
    assert_eq!(
        v.wave_px,
        [4.0, 3.0, 3.0, 24.0],
        ".wave b, frame(): 3 + 21 at full level"
    );
    assert_eq!(v.text_lines, 2, "--voice-text-lines");
    assert_eq!(
        (s.caption.pad_px, s.caption.inset_px),
        (3.0, 2.0),
        ".vbar padding, .st padding-inline"
    );
    let room = s.caption.bar_px - 2.0 * s.caption.pad_px;
    assert!(v.wave_px[3] <= room, "the wave fits in the bar");
}

#[test]
fn the_voice_panel_takes_the_mock_up_sizes() {
    let v = load().expect("shape.toml loads").voice;
    assert_eq!(v.progress_px, 4.0, "--voice-progress-px");
    assert_eq!(v.switch_px, [40.0, 22.0, 12.0], "--voice-switch-px");
    assert_eq!(v.value_px, 92.0, "--voice-value-px");
    assert_eq!(v.col_px, 84.0, "--voice-col-px");
    assert_eq!(v.panel_px, 600.0, ".vp: 600 wide");
    assert_eq!(v.fact_px, 200.0, "voice-cloud.css --voice-fact-px");
    assert_eq!(v.pick_across, 2, "Compare's picks pair, two to a row");
    assert_eq!(
        v.pick_name_px, 96.0,
        "a cut pick's name keeps about 15 characters"
    );
    assert_eq!(
        v.chip_name_px, 128.0,
        "the bar's chip shows about 20 characters whole"
    );
    assert_eq!(v.wave_room_px, 48.0, "the wave keeps about seven bars");
    let [width, gap, _, _] = v.wave_px;
    assert!(v.wave_room_px >= width + gap, "a bar and its gap at least");
    assert_eq!(
        v.run_share,
        [0.4, 0.5],
        ".prog.run i: 40 %, .still: opacity .5"
    );
}

#[test]
fn an_odd_voice_size_is_refused_by_name() {
    refused_in(&[
        (
            "voice",
            "text_lines = 2",
            "text_lines = 0",
            "voice.text_lines",
        ),
        ("voice", "bar_px = [500.0", "bar_px = [0.0", "voice.bar_px"),
        (
            "voice",
            "bar_px = [500.0, 650.0]",
            "bar_px = [500.0, 400.0]",
            "voice.bar_px",
        ),
        (
            "voice",
            "wave_px = [4.0, 3.0",
            "wave_px = [4.0, -1.0",
            "voice.wave_px",
        ),
        (
            "voice",
            "wave_px = [4.0, 3.0, 3.0, 24.0]",
            "wave_px = [4.0, 3.0, 30.0, 24.0]",
            "voice.wave_px",
        ),
        (
            "voice",
            "switch_px = [40.0, 22.0, 12.0]",
            "switch_px = [40.0, 22.0, 30.0]",
            "voice.switch_px",
        ),
        ("voice", "col_px = 84.0", "col_px = nan", "voice.col_px"),
        (
            "caption",
            "inset_px = 2.0",
            "inset_px = -1.0",
            "caption.inset_px",
        ),
    ]);
}

#[test]
fn an_odd_panel_size_is_refused_by_name() {
    refused_in(&[
        (
            "voice",
            "panel_px = 600.0",
            "panel_px = 0.0",
            "voice.panel_px",
        ),
        ("voice", "fact_px = 200.0", "fact_px = 0.0", "voice.fact_px"),
        (
            "voice",
            "pick_across = 2",
            "pick_across = 0",
            "voice.pick_across",
        ),
        (
            "voice",
            "pick_name_px = 96.0",
            "pick_name_px = 0.0",
            "voice.pick_name_px",
        ),
        (
            "voice",
            "run_share = [0.4, 0.5]",
            "run_share = [0.4, 1.5]",
            "voice.run_share",
        ),
        (
            "voice",
            "chip_name_px = 128.0",
            "chip_name_px = 0.0",
            "voice.chip_name_px",
        ),
        (
            "voice",
            "wave_room_px = 48.0",
            "wave_room_px = -1.0",
            "voice.wave_room_px",
        ),
    ]);
}

//! Where a passing language's time went, read from the copy's stage and typed lines by their stamps, numbers only.

use kx_ipc::gate::{LOAD, OPEN, PIECE, SEND, SOUND_MS, STAGE, STAGE_LINE, START, STOP, TOOK_MS};

use kx_mod_voice::TYPED_LINE;

use super::super::stages::{self, Piece, Timing};
use crate::budget::stamp_us;

/// A line stamped `ms` past 10:00:00 that holds `rest`.
fn at(ms: u32, rest: &str) -> String {
    let (s, f) = (ms / 1000, ms % 1000);
    format!("2026-11-20T10:00:{s:02}.{f:03}000Z DEBUG keyxtend::played::stage: {rest}")
}

/// The stage line for `stage`, with `more` after it.
fn stage(stage: &str, more: &str) -> String {
    format!("{STAGE_LINE} {STAGE}={stage} {more}")
}

/// The timing `lines` give, from 10:00:00.
fn read(lines: &[String]) -> Result<Timing, String> {
    let since = stamp_us("2026-11-20T10:00:00.000000Z").expect("a stamp");
    stages::read(&lines.join("\n"), since)
}

/// The voice module's typed line, stamped `ms`.
fn typed(ms: u32) -> String {
    at(ms, TYPED_LINE)
}

#[test]
fn a_local_dictations_time_reads_from_its_stage_lines() {
    let lines = [
        at(0, &stage(START, "")),
        at(400, &stage(OPEN, "")),
        at(6000, &stage(STOP, "")),
        at(7200, &stage(PIECE, "index=0")),
        at(7250, &stage(LOAD, &format!("{TOOK_MS}=900"))),
        typed(7300),
    ];
    let want = Timing {
        start_open_ms: Some(400),
        load_ms: Some(900),
        pieces: vec![Piece {
            ms: Some(1200),
            sound_ms: None,
        }],
        stop_text_ms: Some(1300),
    };
    assert_eq!(read(&lines), Ok(want));
}

#[test]
fn a_cloud_dictations_pieces_count_from_the_send_then_from_each_other() {
    let lines = [
        at(0, &stage(START, "")),
        at(300, &stage(OPEN, "")),
        at(5000, &stage(STOP, "")),
        at(5200, &stage(SEND, "")),
        at(6000, &stage(PIECE, &format!("index=0 {SOUND_MS}=6000"))),
        at(6500, &stage(PIECE, &format!("index=1 {SOUND_MS}=3000"))),
        typed(6600),
    ];
    let t = read(&lines).expect("a timing");
    let piece = |ms, sound_ms| Piece {
        ms: Some(ms),
        sound_ms: Some(sound_ms),
    };
    assert_eq!(t.pieces, [piece(800, 6000), piece(500, 3000)]);
    assert_eq!(
        (t.load_ms, t.stop_text_ms),
        (None, Some(1600)),
        "no model loads"
    );
}

#[test]
fn a_piece_before_stop_adds_nothing_and_a_missing_stage_reads_none() {
    let lines = [
        at(0, &stage(START, "")),
        at(3000, &stage(PIECE, "index=0")),
        at(5000, &stage(STOP, "")),
    ];
    let t = read(&lines).expect("a timing");
    assert_eq!(t.start_open_ms, None, "no microphone line");
    assert_eq!(t.pieces[0].ms, Some(0), "written while listening");
    assert_eq!(t.stop_text_ms, None, "no text went");
    let early = read(&[at(1000, &stage(PIECE, "index=0"))]).expect("a timing");
    assert_eq!(early.pieces[0].ms, None, "nothing it counts from");
}

#[test]
fn only_this_languages_stage_and_typed_lines_count() {
    let mut lines = vec![
        "2026-11-20T09:59:59.000000Z DEBUG x: voice gate build: a stage stage=stop".to_owned(),
        at(0, &stage(START, "")),
        at(100, "INFO kx_mod_voice: another line stage=open"),
        at(500, &stage(OPEN, "")),
    ];
    lines.push(at(900, &stage(STOP, "")));
    lines.push(typed(1000));
    let t = read(&lines).expect("a timing");
    assert_eq!((t.start_open_ms, t.stop_text_ms), (Some(500), Some(100)));
}

#[test]
fn an_odd_stage_line_is_refused_without_its_value() {
    for bad in [
        stage("whisper", ""),
        stage(LOAD, ""),
        stage(LOAD, &format!("{TOOK_MS}=12ab")),
        stage(PIECE, &format!("{SOUND_MS}=-4")),
    ] {
        let e = read(&[at(0, &bad)]).expect_err(&bad);
        assert!(!e.contains("whisper") && !e.contains("12ab"), "{e}");
    }
}

#[test]
fn a_timing_prints_numbers_only() {
    let t = Timing {
        start_open_ms: Some(1),
        load_ms: None,
        pieces: vec![Piece {
            ms: Some(2),
            sound_ms: Some(3),
        }],
        stop_text_ms: Some(4),
    };
    let v = serde_json::to_value(&t).expect("a value");
    let text = v.to_string();
    assert!(only_keys(&v), "{text}");
}

/// True when every string in `v` is an object's key, never a value.
fn only_keys(v: &serde_json::Value) -> bool {
    match v {
        serde_json::Value::String(_) => false,
        serde_json::Value::Array(a) => a.iter().all(only_keys),
        serde_json::Value::Object(o) => o.values().all(only_keys),
        _ => true,
    }
}

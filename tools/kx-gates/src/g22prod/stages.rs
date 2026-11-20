//! Where a passing language's time went, from the copy's log: the gate build's stage lines and the voice module's
//! typed line, by their stamps, give start to microphone open, the model's load, each piece and Stop to text, in ms.

use kx_ipc::gate::{LOAD, OPEN, PIECE, SEND, SOUND_MS, STAGE, START, STOP, TOOK_MS};
use kx_module_api::US_PER_MS;
use serde::Serialize;

use crate::budget::stamp_us;
use crate::hooklog::{read_since, value_of};

/// What a line of the log marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Start,
    Open,
    Stop,
    Send,
    Load { took_ms: u64 },
    Piece { sound_ms: Option<u64> },
    Typed,
}

/// One piece: its wait from Stop, the send or the piece before, whichever came last, and its sound when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct Piece {
    /// Its wait, in ms; 0 when written while listening, none with nothing to count from.
    pub(super) ms: Option<u64>,
    /// Its sound's length, in ms.
    pub(super) sound_ms: Option<u64>,
}

/// Where a language's time went, in ms; none for a stage the log does not show.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(super) struct Timing {
    /// Start to the microphone open.
    pub(super) start_open_ms: Option<u64>,
    /// The model's load, as its job timed it; a cloud line loads none.
    pub(super) load_ms: Option<u64>,
    /// Each piece, in order.
    pub(super) pieces: Vec<Piece>,
    /// Stop to the text going to the keyboard.
    pub(super) stop_text_ms: Option<u64>,
}

/// The lines a timing reads, as `[g22_product]` names them.
pub(super) struct Lines<'a> {
    /// The gate build's stage line.
    pub(super) stage: &'a str,
    /// The voice module's typed line.
    pub(super) typed: &'a str,
}

/// The timing of `log`'s lines stamped at or after `since_us`; an error for a stage line this run cannot read.
pub(super) fn read(log: &str, since_us: i64, lines: &Lines) -> Result<Timing, String> {
    let marks = read_since(log, since_us, |l| mark_of(l, lines))?;
    Ok(timing(&marks))
}

/// The stamp and mark of `line`; none for another line, and an error that never repeats a value it cannot read.
fn mark_of(line: &str, lines: &Lines) -> Result<Option<(i64, Mark)>, String> {
    let Some(at) = stamp_us(line) else {
        return Ok(None);
    };
    if line.contains(lines.typed) {
        return Ok(Some((at, Mark::Typed)));
    }
    if !line.contains(lines.stage) {
        return Ok(None);
    }
    let odd = |what: &str| format!("{}: {what}", lines.stage);
    let number = |name: &str| match value_of(line, name) {
        None => Ok(None),
        Some(v) => v
            .parse()
            .map(Some)
            .map_err(|_| odd(&format!("{name} is no number"))),
    };
    let mark = match value_of(line, STAGE) {
        Some(START) => Mark::Start,
        Some(OPEN) => Mark::Open,
        Some(STOP) => Mark::Stop,
        Some(SEND) => Mark::Send,
        Some(LOAD) => Mark::Load {
            took_ms: number(TOOK_MS)?.ok_or_else(|| odd(&format!("no {TOOK_MS}")))?,
        },
        Some(PIECE) => Mark::Piece {
            sound_ms: number(SOUND_MS)?,
        },
        _ => return Err(odd("no stage this run knows")),
    };
    Ok(Some((at, mark)))
}

/// The timing of `marks`, stamped in µs, in the log's order.
fn timing(marks: &[(i64, Mark)]) -> Timing {
    let first = |want: Mark| marks.iter().find(|(_, m)| *m == want).map(|(at, _)| *at);
    let (start, open, stop) = (first(Mark::Start), first(Mark::Open), first(Mark::Stop));
    let after_stop = |&&(at, m): &&(i64, Mark)| m == Mark::Typed && stop.is_some_and(|s| at >= s);
    let typed = marks.iter().find(after_stop).map(|(at, _)| *at);
    let load_ms = marks.iter().find_map(|(_, m)| match m {
        Mark::Load { took_ms } => Some(*took_ms),
        _ => None,
    });
    Timing {
        start_open_ms: span(start, open),
        load_ms,
        pieces: pieces(marks, first(Mark::Send).or(stop)),
        stop_text_ms: span(stop, typed),
    }
}

/// Each piece's wait in `marks`, counted from `from`, then from the piece before.
fn pieces(marks: &[(i64, Mark)], from: Option<i64>) -> Vec<Piece> {
    let mut last = from;
    let each = |&(at, m): &(i64, Mark)| {
        let Mark::Piece { sound_ms } = m else {
            return None;
        };
        let ms = last.map(|l| span(Some(l), Some(at)).unwrap_or(0));
        last = Some(last.map_or(at, |l| l.max(at)));
        Some(Piece { ms, sound_ms })
    };
    marks.iter().filter_map(each).collect()
}

/// The ms from `from` to `to`; none when either is missing or `to` came first.
fn span(from: Option<i64>, to: Option<i64>) -> Option<u64> {
    let us = to?.checked_sub(from?)?;
    u64::try_from(us).ok().map(|us| us / u64::from(US_PER_MS))
}

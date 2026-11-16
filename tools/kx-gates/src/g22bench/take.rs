//! One take: a dictation of the gate worker's clip through the product's speech host, stopped once the job has heard
//! the clip and its tail, counted by its level notes, which follow the sound even when a capped job plays it late. It
//! keeps the job's own load and memory, the bench's own times after Stop and the words wrong, never the text.
#![cfg(windows)]

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use kx_mod_voice::score::Wrong;
use kx_module_api::ports::{Speech, SpeechAsk, SpeechDone, SpeechNote, SpeechRun, SpeechStage};
use kx_module_api::{Secret, Zeroizing};

use crate::g22cfg::fewest;

/// Why a take failed without a fault of the job's.
const SHUT: &str = "the microphone did not open within the wait";
const SLOW: &str = "the clip did not play within the wait";
const EARLY: &str = "the run ended before Stop";
const LATE: &str = "no text within the wait";
const SILENT: &str = "the run told nothing more";
const WIPED: &str = "the job ended and wiped the recording";

/// What a take listens for and judges by.
pub(super) struct Script<'a> {
    /// The level notes to hear before Stop: the clip and its tail, at `[voice] level_hz` a second of sound.
    pub(super) levels: u32,
    /// The longest wait for them, and after Stop for every piece.
    pub(super) within: Duration,
    /// The clip's sentence, for the words-wrong count only.
    pub(super) sentence: &'a str,
    /// Other readings of it the count accepts, where the credited spelling is a quirk.
    pub(super) also: &'a [String],
    /// The clip's language, whose rules count the words.
    pub(super) lang: &'a str,
}

/// What one take measured.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Take {
    /// The model's load, in ms, as the job timed it; 0 when it was loaded already.
    pub(super) load_ms: u32,
    /// Stop to the first piece's text, in ms; `None` when no piece came.
    pub(super) first_ms: Option<u32>,
    /// Stop to the end of the last piece, in ms, as the bench heard them.
    pub(super) total_ms: u32,
    /// The job's most memory, in MB.
    pub(super) memory_mb: u32,
    /// Words wrong of the sentence; `None` for a language without rules.
    pub(super) wrong: Option<Wrong>,
}

/// A note, and when the bench heard it.
type Heard = (Instant, SpeechNote);

/// What a take gathers: whether the microphone opened, the levels heard, when Stop went and the first piece came, the
/// pieces and the end.
#[derive(Default)]
struct Gathered {
    opened: bool,
    levels: u32,
    stopped: Option<Instant>,
    first: Option<Instant>,
    pieces: Vec<Secret>,
    done: Option<(Instant, SpeechDone)>,
}

/// Runs one take of `ask` on `speech` as `script` says; the run is closed on every path.
pub(super) fn take(speech: &dyn Speech, ask: SpeechAsk, script: &Script) -> Result<Take, String> {
    let (tx, rx) = mpsc::channel();
    let tell = Box::new(move |note| drop(tx.send((Instant::now(), note))));
    let mut run = speech.open(ask, tell).map_err(|e| e.to_string())?;
    let got = listen(&rx, &mut *run, script);
    run.close();
    measure(&got?, script)
}

/// Listens until the job heard `script.levels` levels, stops, and gathers every piece.
fn listen(
    rx: &Receiver<Heard>,
    run: &mut dyn SpeechRun,
    script: &Script,
) -> Result<Gathered, String> {
    let mut got = Gathered::default();
    let heard = |g: &Gathered| g.opened && g.levels >= script.levels;
    gather(rx, Instant::now() + script.within, &mut got, heard)?;
    // Takes what came with the last level, so a run that already ended is never stopped.
    gather(rx, Instant::now(), &mut got, |_| false)?;
    if got.done.is_some() {
        return Err(EARLY.into());
    }
    if !heard(&got) {
        return Err((if got.opened { SLOW } else { SHUT }).into());
    }
    let stopped = Instant::now();
    got.stopped = Some(stopped);
    run.stop().map_err(|e| e.to_string())?;
    gather(rx, stopped + script.within, &mut got, |_| false)?;
    Ok(got)
}

/// The numbers `got` holds, its pieces counted against the closest of `script`'s readings; the caller drops them, wiped.
fn measure(got: &Gathered, script: &Script) -> Result<Take, String> {
    let ((end, done), stopped) = (got.done.ok_or(LATE)?, got.stopped.ok_or(LATE)?);
    let since = |at: Instant| ms(at.saturating_duration_since(stopped));
    let heard = Zeroizing::new(
        got.pieces
            .iter()
            .map(|p| p.expose().as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let wrong = fewest(script.sentence, script.also, &heard, script.lang);
    Ok(Take {
        load_ms: done.load_ms,
        first_ms: got.first.map(since),
        total_ms: since(end),
        memory_mb: done.memory_mb,
        wrong,
    })
}

/// Gathers notes into `got` until `until`, `enough` or the end; a fault that ends the run, or a wipe, fails the take.
fn gather(
    rx: &Receiver<Heard>,
    until: Instant,
    got: &mut Gathered,
    enough: impl Fn(&Gathered) -> bool,
) -> Result<(), String> {
    while got.done.is_none() && !enough(got) {
        let left = until.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok((at, note)) => hear(got, at, note)?,
            Err(RecvTimeoutError::Timeout) => return Ok(()),
            Err(RecvTimeoutError::Disconnected) => return Err(SILENT.into()),
        }
    }
    Ok(())
}

/// Adds `note`, heard `at`, to `got`.
fn hear(got: &mut Gathered, at: Instant, note: SpeechNote) -> Result<(), String> {
    match note {
        SpeechNote::Stage(SpeechStage::Open) => got.opened = true,
        SpeechNote::Level(_) => got.levels = got.levels.saturating_add(1),
        SpeechNote::Piece { text, .. } => {
            got.first.get_or_insert(at);
            got.pieces.push(text);
        }
        SpeechNote::Done(done) => got.done = Some((at, done)),
        SpeechNote::Fault(e) if e.ends() => return Err(e.to_string()),
        SpeechNote::Wiped => return Err(WIPED.into()),
        _ => {}
    }
    Ok(())
}

/// `d` in whole ms, held at the top of a `u32`.
fn ms(d: Duration) -> u32 {
    u32::try_from(d.as_millis()).unwrap_or(u32::MAX)
}

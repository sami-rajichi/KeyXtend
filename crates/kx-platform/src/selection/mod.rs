//! The OS-neutral half of the selection watch (ADR-0041): its thread and pacing, each job's deadline, the hung-app rule
//! and the read rules every OS shares. An adapter finds the focused element and says what it shows (`SelProbe`), and
//! its change handlers call `Shared::moved`.

mod inbox;
mod pace;
mod run;

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use kx_module_api::Redacted;
use kx_module_api::ports::{SelError, SelKind, SelSpot, same_box};

pub use inbox::{Inbox, Shared};
use run::Ask;
pub use run::{Job, Tell, spawn};

/// Whose an element is, which the change handlers ask on any thread.
pub trait Ours {
    /// One element, as the OS hands it over.
    type El;

    /// Whether `el` shows in our own process.
    fn ours(&self, el: &Self::El) -> bool;
}

/// What an adapter asks the OS on the watch's thread, each call by its deadline `by`, never logging what it reads.
pub trait SelProbe: Ours {
    /// The element with the keyboard focus, in any app.
    ///
    /// # Errors
    /// `TimedOut` past `by`, `Refused` when the OS refuses.
    fn focused(&self, by: Instant) -> Result<Self::El, SelError>;

    /// What `el`'s box and selection show: its kind, whether its selection holds text, and its boxes; never its text.
    ///
    /// # Errors
    /// `TimedOut` past `by`, `Refused` when the app refuses.
    fn spot(&self, el: &Self::El, by: Instant) -> Result<SelSpot, SelError>;

    /// Whether `el` is box `box_id` and its selection still holds text; false on any doubt.
    fn still(&self, el: &Self::El, box_id: u64, by: Instant) -> bool;

    /// At most `max_chars` characters of `el`'s selection from its start; the watch asks only the box the read named,
    /// when it may be read and holds a selection.
    ///
    /// # Errors
    /// `TimedOut` past `by`, `Refused` when the app refuses.
    fn read(
        &self,
        el: &Self::El,
        max_chars: u16,
        by: Instant,
    ) -> Result<Redacted<String>, SelError>;

    /// Whether the clipboard holds any format, reading none; it never waits on another app.
    fn clip_holds(&self) -> bool;
}

/// Answers `job` on the watch's thread: one past its deadline makes no OS call, and focus in our own process is never
/// looked at or read.
pub fn serve<P: SelProbe>(probe: &P, job: Job) {
    // The token lives until the answer is sent, so a run that gave up on this job sees the thread busy until then.
    let Job { ask, held: _held } = job;
    match ask {
        Ask::Look { by, reply } => {
            let _ = reply.send(look(probe, by));
        }
        Ask::Still { box_id, by, reply } => {
            let _ = reply.send(still(probe, box_id, by));
        }
        Ask::Read {
            box_id,
            max_chars,
            by,
            reply,
        } => {
            let _ = reply.send(read(probe, box_id, max_chars, by));
        }
        Ask::Clip { by, reply } => {
            let _ = reply.send(late(by).is_ok() && probe.clip_holds());
        }
    }
}

/// The focused element, unless `by` has passed; `None` when it shows in our own process.
fn theirs<P: SelProbe>(probe: &P, by: Instant) -> Result<Option<P::El>, SelError> {
    late(by)?;
    let el = probe.focused(by)?;
    Ok((!probe.ours(&el)).then_some(el))
}

/// The focused box's spot; `None` in our own process, whose focus keeps the last box.
fn look<P: SelProbe>(probe: &P, by: Instant) -> Result<Option<SelSpot>, SelError> {
    match theirs(probe, by)? {
        Some(el) => probe.spot(&el, by).map(Some),
        None => Ok(None),
    }
}

/// Whether box `box_id` still has focus and its selection holds text; never in our own process.
fn still<P: SelProbe>(probe: &P, box_id: u64, by: Instant) -> bool {
    matches!(theirs(probe, by), Ok(Some(el)) if probe.still(&el, box_id, by))
}

/// Box `box_id`'s selected text, its box checked afresh in the same job: refused in a `Closed` box, another box or our
/// own process, and empty at a bare caret, with no text call either way.
fn read<P: SelProbe>(
    probe: &P,
    box_id: u64,
    max_chars: u16,
    by: Instant,
) -> Result<Redacted<String>, SelError> {
    let el = theirs(probe, by)?.ok_or(SelError::Closed)?;
    let spot = probe.spot(&el, by)?;
    if spot.kind == SelKind::Closed || !same_box(spot.box_id, box_id) {
        return Err(SelError::Closed);
    }
    if !spot.selected {
        return Ok(Redacted::new(String::new()));
    }
    late(by)?;
    probe.read(&el, max_chars, by)
}

/// `TimedOut` once `by` has passed.
///
/// # Errors
/// `TimedOut` past `by`.
pub fn late(by: Instant) -> Result<(), SelError> {
    if Instant::now() > by {
        Err(SelError::TimedOut)
    } else {
        Ok(())
    }
}

/// Locks `m`, taking the value even when a panicking thread poisoned it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests;

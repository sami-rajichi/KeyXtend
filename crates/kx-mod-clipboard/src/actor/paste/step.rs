//! One insert step: the next entry is read first, and only then is its join written, the entry waiting behind it;
//! an entry whose content is gone gets no join. A failed write stops with `Busy` or `PasteFailed` (Ruling 41).

use kx_module_api::ports::{ClipError, ClipPart, PanelNote, PasteAsk, SealedError};
use kx_module_api::{Redacted, keys};

use super::super::Actor;
use super::Paste;
use crate::insert::{Insert, Step, paste_parts};

/// An entry read and written next, behind the join just written.
#[derive(Debug)]
struct Ready {
    step: Step,
    parts: Vec<ClipPart>,
}

/// An insert's steps left, the entry waiting behind its join, and whether an entry went out yet.
#[derive(Debug)]
pub(in crate::actor) struct Run {
    insert: Insert,
    ready: Option<Ready>,
    wrote: bool,
}

impl Run {
    /// A run of `insert`, nothing written yet.
    pub(super) fn new(insert: Insert) -> Self {
        Self {
            insert,
            ready: None,
            wrote: false,
        }
    }
}

impl Actor {
    /// Writes the next step and asks the keyboard to paste it once; with none left, the insert ends, with a note when
    /// nothing went out.
    pub(super) fn step(&mut self, mut run: Run) {
        let Some((parts, step)) = self.next_write(&mut run) else {
            let note = (!run.wrote).then_some(PanelNote::PasteFailed);
            return self.finish(note);
        };
        if let Err(error) = self.ports.writer.write(&parts, self.settings.wait()) {
            return self.finish(Some(note_of(error)));
        }
        if let Some(step) = &step {
            run.insert.written(step);
            run.wrote = true;
        }
        self.paste.round = self.paste.round.wrapping_add(1);
        let (round, since) = (self.paste.round, self.ports.clock.now());
        let entry = step.map(|s| s.id);
        self.paste.state = Paste::Asked {
            run,
            round,
            entry,
            since,
        };
        // The keyboard answers on this thread while it handles this; the answer waits in the inbox, no lock held.
        self.publish(&PasteAsk { round });
    }

    /// What the next step writes: the entry waiting behind its join, else the next entry whose content is in hand,
    /// its join first when it takes one. Gives the step of an entry, none for a join.
    fn next_write(&mut self, run: &mut Run) -> Option<(Vec<ClipPart>, Option<Step>)> {
        if let Some(Ready { step, parts }) = run.ready.take() {
            return Some((parts, Some(step)));
        }
        loop {
            let step = run.insert.next(&self.history)?;
            let Some(parts) = self.content(step.id) else {
                continue;
            };
            let parts = paste_parts(parts, step.files.as_ref());
            if !step.join {
                return Some((parts, Some(step)));
            }
            run.ready = Some(Ready { step, parts });
            let join = Redacted::new(self.settings.join_config().text());
            return Some((vec![ClipPart::Text(join)], None));
        }
    }

    /// Entry `id`'s content; content gone or damaged deletes the entry.
    fn content(&mut self, id: u64) -> Option<Vec<ClipPart>> {
        let entry = self.history.get(id)?;
        match self.store.parts(entry) {
            Ok(Some(parts)) => return Some(parts),
            Ok(None) => self.remove(id),
            Err(SealedError::Damaged) => {
                self.remove(id);
                self.say_once(keys::CLIP_DAMAGED);
            }
            Err(error) => tracing::warn!(%error, "a clipboard entry could not be read"),
        }
        None
    }
}

/// The note a failed write shows: `Busy` only while another app holds the clipboard.
fn note_of(error: ClipError) -> PanelNote {
    if error == ClipError::Busy {
        PanelNote::Busy
    } else {
        PanelNote::PasteFailed
    }
}

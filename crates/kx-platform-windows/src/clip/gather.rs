//! What one look gathers, pure over how a format is sized and a part read. Text, HTML, RTF and files come first; an
//! image is fetched only when no text is left, as the picture Office puts beside text is not kept.

use kx_module_api::ports::ClipPart;

use super::parts::{Kind, Want, blank};

/// What a look found: the parts in the clipboard's order, and whether a blank text was dropped from them.
#[derive(Debug)]
pub(crate) struct Found {
    /// The parts kept.
    pub(crate) parts: Vec<ClipPart>,
    /// Whether a text of only blanks was dropped.
    pub(crate) blanked: bool,
}

/// Gathers the parts of `wants`, each sized before any is read. `Err` gives the bytes counted once they pass
/// `max_bytes`, and no format after that point is fetched.
pub(crate) fn gather(
    wants: &[Want],
    max_bytes: u64,
    size: impl Fn(u32) -> Option<u64>,
    read: impl Fn(Want) -> Option<ClipPart>,
) -> Result<Found, u64> {
    let mut look = Look {
        size,
        read,
        max_bytes,
        used: 0,
    };
    let mut slots: Vec<Option<ClipPart>> = wants.iter().map(|_| None).collect();
    let first = |w: &Want| !matches!(w.kind, Kind::Image(_));
    look.fill(wants, &mut slots, first)?;
    let blanked = drop_blank(&mut slots);
    let texted = slots
        .iter()
        .flatten()
        .any(|p| matches!(p, ClipPart::Text(_)));
    if !texted {
        look.fill(wants, &mut slots, |w| !first(w))?;
    }
    let parts = slots.into_iter().flatten().collect();
    Ok(Found { parts, blanked })
}

/// One look at a copy: how a format is sized and a part read, the cap, and the bytes counted so far.
struct Look<S, R> {
    size: S,
    read: R,
    max_bytes: u64,
    used: u64,
}

impl<S: Fn(u32) -> Option<u64>, R: Fn(Want) -> Option<ClipPart>> Look<S, R> {
    /// Sizes the `wants` that `pick` takes, then reads them into their `slots`; `Err` with the count past the cap.
    fn fill(
        &mut self,
        wants: &[Want],
        slots: &mut [Option<ClipPart>],
        pick: impl Fn(&Want) -> bool,
    ) -> Result<(), u64> {
        let max = self.max_bytes;
        let size = &self.size;
        let add = |sum: u64, w: &Want| {
            let sum = sum.saturating_add(size(w.format).unwrap_or(0));
            if sum > max { Err(sum) } else { Ok(sum) }
        };
        self.used = wants.iter().filter(|w| pick(w)).try_fold(self.used, add)?;
        let chosen = wants.iter().zip(slots.iter_mut()).filter(|(w, _)| pick(w));
        for (want, slot) in chosen {
            *slot = (self.read)(*want);
        }
        Ok(())
    }
}

/// Drops each text of only blanks from `slots`; true when one was dropped.
fn drop_blank(slots: &mut [Option<ClipPart>]) -> bool {
    let mut dropped = false;
    for slot in slots.iter_mut() {
        if matches!(slot, Some(ClipPart::Text(t)) if blank(t.expose())) {
            *slot = None;
            dropped = true;
        }
    }
    dropped
}

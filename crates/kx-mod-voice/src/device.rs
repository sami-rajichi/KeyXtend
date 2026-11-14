//! Where a model runs (ADR-0044 decisions 5, 7 and 8): the processor, a picked card once its check passed and it has
//! room, or for Automatic the fastest checked card with room; and a card check's verdict from its times and texts.

use kx_module_api::ports::{
    CardId, CardUse, CheckDone, Device, DeviceKind, DevicePick, Hundredths, Verdict,
};

use crate::score::apart;

/// What a card's check must meet: `card_gain` and `card_diff`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CardRules {
    /// How many times faster than the processor the card must be.
    pub gain: Hundredths,
    /// The most words its text may differ from the processor's.
    pub diff: u8,
}

/// A device the engine found, with its check for the model in use under its present driver, if any.
#[derive(Copy, Clone, Debug)]
pub struct Seen<'a> {
    /// The device now, with its free memory.
    pub device: &'a Device,
    /// Its check for this model and driver; `None` before one ran.
    pub check: Option<Verdict>,
}

/// Why the processor runs the model.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// The processor is picked.
    Picked,
    /// Automatic: no card has passed its check for this model.
    NoneChecked,
    /// The picked card has no check for this model yet.
    Unchecked,
    /// The picked card's check failed, with its verdict.
    Failed(Verdict),
    /// The picked card is gone.
    Gone,
    /// The card lacks room now: the model and the margin against its free memory, in MB.
    NoRoom {
        /// The model's card memory plus the margin.
        need_mb: u32,
        /// The card's free memory now.
        free_mb: u32,
    },
}

/// Where the model runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Serve {
    /// The processor, and why no card.
    Cpu(Why),
    /// This card, with what it must keep free.
    Card(CardUse),
}

/// Where `pick` runs a model needing `need_mb` on a card, keeping `margin_mb` free, among the devices `seen`.
#[must_use]
pub fn serve(pick: &DevicePick, seen: &[Seen<'_>], need_mb: u32, margin_mb: u32) -> Serve {
    let room = need_mb.saturating_add(margin_mb);
    let card = |id: &CardId| CardUse {
        id: id.clone(),
        need_mb,
        margin_mb,
    };
    let run = |id: Result<&CardId, Why>| id.map_or_else(Serve::Cpu, |id| Serve::Card(card(id)));
    match pick {
        DevicePick::Cpu => Serve::Cpu(Why::Picked),
        DevicePick::Auto => run(auto(seen, room)),
        DevicePick::Card(want) => {
            let Some((id, one)) = cards(seen).find(|(id, _)| *id == want) else {
                return Serve::Cpu(Why::Gone);
            };
            match one.check {
                None => Serve::Cpu(Why::Unchecked),
                Some(Verdict::Passed { .. }) => run(roomy(&one, room).map(|()| id)),
                Some(failed) => Serve::Cpu(Why::Failed(failed)),
            }
        }
    }
}

/// Automatic's card: the fastest passed card with `room` free, a tie to the most free memory.
fn auto<'a>(seen: &[Seen<'a>], room: u32) -> Result<&'a CardId, Why> {
    let mut best: Option<(Hundredths, u32, &CardId)> = None;
    let mut crowded: Option<u32> = None;
    for (id, one) in cards(seen) {
        let Some(fast) = gain(one.check) else {
            continue;
        };
        let free_mb = one.device.memory.free_mb;
        if roomy(&one, room).is_err() {
            crowded = crowded.max(Some(free_mb));
        } else if best.is_none_or(|(g, free, _)| (fast, free_mb) > (g, free)) {
            best = Some((fast, free_mb, id));
        }
    }
    match (best, crowded) {
        (Some((_, _, id)), _) => Ok(id),
        (None, Some(free_mb)) => Err(Why::NoRoom {
            need_mb: room,
            free_mb,
        }),
        (None, None) => Err(Why::NoneChecked),
    }
}

/// The seen cards, each with its id.
fn cards<'s, 'a>(seen: &'s [Seen<'a>]) -> impl Iterator<Item = (&'a CardId, Seen<'a>)> + 's {
    seen.iter().filter_map(|one| match &one.device.kind {
        DeviceKind::Card(id) => Some((id, *one)),
        DeviceKind::Cpu => None,
    })
}

/// Fine when the card has `room` free now, else why not.
fn roomy(one: &Seen<'_>, room: u32) -> Result<(), Why> {
    let free_mb = one.device.memory.free_mb;
    (free_mb >= room).then_some(()).ok_or(Why::NoRoom {
        need_mb: room,
        free_mb,
    })
}

/// A passed check's gain.
fn gain(check: Option<Verdict>) -> Option<Hundredths> {
    match check {
        Some(Verdict::Passed { gain }) => Some(gain),
        _ => None,
    }
}

/// A card check's verdict: its text against the processor's first (`card_diff` words in the clip's language `lang`),
/// then its speed (`card_gain`).
#[must_use]
pub fn verdict(done: &CheckDone, lang: &str, rules: &CardRules) -> Verdict {
    let words = apart(done.card_text.expose(), done.cpu_text.expose(), lang);
    if words > u16::from(rules.diff) {
        return Verdict::Differs {
            words,
            allowed: rules.diff,
        };
    }
    let per_one = u64::from(Hundredths::PER_ONE);
    let times = (u64::from(done.cpu_ms) * per_one).checked_div(u64::from(done.card_ms));
    let gain = Hundredths(times.map_or(u32::MAX, |t| u32::try_from(t).unwrap_or(u32::MAX)));
    if gain < rules.gain {
        return Verdict::Slow {
            gain,
            need: rules.gain,
        };
    }
    Verdict::Passed { gain }
}

#[cfg(test)]
mod tests;

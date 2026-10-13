//! The sounds compiled into the app: each clip's samples, and what each event plays, in turn, at its gain. kx-ui's
//! build checks the files `sounds.toml` names and embeds their samples, so nothing reads audio at run time.

use kx_platform::ui::{Clip, ClipId};
use kx_theme::sounds::Event;

/// Decibels per factor of ten in amplitude, and that factor.
const DB_PER_DECADE: f64 = 20.0;
const DECADE: f64 = 10.0;

/// Bytes aligned for 16-bit reads, so the speaker reads the samples in place.
#[repr(align(2))]
struct Aligned<T: ?Sized>(T);

/// The table kx-ui's build writes from `sounds.toml` and the checked files.
#[cfg(feature = "qt")]
mod table {
    use super::Aligned;
    include!(concat!(env!("OUT_DIR"), "/sounds.rs"));
}

/// Without Qt the app is not built, so no sound is compiled in.
#[cfg(not(feature = "qt"))]
mod table {
    use super::Aligned;
    /// No clip.
    pub(super) static CLIPS: [(&str, &Aligned<[u8]>); 0] = [];
    /// No event.
    pub(super) static EVENTS: [(&str, &[&str], f64); 0] = [];
    /// No rate.
    pub(super) const RATE: u32 = 0;
    /// No voice.
    pub(super) const VOICES: usize = 0;
}

/// What one event plays: its clips, in turn, and its gain.
#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    /// Its clips, played in turn.
    pub clips: Vec<ClipId>,
    /// Its balance as a linear gain, from 0 to 1.
    pub gain: f32,
}

/// Every compiled-in clip, each named by its place in the table.
#[must_use]
pub fn clips() -> Vec<Clip> {
    table::CLIPS
        .iter()
        .enumerate()
        .filter_map(|(i, (_, bytes))| {
            Some(Clip {
                id: id(i)?,
                rate: table::RATE,
                samples: &bytes.0,
            })
        })
        .collect()
}

/// What `event` plays; `None` when no sound is compiled in. It allocates and computes the gain, so callers build each
/// event's cue once at start, never per play.
#[must_use]
pub fn cue(event: Event) -> Option<Cue> {
    let (_, names, gain_db) = table::EVENTS.iter().find(|(e, ..)| *e == event.name())?;
    let place = |name: &&str| table::CLIPS.iter().position(|(n, _)| n == name);
    let clips = names.iter().filter_map(place).filter_map(id).collect();
    Some(Cue {
        clips,
        gain: gain(*gain_db),
    })
}

/// Voices per clip, so quick plays of one clip overlap; 0 when no sound is compiled in.
#[must_use]
pub fn voices() -> usize {
    table::VOICES
}

/// The id of the clip at place `i` in the table.
fn id(i: usize) -> Option<ClipId> {
    u16::try_from(i).ok().map(ClipId::new)
}

/// The linear gain of a balance of `db` decibels.
#[allow(
    clippy::cast_possible_truncation,
    reason = "a balance of 0 dB or less is a gain from 0 to 1, which f32 holds"
)]
fn gain(db: f64) -> f32 {
    DECADE.powf(db / DB_PER_DECADE) as f32
}

#[cfg(test)]
mod tests;

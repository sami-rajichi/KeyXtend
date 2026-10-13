//! The key and mouse sounds: each event's clips in turn at its gain, under the `[ui]` switches and volume. Each play is
//! a send to the speaker, which never blocks, and nothing here logs.

use super::{Hub, Redraw};
use crate::settings::{SOUND_VOLUME, UiSettings};
use crate::sounds::{self, Cue};
use kx_module_api::ports::{AssistNote, FaceAction};
use kx_platform::ui::{ClipId, Speaker, SpeakerConfig};
use kx_theme::sounds::Event;
use std::sync::atomic::{AtomicUsize, Ordering};

/// One event's cue, built once, and how many of its takes were picked.
struct Turn {
    event: Event,
    cue: Cue,
    picked: AtomicUsize,
}

/// Every event's cue with its turn; an event without clips has none.
pub(super) struct Sounds(Vec<Turn>);

impl Sounds {
    /// Builds each event's cue once and hands `speaker` every clip, to rest after `idle_ms` of silence.
    pub(super) fn new(speaker: &dyn Speaker, idle_ms: u32) -> Self {
        prepare(speaker, idle_ms);
        let turns = Event::ALL.into_iter().filter_map(|event| {
            let cue = sounds::cue(event)?;
            let picked = AtomicUsize::new(0);
            Some(Turn { event, cue, picked })
        });
        Self(turns.collect())
    }

    /// `event`'s next take and its gain at `volume`; `None` when the event has no clip.
    fn next(&self, event: Event, volume: u8) -> Option<(ClipId, f32)> {
        let turn = self.0.iter().find(|t| t.event == event)?;
        let n = turn.picked.fetch_add(1, Ordering::Relaxed);
        let id = *turn.cue.clips.get(n.checked_rem(turn.cue.clips.len())?)?;
        let full = f32::from(*SOUND_VOLUME.end());
        Some((id, turn.cue.gain * f32::from(volume) / full))
    }
}

impl Hub {
    /// Plays `event`'s next take at the `[ui]` volume, unless its switch is off or the volume is 0.
    pub fn sound(&self, event: Event) {
        let volume = {
            let live = self.lock();
            if !switched_on(&live.settings, event) {
                return;
            }
            live.settings.sound_volume
        };
        if volume == 0 {
            return;
        }
        if let Some((id, gain)) = self.sounds.next(event, volume) {
            self.ports.speaker.play(id, gain);
        }
    }

    /// Does a key's `action`, calls `mark` once the key went out, then plays the key sound for a tap or a key's
    /// first press; a held key's repeat never sounds.
    pub fn sent(&self, action: FaceAction, mark: impl FnOnce()) -> Redraw {
        let pressed = matches!(action, FaceAction::Tap { .. } | FaceAction::Press { .. });
        let redraw = self.act(action);
        mark();
        if pressed {
            self.sound(Event::Key);
        }
        redraw
    }

    /// Hands the speaker every clip again when `[ui] sound_idle_ms` went from `before` to `now`.
    pub(super) fn idle_changed(&self, before: u32, now: u32) {
        if before != now {
            prepare(&*self.ports.speaker, now);
        }
    }
}

/// Plays a mouse-assist note's sound: a hold that fired, a pick-up or a drop; the other notes have none.
pub(super) fn on_note(hub: &Hub, note: &AssistNote) {
    let event = match note {
        AssistNote::Fired { .. } => Event::Rclick,
        AssistNote::Picked { .. } => Event::Pick,
        AssistNote::Dropped { .. } => Event::Drop,
        AssistNote::HoldStart { .. } | AssistNote::HoldEnd | AssistNote::Cancelled => return,
    };
    hub.sound(event);
}

/// The `[ui]` switch of `event`.
fn switched_on(s: &UiSettings, event: Event) -> bool {
    match event {
        Event::Key => s.sound_key,
        Event::Rclick => s.sound_rclick,
        Event::Pick => s.sound_pick,
        Event::Drop => s.sound_drop,
    }
}

/// Hands `speaker` every compiled-in clip and the voices per clip, to rest after `idle_ms` of silence.
fn prepare(speaker: &dyn Speaker, idle_ms: u32) {
    let config = SpeakerConfig {
        voices: sounds::voices(),
        idle_ms,
    };
    speaker.prepare(sounds::clips(), config);
}

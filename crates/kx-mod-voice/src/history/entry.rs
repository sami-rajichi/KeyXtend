//! What History keeps and how a change reaches the store: a text with its time, language, model, pin and recording;
//! the rules `[voice]` sets; and the edit naming the blobs to write and remove.

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::{LocalTime, MB};
use kx_module_api::{Redacted, Secret};

use crate::VoiceSettings;

/// When a text was dictated: the moment and the UTC offset there, so its day stays the local day it was said.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stamp {
    /// Milliseconds since the Unix epoch, in UTC.
    pub unix_ms: i64,
    /// The local offset from UTC then, in minutes east.
    pub offset_min: i16,
}

impl Stamp {
    /// The stamp of `now`.
    #[must_use]
    pub fn of(now: &LocalTime) -> Self {
        Self {
            unix_ms: now.unix_ms,
            offset_min: now.offset_min,
        }
    }

    /// The wall time where it was dictated; `None` past the years a date holds.
    #[must_use]
    pub fn local(self) -> Option<LocalTime> {
        LocalTime::at(self.unix_ms, self.offset_min)
    }
}

/// A kept recording: its bytes, 16 kHz mono sound as the worker hands it over, and the blobs that hold them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sound {
    /// Its size in bytes.
    pub bytes: u64,
    /// How many blobs hold it, in order from 0.
    pub blobs: u32,
}

/// One text in History.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Its id, which also names its blobs; never used twice.
    pub id: u64,
    /// Its place in the order: the counter when it was kept or last unpinned.
    pub seq: u64,
    /// When it was dictated, or last unpinned, which counts as new as in the clipboard's History.
    pub at: Stamp,
    /// The text, without the suffix typed after it.
    pub text: Secret,
    /// The language it was dictated in, an ISO 639 code.
    pub lang: String,
    /// The model that wrote it: a row's id, a model file's name, or the cloud pick, `service:model`.
    pub model: Redacted<String>,
    /// It is pinned, outside `history_max`.
    pub pinned: bool,
    /// Its recording, with Keep recordings.
    pub sound: Option<Sound>,
}

/// What `[voice]` lets History keep in this run.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Texts are kept: `history` on and the run's saves not hidden (ADR-0037).
    pub texts: bool,
    /// The most unpinned texts (`history_max`).
    pub max: usize,
    /// Recordings are kept: texts too, and `keep_voice` on.
    pub voice: bool,
    /// The most bytes all recordings take (`voice_max_mb`).
    pub voice_bytes: u64,
}

impl Rules {
    /// The rules of `settings`; a hidden run keeps nothing.
    #[must_use]
    pub fn of(settings: &VoiceSettings, hidden: bool) -> Self {
        let texts = settings.history && !hidden;
        Self {
            texts,
            max: usize::from(settings.history_max),
            voice: texts && settings.keep_voice,
            voice_bytes: u64::from(settings.voice_max_mb).saturating_mul(MB),
        }
    }
}

/// The blobs a change writes and removes; the store writes first, then removes, so a crash leaves at most leftovers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Edit {
    /// Texts to write: new, replaced, pinned, unpinned, or which lost their recording.
    pub put: BTreeSet<u64>,
    /// Texts gone, each with the recording it held.
    pub gone: BTreeMap<u64, Option<Sound>>,
    /// Recordings gone while their text stays, or whose text was gone before they joined it.
    pub mute: BTreeMap<u64, Sound>,
}

impl Edit {
    /// True when it changes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.put.is_empty() && self.gone.is_empty() && self.mute.is_empty()
    }

    /// Adds `later`'s changes; a text gone since is no longer written.
    pub fn merge(&mut self, later: Self) {
        self.put.extend(later.put);
        self.mute.extend(later.mute);
        for (id, sound) in later.gone {
            self.put.remove(&id);
            let held = self.gone.entry(id).or_default();
            *held = held.or(sound);
        }
    }

    /// Notes `entry` as gone.
    pub(crate) fn drop_entry(&mut self, entry: &Entry) {
        self.put.remove(&entry.id);
        self.gone.insert(entry.id, entry.sound);
    }
}

/// The texts the Delete-all question counted; only `History::ask_all` makes one, so Delete all always asks first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked(pub(crate) BTreeSet<u64>);

impl Asked {
    /// How many unpinned texts the question names.
    #[must_use]
    pub fn count(&self) -> usize {
        self.0.len()
    }
}

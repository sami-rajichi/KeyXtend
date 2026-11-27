//! The keys each window shows and why one dims (edges 8, 11, 14 and 21), the capture bar's picking view and the
//! editor's; a click on a dimmed key is dropped.

use kx_module_api::ports::{
    AskKey, BarKey, Dim, Editor, Gear, Keep, Keycap, Picking, RecKey, Rect, SnipClick, SnipStage,
    StripKey,
};

use super::Actor;
use crate::flow::{Edit, Held, Sounds};
use crate::part;

/// A key that acts, unless `dim` says why not.
const fn cap<K>(key: K, on: bool, dim: Option<Dim>) -> Keycap<K> {
    Keycap { key, on, dim }
}

/// Why a microphone that cannot record dims Mic.
const fn mic_dim(gear: Gear) -> Option<Dim> {
    match gear {
        Gear::Ready => None,
        Gear::Missing => Some(Dim::NoMic),
        Gear::Refused => Some(Dim::MicRefused),
    }
}

/// Why a sound device that cannot record dims Sound.
const fn sound_dim(gear: Gear) -> Option<Dim> {
    match gear {
        Gear::Ready => None,
        Gear::Missing | Gear::Refused => Some(Dim::NoSound),
    }
}

/// True when `key` shows dimmed among `keys`.
fn dimmed<K: PartialEq>(keys: &[Keycap<K>], key: &K) -> bool {
    keys.iter().any(|cap| cap.key == *key && cap.dim.is_some())
}

impl Actor {
    /// Why Save dims: a hidden run made no Snips folder (edge 21).
    pub(super) fn save_dim(&self) -> Option<Dim> {
        self.seen.hidden.then_some(Dim::Hidden)
    }

    /// Why both video ways and Record dim, the first that holds of: a hidden run, a capture window not kept out of
    /// captures, or no video encoders (edge 14).
    fn video_dim(&self) -> Option<Dim> {
        let seen = &self.seen;
        let why = [
            (seen.hidden, Dim::Hidden),
            (seen.bare, Dim::Bare),
            (!seen.facts.video, Dim::NoVideo),
        ];
        why.into_iter()
            .find_map(|(holds, dim)| holds.then_some(dim))
    }

    /// The capture bar over the frozen screens, on the screen the pointer was on; `None` before a freeze.
    pub(super) fn picking(&self, picking: crate::flow::Picking) -> Option<Picking> {
        let frozen = self.flow.frozen()?;
        let screen = part::screen(frozen.pointer, &frozen.screens).or(frozen.screens.first())?;
        let video = |key| matches!(key, BarKey::VideoRegion | BarKey::VideoFull);
        let keys = BarKey::ALL.iter().map(|&key| {
            let dim = if video(key) { self.video_dim() } else { None };
            cap(key, key == picking.way, dim)
        });
        let windows = (picking.way == BarKey::Window).then(|| frozen.windows.clone());
        Some(Picking {
            screen: screen.rect(),
            keys: keys.collect(),
            corner: picking.corner,
            within: picking
                .corner
                .and_then(|c| part::bound(picking.way, c, &frozen.screens)),
            windows: windows.unwrap_or_default(),
        })
    }

    /// The strip's keys for what it holds: a picture's, a recorded video's with no Edit (ADR-0060), or a video's before
    /// it records.
    pub(super) fn strip_keys(&self, held: &Held) -> Vec<Keycap<StripKey>> {
        let taken = match held {
            Held::Picture(_) => vec![StripKey::Copy, StripKey::Edit],
            Held::Video(_) => vec![StripKey::Play, StripKey::Speed, StripKey::Copy],
            Held::Ready(sounds) => return self.ready_keys(*sounds),
        };
        let save = self.save_dim();
        let ends = [
            cap(StripKey::Save, false, save),
            cap(StripKey::Close, false, None),
        ];
        let keys = taken.into_iter().map(|key| cap(key, false, None));
        keys.chain(ends).collect()
    }

    /// The strip's keys of a video before it records.
    fn ready_keys(&self, sounds: Sounds) -> Vec<Keycap<StripKey>> {
        let record = self.video_dim().or(self.seen.low.then_some(Dim::DiskLow));
        let facts = self.seen.facts;
        vec![
            cap(StripKey::Record, false, record),
            cap(StripKey::Mic, sounds.mic, mic_dim(facts.mic)),
            cap(StripKey::Sound, sounds.sound, sound_dim(facts.sound)),
            cap(StripKey::Close, false, None),
        ]
    }

    /// The record bar's keys.
    pub(super) fn rec_keys(&self, sounds: Sounds) -> Vec<Keycap<RecKey>> {
        let facts = self.seen.facts;
        vec![
            cap(RecKey::Mic, sounds.mic, mic_dim(facts.mic)),
            cap(RecKey::Sound, sounds.sound, sound_dim(facts.sound)),
            cap(RecKey::Stop, false, None),
            cap(RecKey::Cancel, false, None),
        ]
    }

    /// The editor on its picture; `None` before its picture and model are known.
    pub(super) fn editor(&self, edit: Edit) -> Option<Editor> {
        let (place, full) = self.editor_place();
        Some(Editor {
            picture: self.picture.clone()?,
            body: self.canvas.as_ref()?.view(),
            save_dim: self.save_dim(),
            done: edit.done,
            place,
            full,
        })
    }

    /// The editor's box while on a screen still there (edge 19), and its maximised work area as kept, which the ui
    /// checks against the screens it sees then (edge 9).
    fn editor_place(&self) -> (Option<Rect>, Option<Rect>) {
        let screens = &self.seen.screens;
        let on = |r: &Rect| screens.iter().any(|screen| part::meets(*r, *screen));
        (self.settings.editor.filter(on), self.settings.editor_full)
    }

    /// True when `click` names a key the view told last shows dimmed, or a Save while Save dims.
    pub(super) fn dimmed(&self, click: &SnipClick) -> bool {
        let saves = matches!(
            click,
            SnipClick::Flat {
                keep: Keep::Save,
                ..
            } | SnipClick::Ask(AskKey::Save)
        );
        if saves && self.save_dim().is_some() {
            return true;
        }
        let Some(stage) = self.told.page().map(|page| &page.stage) else {
            return false;
        };
        match (click, stage) {
            (SnipClick::Bar(key), SnipStage::Pick(p)) => dimmed(&p.keys, key),
            (SnipClick::Strip(key), SnipStage::Strip(t)) => dimmed(&t.keys, key),
            (SnipClick::Record(key), SnipStage::Record(r)) => dimmed(&r.keys, key),
            _ => false,
        }
    }
}

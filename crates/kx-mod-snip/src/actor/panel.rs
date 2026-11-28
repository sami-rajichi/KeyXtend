//! The Snip panel (spec "The Snip panel"): its Capture and Video pages, each choice written to `[snip]` at once and
//! told to the flow, the folder dialog for the Save dialog's folder, Open folder, and the editor's remembered place.
//! The running snip keeps the settings it started with.

use std::ops::RangeInclusive;

use kx_module_api::ports::{
    AskError, CaptureSheet, Ends, FolderKey, FolderPath, RecordAsk, Rect, SnipClick, SnipKeyboard,
    SnipNote, SnipSheet, SnipTab, VideoRow, VideoSheet,
};
use serde::Serialize;

use super::media::refused_or;
use super::reply::Reply;
use super::{Actor, Back, Job, Msg};
use crate::flow::In;
use crate::settings::{Written, keys};
use crate::{COUNTDOWN_S, SnipSettings};

/// Where `value` stands in `range`.
fn ends<T: PartialEq>(value: &T, range: &RangeInclusive<T>) -> Ends {
    Ends {
        low: value == range.start(),
        high: value == range.end(),
    }
}

/// The countdown one `step` down or up, within its range.
fn countdown(now: u8, step: u8, up: bool) -> u8 {
    let next = if up {
        now.saturating_add(step)
    } else {
        now.saturating_sub(step)
    };
    next.clamp(*COUNTDOWN_S.start(), *COUNTDOWN_S.end())
}

/// The longest video one `step` down or up, on the step's multiples, within its range; the settings keep `step` above
/// 0.
fn longest(now: u16, step: u16, up: bool) -> u16 {
    let step = step.max(1);
    let next = if up {
        (now / step).saturating_add(1).saturating_mul(step)
    } else {
        now.saturating_sub(1) / step * step
    };
    let range = RecordAsk::MAX_MIN;
    next.clamp(*range.start(), *range.end())
}

impl Actor {
    /// Opens the panel on its Capture page.
    pub(super) fn open_panel(&mut self) {
        self.panel = Some(SnipTab::Capture);
    }

    /// The page `tab` shows, from the settings as last written.
    pub(super) fn sheet(&self, tab: SnipTab) -> SnipSheet {
        let s = &self.settings;
        match tab {
            SnipTab::Capture => SnipSheet::Capture(CaptureSheet {
                keyboard: s.keyboard,
                folder: self.save_folder(),
            }),
            SnipTab::Video => SnipSheet::Video(VideoSheet {
                countdown_s: s.countdown_s,
                countdown: ends(&s.countdown_s, &COUNTDOWN_S),
                fps: s.fps,
                max_min: s.max_min,
                longest: ends(&s.max_min, &RecordAsk::MAX_MIN),
                mic_on: s.mic_on,
                sound_on: s.sound_on,
            }),
        }
    }

    /// A click on the open panel; none acts while it is closed.
    pub(super) fn panel_click(&mut self, click: &SnipClick) {
        if self.panel.is_none() {
            return;
        }
        match *click {
            SnipClick::Tab(tab) => self.panel = Some(tab),
            SnipClick::Keyboard(keyboard) => self.keyboard(keyboard),
            SnipClick::Step { row, up } => self.step_row(row, up),
            SnipClick::Folder(FolderKey::Change) => self.change_folder(),
            SnipClick::Folder(FolderKey::Open) => self.open_folder(),
            SnipClick::ClosePanel => self.panel = None,
            _ => {}
        }
    }

    /// The editor ended at `rect`, maximised over work area `full` or not; both are kept for next time in one write.
    pub(super) fn placed(&mut self, rect: Rect, full: Option<Rect>) {
        let values = [(keys::EDITOR, Some(rect)), (keys::EDITOR_FULL, full)];
        let values = values.map(|(key, place)| (key, toml::Value::try_from(Written(place))));
        self.write_all(values, |s| {
            s.editor = Some(rect);
            s.editor_full = full;
        });
    }

    /// The keyboard during a capture.
    fn keyboard(&mut self, keyboard: SnipKeyboard) {
        self.write(keys::KEYBOARD, keyboard, |s| s.keyboard = keyboard);
    }

    /// A Video row's minus or plus; the frame rate's pair picks the recorder's first or last rate.
    fn step_row(&mut self, row: VideoRow, up: bool) {
        let s = &self.settings;
        match row {
            VideoRow::Countdown => {
                let next = countdown(s.countdown_s, s.countdown_step_s, up);
                self.write(keys::COUNTDOWN_S, next, |s| s.countdown_s = next);
            }
            VideoRow::Fps => {
                let fps = RecordAsk::FPS;
                let pick = if up { fps.last() } else { fps.first() };
                let next = pick.copied().unwrap_or(s.fps);
                self.write(keys::FPS, next, |s| s.fps = next);
            }
            VideoRow::Longest => {
                let next = longest(s.max_min, s.max_min_step, up);
                self.write(keys::MAX_MIN, next, |s| s.max_min = next);
            }
            VideoRow::Mic => self.write(keys::MIC_ON, up, |s| s.mic_on = up),
            VideoRow::Sound => self.write(keys::SOUND_ON, up, |s| s.sound_on = up),
        }
    }

    /// Writes `value` under `key`, then applies it and tells the flow; a refused write changes nothing.
    fn write(&mut self, key: &str, value: impl Serialize, apply: impl FnOnce(&mut SnipSettings)) {
        self.write_all([(key, toml::Value::try_from(value))], apply);
    }

    /// Writes each value under its key, all or none, then applies them and tells the flow; a refused write changes
    /// nothing.
    fn write_all<const N: usize>(
        &mut self,
        values: [(&str, Result<toml::Value, toml::ser::Error>); N],
        apply: impl FnOnce(&mut SnipSettings),
    ) {
        let keys = values.each_ref().map(|(key, _)| *key);
        let table = values
            .into_iter()
            .map(|(key, value)| value.map(|value| (key.to_owned(), value)))
            .collect::<Result<toml::Table, _>>();
        let written = table
            .map_err(|_| "a value has no TOML form")
            .and_then(|table| {
                self.ports
                    .handle
                    .set_all(table)
                    .map_err(|_| "it was refused")
            });
        if let Err(why) = written {
            return tracing::warn!(?keys, why, "a snip setting was not written");
        }
        apply(&mut self.settings);
        self.step(In::Settings(Box::new(self.settings.clone())));
    }

    /// Change: the system's folder dialog, at the folder the Save dialog opens at, off the actor; a job that panics or
    /// does not start answers that the dialog failed.
    fn change_folder(&mut self) {
        let Some(at) = self.save_folder() else {
            return;
        };
        let dialogs = self.ports.dialogs.clone();
        let reply = Reply::new(self.ports.back.clone(), Back::Folder(Err(AskError::Failed)));
        let limits = self.settings.pick_limits();
        let job: Job = Box::new(move || reply.send(Back::Folder(dialogs.folder(&at, &limits))));
        self.folders = self.folders.saturating_add(1);
        (self.ports.spawn)(job);
    }

    /// The folder dialog's answer: a folder chosen becomes `save_dir`; Cancel keeps it, and a refusal says why.
    pub(super) fn folder_picked(&mut self, answer: Result<Option<FolderPath>, AskError>) {
        self.folders = self.folders.saturating_sub(1);
        let folder = match answer {
            Ok(Some(folder)) => folder,
            Ok(None) => return,
            Err(error) => {
                tracing::warn!(%error, "the folder dialog gave no folder");
                return self.show_note(refused_or(&error, SnipNote::FolderFailed));
            }
        };
        let Some(typed) = folder.expose().to_str().map(str::to_owned) else {
            tracing::warn!("the folder chosen has no text form");
            return self.show_note(SnipNote::PlaceRefused);
        };
        self.write(keys::SAVE_DIR, typed.clone(), |s| s.save_dir = typed);
    }

    /// Open folder: the folder the Save dialog opens at, in the file manager, off the actor; a refusal shows its note.
    fn open_folder(&mut self) {
        let Some(folder) = self.save_folder() else {
            return;
        };
        let (reveal, back) = (self.ports.reveal.clone(), self.ports.back.clone());
        let job: Job = Box::new(move || {
            if let Err(error) = reveal.show(&folder) {
                tracing::warn!(%error, "the folder was not shown");
                let _ = back.send(Msg::Back(Back::Note(SnipNote::FolderFailed)));
            }
        });
        if !(self.ports.spawn)(job) {
            tracing::warn!("the folder's job did not start");
            self.show_note(SnipNote::FolderFailed);
        }
    }
}

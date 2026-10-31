//! The panel (spec §7 "Panel"): `ClipOpen` toggles it, keys and clicks go to its state machine, and the view goes out
//! once per message while it is open, held until the ui tells its rows (Ruling 32).

use kx_module_api::keys;
use kx_module_api::ports::{ClipAct, ClipKind, ClipPanel, HistoryWanted, Room, SealedError};

use super::Actor;
use super::paste::Paste;
use crate::media::{Size, job_of};
use crate::panel::Do;
use crate::view::{self, Facts, build};

impl Actor {
    /// Opens the panel, or closes it when open (Ruling 29); an open first drops what vanished and looks at the files.
    pub(super) fn toggle(&mut self) {
        if self.panel.is_open() {
            self.panel.close();
            return self.closed();
        }
        self.panel.open();
        self.drop_missing();
        self.look_rows(&self.file_ids());
        self.room_stale = true;
        self.redraw = true;
        self.publish(&ClipPanel { open: true });
    }

    /// The panel closed: an insert under way stops, its note was seen, and the keyboard hears it.
    pub(super) fn closed(&mut self) {
        self.paste.state = Paste::Idle;
        self.note = None;
        self.publish(&ClipPanel { open: false });
    }

    /// Takes a click; the ui's first rows let the held view show.
    pub(super) fn act(&mut self, act: ClipAct) {
        let rows = matches!(act, ClipAct::Rows(_));
        let first = rows && !self.rows_heard;
        self.rows_heard |= rows;
        let done = self.panel.act(act, &self.history);
        self.redraw |= first;
        self.act_on(done);
    }

    /// Does what the panel asks.
    pub(super) fn act_on(&mut self, done: Do) {
        match done {
            Do::Stay => {}
            Do::Show => self.redraw = true,
            Do::Preview(id) => self.preview(id),
            Do::Insert(ids) => self.insert(ids),
            Do::Pin(ids) => self.pin(&ids),
            Do::Unpin(ids) => self.unpin(&ids),
            Do::Delete(ids) => ids.into_iter().for_each(|id| self.remove(id)),
            Do::Close => self.closed(),
        }
    }

    /// Answers `HistoryAsk` with the chord flag and the panel's state, for a keyboard that started later.
    pub(super) fn history_again(&self) {
        let on = self.settings.history_key;
        self.publish(&HistoryWanted { on });
        self.publish(&ClipPanel {
            open: self.panel.is_open(),
        });
    }

    /// Reads the shelf's room again; a failure keeps the last.
    pub(super) fn refresh_room(&mut self) {
        match self.ports.shelf.room() {
            Ok(room) => (self.room, self.room_stale) = (Some(room), false),
            Err(error) => tracing::warn!(%error, "the clipboard drive's room could not be read"),
        }
    }

    /// Draws the open panel once, when this message changed it and the ui's rows are known.
    pub(super) fn flush(&mut self) {
        let redraw = std::mem::take(&mut self.redraw);
        if !redraw || !self.panel.is_open() || !self.rows_heard {
            return;
        }
        if self.room_stale {
            self.refresh_room();
        }
        let none = || Room {
            used: 0,
            free: 0,
            drive: String::new(),
        };
        let facts = Facts {
            now: self.ports.calendar.now(),
            room: self.room.clone().unwrap_or_else(none),
            note: self.note,
        };
        let store = &self.store;
        let thumb = |e: &_| store.thumb(e).ok().flatten();
        let view = build(&mut self.panel, &self.history, &facts, &self.limits, thumb);
        let bare = view
            .0
            .rows
            .iter()
            .filter(|r| r.kind == ClipKind::Image && r.thumb.is_none());
        let bare: Vec<u64> = bare.map(|r| r.id).collect();
        self.publish(&view);
        self.want_thumbs(&bare);
    }

    /// Opens the preview of `id` with its content; an image is then drawn large, and gone content deletes the entry.
    fn preview(&mut self, id: u64) {
        let Some(entry) = self.history.get(id).cloned() else {
            return;
        };
        match self.store.parts(&entry) {
            Ok(Some(parts)) => {
                let shown = view::preview(&entry, &parts, self.settings.preview_chars);
                self.panel.show(id, shown);
                if let Some(job) = job_of(id, Size::Preview, &parts) {
                    self.send_job(job);
                }
                self.redraw = true;
            }
            Ok(None) => self.remove(id),
            Err(SealedError::Damaged) => {
                self.remove(id);
                self.say_once(keys::CLIP_DAMAGED);
            }
            Err(error) => tracing::warn!(%error, "a clipboard entry could not be read"),
        }
    }
}

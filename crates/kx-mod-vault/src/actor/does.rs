//! What the actor does after a key or a click: draw, close, fill, save or paste; and the answers of the keyboard and
//! the clipboard history. Values reach the bus only as the fill and the page carry them, and logs hold error kinds.

use kx_module_api::Mono;
use kx_module_api::ports::{
    ClipTaken, FillAct, FillClick, FillLine, FillNote, FillPanel, FillTyped, VaultEmails,
};

use super::Actor;
use crate::entry::Stamp;
use crate::panel::{Desk, Do, Fill};

impl Actor {
    /// Takes a click; the ui's first rows let views show.
    pub(super) fn act(&mut self, act: FillAct) {
        if matches!(act.click, FillClick::Rows(_)) && !self.rows_heard {
            (self.rows_heard, self.redraw) = (true, true);
        }
        let mut desk = Desk {
            book: &mut self.book,
            random: &*self.ports.random,
            now: Stamp::of(&self.ports.calendar.now()),
        };
        let done = self.panel.act(act, &mut desk);
        self.act_on(done);
    }

    /// Does what the panel asked.
    pub(super) fn act_on(&mut self, done: Do) {
        match done {
            Do::Stay => return,
            Do::Show => {}
            Do::Close => {
                self.publish(&FillPanel { open: false });
                self.closed();
            }
            Do::Fill(fill) => self.fill(&fill),
            Do::Saved => self.saved(),
            Do::Paste => {
                let take = self.pasting.ask(self.ports.clock.now());
                self.publish(&take);
            }
            Do::Backup(step) => self.backup_step(&step),
        }
        self.left_backup();
        self.redraw = true;
    }

    /// The notebook changed: it saves once, the emails hear, and the backup counts it as a change.
    pub(super) fn saved(&mut self) {
        self.save();
        self.tell_emails(false);
        self.changed();
    }

    /// The panel shut: a fill no longer blocks the next one, and Paste stops waiting.
    pub(super) fn closed(&mut self) {
        self.filling.release();
        self.pasting.stop();
        self.backup.leave();
    }

    /// Sends `fill` to the keyboard, or shows why not; nothing of it stays here once sent.
    fn fill(&mut self, fill: &Fill) {
        match self.filling.ask(fill, &self.book, self.tab_gap) {
            Ok(request) => self.publish(&request),
            Err(note) => self.panel.say(note),
        }
    }

    /// The keyboard's answer: a typed fill says so, moves its entry's last use and saves; one that did not finish says
    /// that; an older round's answer is dropped.
    pub(super) fn typed(&mut self, typed: FillTyped) {
        let Some(answer) = self.filling.answer(typed) else {
            return;
        };
        match answer {
            Ok(id) => {
                self.panel.say(FillNote::Typed);
                let now = Stamp::of(&self.ports.calendar.now());
                if self.panel.filled(&mut self.book, id, now) {
                    self.save();
                }
            }
            Err(note) => self.panel.say(note),
        }
        self.redraw = true;
    }

    /// The clipboard history's answer to the take waited for; any other is dropped, its text wiped.
    pub(super) fn taken(&mut self, taken: &ClipTaken) {
        if self.pasting.answers(taken) {
            let done = self.panel.pasted(&taken.text);
            self.act_on(done);
        }
    }

    /// Saves the notebook; a failure is logged once by its kind and shows as a note, and the next change, the retry or
    /// the stop tries again.
    pub(super) fn save(&mut self) {
        match self.store.save(&self.book) {
            Ok(()) => self.unsaved = false,
            Err(error) => {
                if !std::mem::replace(&mut self.unsaved, true) {
                    tracing::warn!(%error, "the notebook was not saved");
                }
            }
        }
    }

    /// True while the notebook or the backup's blob waits for a save that works, which the note line says.
    pub(super) fn failing(&self) -> bool {
        self.unsaved || self.backup.unsaved()
    }

    /// While a save fails, tries the notebook and the blob again every `place_check_s`, counted from `now` at first.
    pub(super) fn retry_saves(&mut self, now: Mono) {
        if !self.failing() {
            self.retry_at = None;
            return;
        }
        let next = now.saturating_add(self.retry_every);
        if *self.retry_at.get_or_insert(next) > now {
            return;
        }
        if self.unsaved {
            self.save();
        }
        self.backup.retry();
        self.retry_at = self.failing().then_some(next);
    }

    /// Tells the emails when `always`, or when they changed since last told.
    pub(super) fn tell_emails(&mut self, always: bool) {
        let emails = self.book.emails();
        if always || emails != self.emails {
            self.publish(&VaultEmails {
                emails: emails.clone(),
            });
            self.emails = emails;
        }
    }

    /// Tells the line the page shows when `always`, or when it changed since last told, whatever the rows.
    pub(super) fn tell_line(&mut self, always: bool) {
        let line = self.panel.line(self.backup.typing());
        if always || line != self.line {
            self.publish(&FillLine(line.clone()));
            self.line = line;
        }
    }

    /// Tells a line that changed, and draws the open panel once its rows are known, when anything changed; a save
    /// failing or working again is a change, though no click made it.
    pub(super) fn flush(&mut self) {
        self.tell_line(false);
        let failing = self.failing();
        let turned = std::mem::replace(&mut self.told_failing, failing) != failing;
        if (std::mem::take(&mut self.redraw) || turned) && self.panel.is_open() && self.rows_heard {
            let view = self.view();
            self.publish(&view);
        }
    }
}

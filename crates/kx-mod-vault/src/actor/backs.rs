//! What the actor does for the backup: it hands on the backup pages' steps and the jobs' answers, tells each change,
//! makes the first offer, raises the memory checks and takes their answers, brings a box's or a `KeePass` file's
//! entries in, and draws the backup's page and the waiting footer.

use kx_module_api::ports::{FillPanel, FillView};
use kx_module_api::{Notice, NoticeAsk, keys};

use super::{Actor, Ports};
use crate::ID;
use crate::backup::{At, Done, Flowed};
use crate::book::Notebook;
use crate::entry::Stamp;
use crate::import::bring_in;
use crate::panel::{Page, Step};
use crate::view::{Backdrop, draw};

/// What the backup pages open on, from a notice's answer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Opening {
    /// Setup, after the first offer's Yes.
    Setup,
    /// A memory check, after its banner's Check.
    Check,
}

/// What the backup reads now: the notebook, the times and the rows a page shows.
fn at<'a>(book: &'a Notebook, ports: &Ports, rows: usize) -> At<'a> {
    At {
        book,
        now: ports.clock.now(),
        wall: ports.calendar.now(),
        rows,
    }
}

impl Actor {
    /// Hands `step` to the backup pages.
    pub(super) fn backup_step(&mut self, step: &Step) {
        let at = at(&self.book, &self.ports, self.panel.rows());
        let flowed = self.backup.step(step, &at);
        self.flowed(flowed);
    }

    /// A backup job's answer; a note for the list shows once the backup pages are left.
    pub(super) fn job_done(&mut self, done: Done) {
        let at = at(&self.book, &self.ports, self.panel.rows());
        let flowed = self.backup.done(done, &at);
        self.flowed(flowed);
        self.tell_note();
    }

    /// A write's note shows on the panel's own page while no backup page shows; while shut, it waits for the open.
    pub(super) fn tell_note(&mut self) {
        if !self.panel.is_open() || self.panel.on_backup() {
            return;
        }
        if let Some(note) = self.backup.untold() {
            self.panel.tell(note);
            self.redraw = true;
        }
    }

    /// Does what the backup says; anything but Stay draws again.
    fn flowed(&mut self, flowed: Flowed) {
        match flowed {
            Flowed::Stay => return,
            Flowed::Leave => self.panel.leave_backup(),
            Flowed::Show => {}
            Flowed::Offer => {
                self.offered();
                return;
            }
            Flowed::Brought => self.brought(),
            Flowed::Keepass => self.imported(),
        }
        self.redraw = true;
    }

    /// The boxes found offer themselves, but only over the list of a notebook still empty.
    fn offered(&mut self) {
        let list = matches!(self.panel.page(), Page::List) && self.book.is_empty();
        if list && self.panel.show_backup().is_some() {
            self.backup.offer_shown();
            self.redraw = true;
        } else {
            self.backup.leave();
        }
    }

    /// Brings in the entries a box gave and saves once, unless they would not fit; the backup then says which.
    fn brought(&mut self) {
        let Some(entries) = self.backup.take_brought() else {
            return;
        };
        let added = self.book.merge(entries, &self.limits);
        if added.is_some() {
            self.save();
            self.tell_emails(false);
        }
        let at = at(&self.book, &self.ports, self.panel.rows());
        self.backup.settle(added, &at);
    }

    /// Brings in what a `KeePass` file gave: anything new saves once and is a change as any save is, and the backup
    /// then goes back where Import was pressed.
    fn imported(&mut self) {
        let Some(import) = self.backup.take_import() else {
            return;
        };
        let now = Stamp::of(&self.ports.calendar.now());
        let random = &*self.ports.random;
        let counts = bring_in(&mut self.book, import, now, random, &self.limits);
        if counts.added > 0 {
            self.saved();
        }
        let flowed = self.backup.settle_import(counts);
        self.flowed(flowed);
    }

    /// A save, edit or delete: the boxes are written after the wait, and the first entry ever saved brings the offer.
    pub(super) fn changed(&mut self) {
        self.backup.changed(self.ports.clock.now());
        if self.backup.offers(&self.book) {
            let offer = Notice {
                ask: Some(&NoticeAsk::YES_LATER),
                ..Notice::about(keys::VAULT_OFFER, ID)
            };
            self.publish(&offer);
        }
    }

    /// The user answered notice `key`: the offer's Yes opens setup, and a memory check's banner asks again after
    /// Later, its Check opening the check.
    pub(super) fn answered(&mut self, key: &'static str, yes: bool) {
        match key {
            keys::VAULT_OFFER if yes => self.open_for(Opening::Setup),
            keys::VAULT_CHECK => {
                let check = yes && self.backup.checking();
                self.backup.later(Stamp::of(&self.ports.calendar.now()));
                if check {
                    self.open_for(Opening::Check);
                }
            }
            _ => {}
        }
    }

    /// Opens the panel on `opening`, or once an unsaved edit closes.
    fn open_for(&mut self, opening: Opening) {
        let Some(opened) = self.panel.show_backup() else {
            self.opening = Some(opening);
            return;
        };
        self.opening = None;
        if opened {
            self.publish(&FillPanel { open: true });
        }
        self.go(opening);
    }

    /// Shows `opening` on the backup pages.
    fn go(&mut self, opening: Opening) {
        match opening {
            Opening::Setup => self.backup_step(&Step::Open),
            Opening::Check => {
                let flowed = self.backup.check();
                self.flowed(flowed);
            }
        }
        self.redraw = true;
    }

    /// An opening remembered over an edit shows once the open panel shows no edit.
    pub(super) fn open_later(&mut self) {
        let Some(opening) = self.opening else {
            return;
        };
        if !self.panel.is_open() || self.panel.show_backup().is_none() {
            return;
        }
        self.opening = None;
        self.go(opening);
    }

    /// Raises the memory check's banner once it is due, unless a fill waits or a text is typed.
    pub(super) fn raise_check(&mut self) {
        let busy = self.filling.waiting()
            || matches!(self.panel.page(), Page::Edit(_))
            || self.backup.typing();
        let wall = Stamp::of(&self.ports.calendar.now());
        if self.backup.check_due(wall, busy) {
            let check = Notice {
                ask: Some(&NoticeAsk::CHECK_LATER),
                ..Notice::about(keys::VAULT_CHECK, ID)
            };
            self.publish(&check);
        }
    }

    /// The backup pages close with the panel or when another page shows.
    pub(super) fn left_backup(&mut self) {
        if !self.panel.on_backup() && self.backup.showing() {
            self.backup.leave();
        }
    }

    /// The view: the panel's page, or the backup's while it shows, with the place the backup waits for, its hold and a
    /// failed save.
    pub(super) fn view(&mut self) -> FillView {
        let wall = self.ports.calendar.now();
        let paged = self
            .panel
            .on_backup()
            .then(|| self.backup.draw(self.panel.rows(), &wall));
        let back = Backdrop {
            paged: paged.flatten(),
            waiting: self.backup.waiting(),
            paused: self.backup.paused(),
            failed: self.failing(),
        };
        draw(&mut self.panel, &self.book, back)
    }
}

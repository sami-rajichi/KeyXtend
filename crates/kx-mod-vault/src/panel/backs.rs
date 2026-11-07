//! The backup pages as the panel sees them: which page shows, and the clicks and keys it hands on. The header's
//! Close, New and Backup act here as on any page; the module draws the backup pages and says when they are left.

use kx_module_api::ports::{FillButton, FillClick, PanelKey};

use super::{Do, Page, Panel, Step};
use crate::book::Notebook;

impl Panel {
    /// True while the backup pages show.
    #[must_use]
    pub fn on_backup(&self) -> bool {
        matches!(self.page, Page::Backup)
    }

    /// The header's Backup: the backup pages open, unless New or Edit shows, which it would drop unsaved.
    pub(super) fn open_backup(&mut self) -> Do {
        if matches!(self.page, Page::Edit(_)) {
            return Do::Stay;
        }
        self.page = Page::Backup;
        self.note = None;
        Do::Backup(Step::Open)
    }

    /// Import on the empty notebook's invitation: the backup pages open on its folder list; any other list has none.
    pub(super) fn open_import(&mut self, book: &Notebook) -> Do {
        if !matches!(self.page, Page::List) || !book.is_empty() {
            return Do::Stay;
        }
        self.page = Page::Backup;
        self.note = None;
        Do::Backup(Step::Import)
    }

    /// Shows the backup pages, opening the panel fresh when shut, as the first offer's Yes does: true when it opened,
    /// and `None` while New or Edit shows, which it would drop unsaved.
    pub fn show_backup(&mut self) -> Option<bool> {
        let opened = !self.is_open();
        if opened {
            self.reset(Page::Backup);
        } else if matches!(self.page, Page::Edit(_)) {
            return None;
        } else {
            (self.page, self.note) = (Page::Backup, None);
        }
        Some(opened)
    }

    /// Leaves the backup pages for the list.
    pub fn leave_backup(&mut self) {
        if self.on_backup() {
            self.page = Page::List;
        }
    }

    /// A click on the backup pages: the header's buttons act here, the rest go on to the module.
    pub(super) fn backup_click(&mut self, click: FillClick) -> Do {
        match click {
            FillClick::Button(FillButton::Close) => self.shut(),
            FillClick::Button(FillButton::New) => {
                self.page = Page::List;
                self.start_new()
            }
            FillClick::Button(FillButton::Backup) => self.open_backup(),
            other => Do::Backup(Step::Click(other)),
        }
    }

    /// A key on the backup pages, which the module takes.
    pub(super) fn backup_key(key: &PanelKey) -> Do {
        Do::Backup(Step::Key(key.clone()))
    }
}

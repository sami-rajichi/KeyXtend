//! Moving between the panel's pages: Back, up one folder or to where a cloud page was opened from, and showing a page
//! afresh with what it needs read.

use kx_module_api::ports::{ModelSide, VoiceTab};

use super::typing::Typed;
use super::{Do, Page, Panel, wrap};

impl Panel {
    /// Back: up one folder in the folder list, a card to where it was opened, a key page to the Cloud page, else home;
    /// leaving Compare stops it.
    pub(super) fn back(&mut self) -> Vec<Do> {
        match self.page {
            Some(Page::Folders) => self.up(),
            Some(Page::Card) if self.cloud.from_lines => self.go(Page::Lines),
            Some(Page::Card | Page::Lines) => self.go(Page::Tab(VoiceTab::Cloud)),
            Some(Page::Tab(_)) => {
                let mut out = wrap(self.compare.cancel());
                self.compare.reset();
                out.extend(self.go(Page::Home));
                out
            }
            Some(Page::Home) | None => Vec::new(),
        }
    }

    /// Shows `page` afresh, with what it needs read; a Paste or test waited for is dropped.
    pub(super) fn go(&mut self, page: Page) -> Vec<Do> {
        self.page = Some(page);
        (self.at, self.note, self.asked, self.hold) = (0, None, None, None);
        self.ticks.clear();
        self.cloud.next();
        (self.cloud.refused, self.cloud.refused_model) = (None, None);
        self.cloud.typed = Typed::default();
        match page {
            Page::Tab(VoiceTab::Models) => {
                self.cloud.side = (ModelSide::Local, 0);
                vec![Do::Scan]
            }
            Page::Tab(VoiceTab::Device | VoiceTab::Options) => vec![Do::Devices],
            Page::Folders => {
                (self.browse, self.dirs) = (None, Vec::new());
                Vec::new()
            }
            Page::Lines => {
                self.cloud.dots = 0;
                let shown = self.cloud.shown.clone();
                shown.map(Do::Dots).into_iter().collect()
            }
            Page::Home | Page::Card | Page::Tab(_) => Vec::new(),
        }
    }
}

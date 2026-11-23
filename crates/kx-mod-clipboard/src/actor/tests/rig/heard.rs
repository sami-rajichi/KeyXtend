//! The rig's readers of what the actor published: each kind of event, in the order it came.

use kx_module_api::ports::{ClipPage, ClipView, PanelLine, PasteAsk};

use super::super::ears::Heard;
use super::Rig;

impl Rig {
    /// Everything published so far.
    pub fn all(&self) -> Vec<Heard> {
        self.heard.lock().unwrap().clone()
    }

    /// Every published event `pick` takes, as it says.
    fn picked<T>(&self, pick: impl Fn(Heard) -> Option<T>) -> Vec<T> {
        self.all().into_iter().filter_map(pick).collect()
    }

    /// Every view published.
    pub fn views(&self) -> Vec<ClipView> {
        self.picked(|h| match h {
            Heard::View(v) => Some(v),
            _ => None,
        })
    }

    /// The last page published.
    pub fn page(&self) -> ClipPage {
        self.views().last().unwrap().0.as_ref().clone()
    }

    /// The last page's titles.
    pub fn titles(&self) -> Vec<String> {
        let rows = self.page().rows;
        rows.iter().map(|r| r.title.expose().clone()).collect()
    }

    /// Every `ClipPanel` published, as its flag.
    pub fn panels(&self) -> Vec<bool> {
        self.picked(|h| match h {
            Heard::Panel(p) => Some(p.open),
            _ => None,
        })
    }

    /// Every `ClipLine` published, as its line.
    pub fn lines(&self) -> Vec<PanelLine> {
        self.picked(|h| match h {
            Heard::Line(l) => Some(l),
            _ => None,
        })
    }

    /// Every `HistoryWanted` published, as its flag.
    pub fn wanted(&self) -> Vec<bool> {
        self.picked(|h| match h {
            Heard::Wanted(w) => Some(w.on),
            _ => None,
        })
    }

    /// Every paste request.
    pub fn pastes(&self) -> Vec<PasteAsk> {
        self.picked(|h| match h {
            Heard::Paste(p) => Some(p),
            _ => None,
        })
    }

    /// Every notice key published.
    pub fn notices(&self) -> Vec<&'static str> {
        self.picked(|h| match h {
            Heard::Notice(n) => Some(n.key),
            _ => None,
        })
    }
}

//! The line the lane follows (ADR-0040): the app's box, a panel's search, read from its own text and never from the box,
//! or a private line, which shows nothing. Nothing on a line is learned, and its text stays in memory, wiped as it goes.

use super::Actor;
use crate::pack::Pack;
use crate::score::knows;
use kx_module_api::ports::{FieldChange, PanelLine, TypingModeAsk, read_end};
use kx_module_api::{Redacted, Secret, Zeroizing};

/// What the lane follows, as the keyboard tells it.
#[derive(Default, PartialEq, Eq)]
pub(super) enum Line {
    /// The app's box.
    #[default]
    App,
    /// A panel's search, with its whole text.
    Search(Secret),
    /// A panel's private line, whose value never comes here.
    Private,
}

impl Line {
    /// The line the keyboard told, its text moved into the wiped form.
    fn of(line: PanelLine) -> Self {
        match line {
            PanelLine::None => Self::App,
            PanelLine::Search(text) => {
                Self::Search(Redacted::new(Zeroizing::new(text.into_inner())))
            }
            PanelLine::Private => Self::Private,
        }
    }

    /// True while the lane follows the app's box.
    pub(super) fn is_app(&self) -> bool {
        *self == Self::App
    }

    /// The search's text, on a search.
    pub(super) fn search(&self) -> Option<&str> {
        match self {
            Self::Search(text) => Some(text.expose().as_str()),
            Self::App | Self::Private => None,
        }
    }
}

impl Actor {
    /// The keyboard tells the line it aims at: a change ends what the lane followed and reads the new line, and leaving
    /// the app's box learns its open email, as a click does. True when the lane changes; off holds no line.
    pub(super) fn heard_line(&mut self, seed: Option<&Pack<'_>>, line: PanelLine) -> bool {
        let line = Line::of(line);
        if !self.settings.on || line == self.line {
            return false;
        }
        self.keep_email();
        self.line = line;
        self.menu.key();
        self.noted = None;
        let told = self.drop_word();
        if self.following() {
            self.act(seed, told);
        }
        true
    }

    /// Reads the search as a box is read, a long one cut as a platform read cuts it; a private line stays empty.
    pub(super) fn read_line(&mut self, seed: Option<&Pack<'_>>) {
        if let Some(text) = self.line.search() {
            let text = read_end(text, usize::from(self.settings.read_chars));
            self.tracker.read_line(text, &|w: &str| knows(seed, w));
        }
    }

    /// Focus moved in the app behind a line: only the box's kind and id are noted, for when the aim leaves.
    pub(super) fn note_box(&mut self, change: FieldChange) {
        self.kind = change.kind;
        self.tracker.note(change);
    }

    /// Prediction turned off or on: off holds no line, and on asks the keyboard for it again.
    pub(super) fn turned(&mut self, on: bool) {
        self.line = Line::App;
        if on {
            self.publish(&TypingModeAsk);
        }
    }
}

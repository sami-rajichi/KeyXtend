//! The line each page shows: as the keyboard hears it (ADR-0040), by its kind and a search's text before the caret, a
//! form's or a password's text never leaving here; and that line's caret, which moves and takes clicks without moving
//! the list.

use kx_module_api::Redacted;
use kx_module_api::caret::{self, Side};
use kx_module_api::ports::{FillFilter, PanelLine};

use super::edit::{Editor, Tag, Taking};
use super::input::shown;
use super::{Do, Page, Panel, Sub};
use crate::entry::Line;

impl Panel {
    /// The line the page shows; `password` says the backup pages are on their password line. Values draws no search,
    /// and a question or the kind chips or labels table over New and Edit draw no line.
    #[must_use]
    pub fn line(&self, password: bool) -> PanelLine {
        match &self.page {
            Page::List => PanelLine::Search(Redacted::new(self.search.before().to_owned())),
            Page::Edit(edit) if edit.kind.is_some() && edit.sub == Sub::Form => PanelLine::Private,
            Page::Backup if password => PanelLine::Private,
            _ => PanelLine::None,
        }
    }

    /// An arrow on a list page: it moves the search's caret while the line draws text, and else steps the filters.
    pub(super) fn list_arrow(&mut self, side: Side) -> Do {
        if matches!(self.page, Page::Values { .. }) || caret::empty(self.search.text()) {
            return self.next_filter(side == Side::Right);
        }
        shown(self.search.arrow(side))
    }

    /// Steps to the filter after or before, stopping at the ends.
    fn next_filter(&mut self, on: bool) -> Do {
        let at = FillFilter::ALL.iter().position(|&f| f == self.filter);
        let to = at.and_then(|i| {
            if on {
                i.checked_add(1)
            } else {
                i.checked_sub(1)
            }
        });
        let next = to.and_then(|i| FillFilter::ALL.get(i)).copied();
        next.map_or(Do::Stay, |f| self.set_filter(f))
    }

    /// A click placed the caret `at` into what the line that takes the letters draws.
    pub(super) fn place_caret(&mut self, at: u32) -> Do {
        match &mut self.page {
            Page::List => shown(self.search.place_drawn(at)),
            Page::Edit(edit) if edit.sub == Sub::Form => shown(edit.place(at)),
            _ => Do::Stay,
        }
    }

    /// The search's caret, in what it draws.
    pub(crate) fn search_caret(&self) -> u32 {
        self.search.drawn()
    }
}

impl Tag {
    /// The user's own label line, to read.
    const fn own(&self) -> Option<&Line> {
        match self {
            Self::Own(own) => Some(own),
            Self::Table(_) => None,
        }
    }

    /// The user's own label line, to type in.
    const fn own_mut(&mut self) -> Option<&mut Line> {
        match self {
            Self::Own(own) => Some(own),
            Self::Table(_) => None,
        }
    }
}

/// Names the line `taking` the letters out of the name and the highlighted field's own label and value: the one rule,
/// whether a line is read or typed in.
fn taker<L>(taking: Taking, name: L, field: Option<(Option<L>, L)>) -> Option<L> {
    match (taking, field) {
        (Taking::Name, _) => Some(name),
        (Taking::Label, Some((own, _))) => own,
        (Taking::Value, Some((_, value))) => Some(value),
        (Taking::Label | Taking::Value, None) => None,
    }
}

impl Editor {
    /// The line that takes the letters, to read, once a kind is picked.
    fn typed(&self) -> Option<&Line> {
        self.kind?;
        let slot = self.slots.get(self.at);
        let field = slot.map(|s| (s.tag.own(), &s.value));
        taker(self.taking, &self.name, field)
    }

    /// The line that takes the letters, once a kind is picked.
    pub(crate) fn line(&mut self) -> Option<&mut Line> {
        self.kind?;
        let slot = self.slots.get_mut(self.at);
        let field = slot.map(|s| (s.tag.own_mut(), &mut s.value));
        taker(self.taking, &mut self.name, field)
    }

    /// True when the line that takes the letters draws dots: a value hidden and not shown.
    fn masked(&self) -> bool {
        let hidden = self.slots.get(self.at).is_some_and(|s| s.hidden);
        self.taking == Taking::Value && hidden && !self.shown.contains(&self.at)
    }

    /// The arrow on `side` on the line that takes the letters; a mask follows its dots, which hide the text's way and
    /// where letters combine.
    pub(super) fn arrow(&mut self, side: Side) -> bool {
        let masked = self.masked();
        self.line().is_some_and(|line| {
            if masked {
                line.step_dot(side == Side::Right)
            } else {
                line.arrow(side)
            }
        })
    }

    /// Delete on that line; a mask loses one dot.
    pub(super) fn delete(&mut self) -> bool {
        let masked = self.masked();
        self.line().is_some_and(|line| {
            if masked {
                line.delete_dot()
            } else {
                line.delete()
            }
        })
    }

    /// Puts that line's caret `at` into what it draws.
    fn place(&mut self, at: u32) -> bool {
        let masked = self.masked();
        self.line().is_some_and(|line| {
            if masked {
                line.place_dots(at)
            } else {
                line.place_drawn(at)
            }
        })
    }

    /// That line's caret, in what it draws; 0 with no such line.
    pub(crate) fn typed_caret(&self) -> u32 {
        let draws = if self.masked() {
            Line::dots
        } else {
            Line::drawn
        };
        self.typed().map_or(0, draws)
    }
}

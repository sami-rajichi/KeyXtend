//! The language list in the face's state (spec §3.3): the language key opens and steps it, the board's keys move it,
//! and a pick switches the layout.

use super::{Inner, notice};
use crate::face::STATE;
use crate::langkey::{LangClick, click};
use crate::panel::{LangList, Moved};
use crate::settings::LangKey;
use crate::spec::Nav;
use kx_module_api::keys;
use kx_module_api::ports::{FaceError, FaceReply};

impl Inner {
    /// A click on the language key: a step asks the app to switch, which a later refresh sees. With one layout
    /// installed it does nothing, except close a list that is still open.
    pub(in crate::face) fn lang(&mut self, back: bool) -> FaceReply {
        let count = self.ports.layouts.list().len();
        match click(self.settings.lang_key, back, count, self.list.is_some()) {
            LangClick::Still => FaceReply::default(),
            LangClick::Step(by) => match self.ports.layouts.step(by) {
                Ok(()) => {
                    if self.settings.lang_key == LangKey::Carousel {
                        self.turn.asked(by);
                    }
                    FaceReply::default()
                }
                Err(_) => notice(keys::LANG_NOT_SWITCHED, FaceReply::default()),
            },
            LangClick::List => {
                let now = self.ports.layouts.current().label.id;
                let at = self.ids().iter().position(|&id| id == now);
                self.list = match self.list {
                    Some(_) => None,
                    None => Some(LangList::open(at.unwrap_or_default())),
                };
                STATE
            }
        }
    }

    /// Key `nav` for the open list, at `rows` rows per page; with no list open it does nothing.
    pub(in crate::face) fn panel(&mut self, nav: Nav, rows: usize) -> Result<FaceReply, FaceError> {
        let len = self.ports.layouts.list().len();
        let Some(list) = self.list.as_mut() else {
            return Ok(FaceReply::default());
        };
        match list.key(nav, rows, len) {
            Moved::Stay => Ok(FaceReply::default()),
            Moved::Hi => Ok(STATE),
            Moved::Close => {
                self.list = None;
                Ok(STATE)
            }
            Moved::Pick(at) => self.pick(at),
        }
    }

    /// Picks the layout at `index` of the list and closes it.
    pub(super) fn pick(&mut self, index: usize) -> Result<FaceReply, FaceError> {
        let list = self.ports.layouts.list();
        let label = list.get(index).ok_or(FaceError::NoChoice(index))?;
        self.list = None;
        self.turn.clear();
        Ok(match self.ports.layouts.pick(label.id) {
            Ok(()) => STATE,
            Err(_) => notice(keys::LANG_NOT_SWITCHED, STATE),
        })
    }
}

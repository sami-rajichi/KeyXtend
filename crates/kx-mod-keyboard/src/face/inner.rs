//! The face's state behind its lock: typing, size and place, the language key's turn and the open list.

use super::STATE;
use crate::geom::{Area, KeyBox};
use crate::langkey::{LangClick, Turn, click};
use crate::panel::{LangList, Moved};
use crate::place::{Placer, screen_of};
use crate::run::{Ports, TapError, Tapped, Typing};
use crate::settings::{KeyboardSettings, LangKey};
use crate::sizer::{SizeError, Sizer};
use crate::spec::{BoardSpec, Nav};
use crate::view::{StateInput, StateView, state_view};
use kx_module_api::ports::{
    BASE_DPI, FaceAction, FaceError, FaceReply, LayoutId, Layouts, Point, Rect, Screen, physical,
};
use kx_module_api::{SettingsHandle, keys};
use std::sync::Arc;

/// Everything a click can change.
pub(super) struct Inner {
    ports: Ports,
    settings: KeyboardSettings,
    typing: Typing,
    sizer: Sizer,
    placer: Placer,
    /// The screens the UI last reported.
    screens: Vec<Screen>,
    /// The size-1 plate in logical pixels.
    plate: Area,
    turn: Turn,
    /// The language list, while open.
    list: Option<LangList>,
}

impl Inner {
    /// Nothing latched, no list open, at the saved size; placed once the UI reports its screens.
    pub(super) fn new(
        spec: &BoardSpec,
        plate: Area,
        ports: Ports,
        settings: KeyboardSettings,
        store: SettingsHandle,
    ) -> Self {
        Self {
            typing: Typing::new(spec, &settings),
            sizer: Sizer::new(&settings, store.clone()),
            placer: Placer::new(&settings, store),
            turn: Turn::new(ports.layouts.current().label.id),
            ports,
            settings,
            screens: Vec::new(),
            plate,
            list: None,
        }
    }

    /// New services and settings; the latches and wait stay, the size and place come from the settings again.
    pub(super) fn renew(
        &mut self,
        ports: Ports,
        settings: KeyboardSettings,
        store: SettingsHandle,
    ) {
        self.typing.update(&settings);
        self.sizer = Sizer::new(&settings, store.clone());
        self.placer = Placer::new(&settings, store);
        if !self.screens.is_empty() {
            self.placer.start(self.size(), &self.screens);
        }
        if settings.lang_key != LangKey::List {
            self.list = None;
        }
        self.turn = Turn::new(ports.layouts.current().label.id);
        self.ports = ports;
        self.settings = settings;
    }

    /// True while the language list is open.
    pub(super) fn listing(&self) -> bool {
        self.list.is_some()
    }

    /// Types key `id` on the layout shown.
    pub(super) fn tap(&mut self, id: &str, back: bool) -> Result<Tapped, TapError> {
        let table = self.ports.layouts.current();
        self.typing.tap(&self.ports.io(&table), id, back)
    }

    /// The layouts service, to refresh without the lock.
    pub(super) fn layouts(&self) -> Arc<dyn Layouts> {
        Arc::clone(&self.ports.layouts)
    }

    /// Notes the layout shown, setting the names' turn; true when it changed.
    pub(super) fn seen(&mut self) -> bool {
        let now = self.ports.layouts.current().label.id;
        self.turn.seen(now, &self.ids())
    }

    fn ids(&self) -> Vec<LayoutId> {
        self.ports.layouts.list().iter().map(|l| l.id).collect()
    }

    /// A click on the language key: a step asks the app to switch, which a later refresh sees; with one layout
    /// installed it does nothing.
    pub(super) fn lang(&mut self, back: bool) -> FaceReply {
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
    pub(super) fn panel(&mut self, nav: Nav, rows: usize) -> Result<FaceReply, FaceError> {
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
    fn pick(&mut self, index: usize) -> Result<FaceReply, FaceError> {
        let list = self.ports.layouts.list();
        let label = list.get(index).ok_or(FaceError::NoChoice(index))?;
        self.list = None;
        self.turn.clear();
        Ok(match self.ports.layouts.pick(label.id) {
            Ok(()) => STATE,
            Err(_) => notice(keys::LANG_NOT_SWITCHED, STATE),
        })
    }

    /// The size, place and pick actions; the face does taps, relabels and panel keys itself.
    pub(super) fn act(&mut self, action: FaceAction) -> Result<FaceReply, FaceError> {
        match action {
            FaceAction::Smaller => sized(self.sizer.smaller()),
            FaceAction::Bigger => sized(self.sizer.bigger()),
            FaceAction::Preset { index } => sized(self.sizer.pick(index)),
            FaceAction::ResizeBegin { x, y } => {
                let plate = self.plate_px();
                self.sizer.begin(Point { x, y }, plate);
                Ok(FaceReply::default())
            }
            FaceAction::ResizeAt { x, y } => Ok(FaceReply {
                board: self.sizer.follow(Point { x, y }),
                ..FaceReply::default()
            }),
            FaceAction::ResizeEnd => sized(self.sizer.end()),
            FaceAction::Placed { rect, screens } => Ok(self.placed(rect, screens)),
            FaceAction::LangPick { index } => self.pick(index),
            FaceAction::Tap { .. } | FaceAction::Relabel | FaceAction::PanelKey { .. } => {
                Ok(FaceReply::default())
            }
        }
    }

    /// The window's size now, in logical pixels.
    fn size(&self) -> (f64, f64) {
        let plate = self.plate.scale(self.sizer.size());
        (plate.w, plate.h)
    }

    /// The window's size in the physical pixels of the screen it is on, as the pointer counts them.
    fn plate_px(&self) -> (f64, f64) {
        let size = self.size();
        let on = self
            .placer
            .at()
            .and_then(|at| screen_of(at, size, &self.screens));
        let dpi = on.map_or(BASE_DPI, |s| s.dpi);
        (
            f64::from(physical(size.0, dpi)),
            f64::from(physical(size.1, dpi)),
        )
    }

    /// The window ended at `rect`: where the face put it, the saved place is fitted to `screens` again without
    /// saving; anywhere else the user moved it, so it is fitted and saved.
    fn placed(&mut self, rect: Rect, screens: Vec<Screen>) -> FaceReply {
        self.screens = screens;
        let (to, size) = (
            Point {
                x: rect.x,
                y: rect.y,
            },
            self.size(),
        );
        let moved = self.placer.at().is_some_and(|at| at != to);
        let saved = if moved {
            self.placer.moved(to, size, &self.screens).map(|_| ())
        } else {
            self.placer.start(size, &self.screens);
            Ok(())
        };
        let state = self.placer.at() != Some(to);
        match saved {
            Ok(()) => FaceReply {
                state,
                ..FaceReply::default()
            },
            Err(_) => notice(keys::SETTINGS_UNSAVED, STATE),
        }
    }

    /// The settings and the size, which the board is drawn from.
    pub(super) fn sizing(&self) -> (KeyboardSettings, f64) {
        (self.settings.clone(), self.sizer.size())
    }

    /// The state of `keys`, with `panel_keys` driving an open list; a new layout is noted first, so a reader
    /// in its change event already sees the names' turn.
    pub(super) fn state(&mut self, keys: &[KeyBox], panel_keys: &[(String, Nav)]) -> StateView {
        self.seen();
        let table = self.ports.layouts.current();
        let list = self.ports.layouts.list();
        state_view(&StateInput {
            keys,
            latches: self.typing.latches(),
            caps: self.ports.source.caps_on(),
            table: &table,
            list: &list,
            turn: self.turn.turn(),
            open: self.list.as_ref(),
            panel_keys,
            at: self.placer.at(),
        })
    }
}

/// `reply` with notice `key`.
fn notice(key: &'static str, reply: FaceReply) -> FaceReply {
    FaceReply {
        notice: Some(key),
        ..reply
    }
}

/// The reply to a size change: redraw the board when it changed, with a notice when it could not be saved.
fn sized(changed: Result<bool, SizeError>) -> Result<FaceReply, FaceError> {
    let redraw = |board| FaceReply {
        board,
        ..FaceReply::default()
    };
    changed.map(redraw).or_else(|e| match e {
        SizeError::NoPreset(index) => Err(FaceError::NoChoice(index)),
        SizeError::Unsaved(_) => Ok(notice(keys::SETTINGS_UNSAVED, redraw(true))),
    })
}

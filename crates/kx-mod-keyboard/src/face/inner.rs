//! The face's state behind its lock: typing, a held key, size and place, the language key's turn, the open list and
//! the mouse module's state.

mod assist;
mod fit;
mod held;

pub(super) use held::Went;

use super::STATE;
use crate::geom::{Area, KeyBox};
use crate::langkey::{LangClick, Turn, click};
use crate::panel::{LangList, Moved};
use crate::place::Placer;
use crate::repeat::Repeat;
use crate::run::{Ports, Typing};
use crate::settings::{KeyboardSettings, LangKey};
use crate::sizer::Sizer;
use crate::spec::{BoardSpec, Nav};
use crate::view::{StateInput, StateView, state_view};
use kx_module_api::ports::{
    AssistChanged, FaceAction, FaceError, FaceReply, LayoutId, Layouts, Point, Screen,
};
use kx_module_api::{SettingsHandle, keys};
use std::sync::Arc;
use std::time::Duration;

/// Everything a click can change.
pub(super) struct Inner {
    ports: Ports,
    settings: KeyboardSettings,
    typing: Typing,
    /// The key the pointer holds, and when it repeats.
    repeat: Repeat<String>,
    sizer: Sizer,
    placer: Placer,
    /// The screens the UI last reported.
    screens: Vec<Screen>,
    /// The size-1 plate in logical pixels.
    plate: Area,
    /// How far the window reaches past the plate at size 1, as the UI last reported it: half its resize corner.
    overhang: f64,
    turn: Turn,
    /// The language list, while open.
    list: Option<LangList>,
    /// The Caps Lock state the last state view showed, so a press outside the keyboard redraws it.
    shown_caps: Option<bool>,
    /// The mouse module's state, as the bus last told it.
    assist: AssistChanged,
    /// The bus changed what the state shows since the last state view, so the next relabel redraws it.
    unshown: bool,
    /// Debug builds only: Caps Lock shows the other way round from the system's (`FaceAction::FlipCaps`).
    #[cfg(debug_assertions)]
    caps_flipped: bool,
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
            repeat: timed(&settings),
            sizer: Sizer::new(&settings, store.clone()),
            placer: Placer::new(&settings, store),
            turn: Turn::new(ports.layouts.current().label.id),
            ports,
            settings,
            screens: Vec::new(),
            plate,
            overhang: 0.0,
            list: None,
            shown_caps: None,
            assist: AssistChanged::OFF,
            unshown: false,
            #[cfg(debug_assertions)]
            caps_flipped: false,
        }
    }

    /// New services and settings; the latches and wait stay, the size and place come from the settings again. A held
    /// key is dropped, and its burst ends.
    pub(super) fn renew(
        &mut self,
        ports: Ports,
        settings: KeyboardSettings,
        store: SettingsHandle,
    ) {
        self.typing.release();
        self.repeat = timed(&settings);
        self.typing.update(&settings);
        self.sizer = Sizer::new(&settings, store.clone());
        self.placer = Placer::new(&settings, store);
        if !self.screens.is_empty() {
            self.start_place();
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

    /// Types key `id` on the layout shown, ending first a burst left open, so its latches never ride on this click.
    pub(super) fn tap(&mut self, id: &str, back: bool) -> Went {
        let ended = self.ended().state;
        let table = self.ports.layouts.current();
        let tapped = self.typing.tap(&self.ports.io(&table), id, back);
        Went { tapped, ended }
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

    /// A click on the language key: a step asks the app to switch, which a later refresh sees. With one layout
    /// installed it does nothing, except close a list that is still open.
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

    /// The size, place and pick actions; the face does taps, held keys, relabels and panel keys itself.
    pub(super) fn act(&mut self, action: FaceAction) -> Result<FaceReply, FaceError> {
        match action {
            FaceAction::Smaller => self.resized(Sizer::smaller),
            FaceAction::Bigger => self.resized(Sizer::bigger),
            FaceAction::Preset { index } => self.resized(|s| s.pick(index)),
            FaceAction::ResizeEnd => self.resized(Sizer::end),
            FaceAction::ResizeBegin { x, y } => {
                let plate = self.plate_px();
                self.sizer.begin(Point { x, y }, plate);
                Ok(FaceReply::default())
            }
            FaceAction::ResizeAt { x, y } => Ok(FaceReply {
                board: self.sizer.follow(Point { x, y }),
                ..FaceReply::default()
            }),
            FaceAction::Placed {
                rect,
                screens,
                overhang,
            } => Ok(self.placed(rect, screens, overhang)),
            FaceAction::LangPick { index } => self.pick(index),
            FaceAction::Tap { .. }
            | FaceAction::Relabel
            | FaceAction::PanelKey { .. }
            | FaceAction::Press { .. }
            | FaceAction::Tick
            | FaceAction::LetGo { .. }
            | FaceAction::Leave => Ok(FaceReply::default()),
            #[cfg(debug_assertions)]
            FaceAction::FlipCaps => {
                self.caps_flipped = !self.caps_flipped;
                Ok(STATE)
            }
        }
    }

    /// True when Caps Lock changed since the last state view showed it, as after a press on a real keyboard.
    pub(super) fn caps_moved(&self) -> bool {
        self.shown_caps.is_some_and(|shown| shown != self.caps())
    }

    /// Whether Caps Lock shows lit: the system's state, turned over in debug builds by `FaceAction::FlipCaps`.
    fn caps(&self) -> bool {
        let on = self.ports.source.caps_on();
        #[cfg(debug_assertions)]
        let on = on != self.caps_flipped;
        on
    }

    /// The settings and the size, which the board is drawn from.
    pub(super) fn sizing(&self) -> (KeyboardSettings, f64) {
        (self.settings.clone(), self.sizer.size())
    }

    /// The state of `keys`, with `panel_keys` driving an open list; a new layout is noted first, so a reader
    /// in its change event already sees the names' turn.
    pub(super) fn state(&mut self, keys: &[KeyBox], panel_keys: &[(String, Nav)]) -> StateView {
        self.seen();
        let caps = self.caps();
        self.shown_caps = Some(caps);
        self.unshown = false;
        let table = self.ports.layouts.current();
        let list = self.ports.layouts.list();
        state_view(&StateInput {
            keys,
            latches: self.typing.latches(),
            caps,
            table: &table,
            list: &list,
            turn: self.turn.turn(),
            open: self.list.as_ref(),
            panel_keys,
            at: self.placer.at(),
            assist: self.assist,
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

/// A held key's repeats at the delay and pace of `settings`.
fn timed(settings: &KeyboardSettings) -> Repeat<String> {
    let delay = Duration::from_millis(settings.repeat_delay_ms);
    Repeat::new(delay, Duration::from_millis(settings.repeat_every_ms))
}

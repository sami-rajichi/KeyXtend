//! The face's state behind its lock: typing, a held key, size and place, the language key's turn, the open list, the
//! shortcuts layer and the mouse, scroll and power modules' states.

mod assist;
mod fit;
mod held;
mod layer;
mod list;
mod pad;
mod power;

pub(super) use held::Went;

use super::STATE;
use crate::geom::{Area, KeyBox};
use crate::langkey::Turn;
use crate::layer::Layer;
use crate::panel::LangList;
use crate::place::Placer;
use crate::repeat::Repeat;
use crate::run::{Ports, Typing};
use crate::settings::{KeyboardSettings, LangKey};
use crate::sizer::Sizer;
use crate::spec::{BoardSpec, Nav};
use crate::view::{StateInput, StateView, state_view};
use kx_module_api::SettingsHandle;
use kx_module_api::ports::{
    AssistChanged, FaceAction, FaceError, FaceReply, LayoutId, Layouts, Point, PowerChanged,
    Screen, ScrollChanged, ShortcutInfo,
};
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
    /// The scroll module's state, as the bus last told it.
    scroll: ScrollChanged,
    /// The pointer is on the scroll pad.
    hover: bool,
    /// Counts the scroll module's live runs, so a module that started again hears the pointer anew.
    run: u64,
    /// The shortcuts layer, which is never saved.
    layer: Layer,
    /// The shortcuts the platform knows, with the steps the layer's tips name.
    known: Vec<ShortcutInfo>,
    /// The power module's state, as the bus last told it.
    power: PowerChanged,
    /// The bus changed what the state shows since the last state view, so the next relabel redraws it.
    unshown: bool,
}

impl Inner {
    /// Nothing latched, no list open and the layer hidden, at the saved size; placed once the UI reports its screens.
    pub(super) fn new(
        spec: &BoardSpec,
        plate: Area,
        ports: Ports,
        settings: KeyboardSettings,
        store: SettingsHandle,
    ) -> Self {
        let (layer, known) = layer::hidden(spec, &ports, &settings);
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
            scroll: ScrollChanged::OFF,
            hover: false,
            run: 0,
            layer,
            known,
            power: PowerChanged::OFF,
            unshown: false,
        }
    }

    /// New services and settings; the latches, wait and layer stay, the size and place come from the settings again. A
    /// held key is dropped, and its burst ends.
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
        self.relayer(&ports, &settings);
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
        let cells = self.layer.cells();
        let tapped = self.typing.tap(&self.ports.io(&table, &cells), id, back);
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

    /// The size, place and pick actions; the face does taps, held keys, relabels, panel keys and the pad itself.
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
            | FaceAction::Leave
            | FaceAction::Pad { .. }
            | FaceAction::PadHover { .. } => Ok(FaceReply::default()),
        }
    }

    /// True when Caps Lock changed since the last state view showed it, as after a press on a real keyboard.
    pub(super) fn caps_moved(&self) -> bool {
        self.shown_caps.is_some_and(|shown| shown != self.caps())
    }

    /// Whether Caps Lock shows lit, as the layout source says.
    fn caps(&self) -> bool {
        self.ports.source.caps_on()
    }

    /// The state of `keys`, with `panel_keys` driving an open list shown `rows` to a page; a new layout is noted
    /// first, so a reader in its change event already sees the names' turn.
    pub(super) fn state(
        &mut self,
        keys: &[KeyBox],
        panel_keys: &[(String, Nav)],
        rows: usize,
    ) -> StateView {
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
            rows,
            panel_keys,
            at: self.placer.at(),
            assist: self.assist,
            scroll: self.scroll,
            layer: &self.layer,
            power: self.power,
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

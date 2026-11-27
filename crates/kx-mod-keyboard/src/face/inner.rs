//! The face's state behind its lock: typing, a held key, size and place, the language key's turn, the open list, the
//! shortcuts layer, the other modules' states, our open panel and Snip's windows, and a fill waiting its gap.

mod aim;
mod assist;
mod caps;
mod clip;
mod edit;
mod fill;
mod fit;
mod held;
mod layer;
mod lines;
mod list;
mod pad;
mod panels;
mod power;
mod snip;
mod voice;
mod words;

pub(super) use held::Went;
pub(super) use lines::Queued;
pub(super) use words::WordTo;

use super::STATE;
use crate::geom::{Area, KeyBox};
use crate::langkey::Turn;
use crate::layer::Layer;
use crate::panel::{LangList, Panel};
use crate::place::Placer;
use crate::repeat::Repeat;
use crate::run::{Ports, Typing};
use crate::settings::{KeyboardSettings, LangKey};
use crate::sizer::Sizer;
use crate::spec::{BoardSpec, Nav};
use crate::view::{StateInput, StateView, state_view};
use aim::Aim;
use caps::CapsLook;
use fill::Waiting;
use held::timed;
use kx_module_api::SettingsHandle;
use kx_module_api::ports::{
    AssistChanged, FaceAction, FaceError, FaceReply, FillLive, LayoutId, Layouts, Point,
    PowerChanged, Screen, ScrollChanged, ShortcutInfo, VoiceLit,
};
use lines::Lines;
use snip::Snip;
use std::sync::Arc;

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
    /// Caps Lock as the UI's thread saw it, since Windows reads it per thread.
    caps: CapsLook,
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
    /// The vault module's state, as the bus last told it.
    vault: FillLive,
    /// Our panel open now, as the bus last told: the one opened last, so one at a time; Snip's windows may show beside
    /// it (`snip.rs`).
    open: Option<Panel>,
    /// Whether the line of the panel that holds the place is aimed at, and the open panel's line.
    aim: Aim,
    /// The line prediction heard last, and what waits behind a change.
    lines: Lines,
    /// The bus changed what the state shows since the last state view, so the next relabel redraws it.
    unshown: bool,
    /// A fill waiting its gap after a Tab; our keys reach only the open panel meanwhile.
    fill: Option<Waiting>,
    /// The voice module's light, as the bus last told it, so the Dictation key glows amber or red.
    voice: VoiceLit,
    /// Snip's line and light, as the bus last told them, a click on the Snip key, and whether they hold the place.
    snip: Snip,
    /// The module runs, so a picked word may type.
    running: bool,
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
            caps: CapsLook::default(),
            assist: AssistChanged::OFF,
            scroll: ScrollChanged::OFF,
            hover: false,
            run: 0,
            layer,
            known,
            power: PowerChanged::OFF,
            vault: FillLive::OFF,
            open: None,
            aim: Aim::default(),
            lines: Lines::default(),
            unshown: false,
            fill: None,
            voice: VoiceLit::Off,
            snip: Snip::default(),
            running: false,
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
        self.follow_caps();
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
            | FaceAction::PadHover { .. }
            | FaceAction::AimPanel
            | FaceAction::AimSnip => Ok(FaceReply::default()),
        }
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
        let caps = self.look_caps();
        self.caps.shown = Some(caps);
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
            fill: self.vault,
            panel: self.open,
            aim: self.aimed_at(),
            voice: self.voice,
            snip: self.snip_lit(),
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

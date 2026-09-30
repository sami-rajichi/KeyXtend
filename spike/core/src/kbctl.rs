//! The stage-2 keyboard's controller: size, theme, sticky modifiers and the layout in front. Faces draw and forward clicks.
//! The views are kx-mod-keyboard's (P3 Task 13); the spike dresses them with its text, icons and working side keys.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use kx_mod_keyboard::view::{
    BoardView, Paint, StateInput, StateView, board_view, paint, state_view,
};
use kx_mod_keyboard::{
    Board, BoardSpec, KeyKind, KeyboardSettings, Limits, Resize, Turn, Typing, board,
};
use kx_mod_layouts::Follow;
use kx_module_api::ports::{LangLabel, LayoutId, LayoutSource, LayoutTable, Point};
use kx_platform::ui::SysLook;
use kx_platform_windows::{WinLayouts, layouts};
use kx_theme::{LookView, Mode, ModeChoice, Themes, look_view, mode_for};

use crate::config::SpikeConfig;
use crate::facecfg::Action;
use crate::layout;

mod lang;
mod typing;
pub use typing::Tapped;

/// The paint of the recording mic key (mock-up `.key.rec`); the product's mic comes in a later version.
pub const REC: Paint = Paint {
    fill: "rec",
    ink: "rec_ink",
    skirt: "rec",
};

/// The keyboard's state; faces own one and ask it what to draw.
pub struct Kb {
    cfg: SpikeConfig,
    themes: Themes,
    spec: BoardSpec,
    /// The product's keyboard settings: the sizes, the rules and the language key's design.
    settings: KeyboardSettings,
    limits: Limits,
    size: f64,
    theme: usize,
    mode: ModeChoice,
    /// The product's typing state: latches and a dead key's wait.
    typing: Typing,
    toggled: BTreeSet<String>,
    follow: Follow,
    hkl: isize,
    sys: SysLook,
    resize: Option<Resize>,
    chars: BTreeMap<isize, Arc<LayoutTable>>,
    labels: BTreeMap<isize, LangLabel>,
    /// The mic is recording, so its key turns red.
    rec: bool,
    /// Which way the language names turn, as the product counts it.
    turn: Turn,
}

impl Kb {
    /// Starts on the product board at its default size, the configured theme and mode, and the layout in front.
    pub fn new(cfg: SpikeConfig, themes: Themes, sys: SysLook) -> Result<Kb, String> {
        let spec = BoardSpec::shipped().map_err(|e| e.to_string())?;
        let kb = KeyboardSettings::from_defaults().map_err(|e| e.to_string())?;
        let front = layouts::id(layout::front_layout());
        let follow = Follow::new(Some(front), &WinLayouts.installed());
        let theme = themes.list.iter().position(|t| t.look.id == cfg.look.theme);
        let typing = Typing::new(&spec, &kb);
        let hkl = follow.shown().map_or(0, |id| id.get() as isize);
        Ok(Kb {
            spec,
            limits: Limits::of(&kb),
            size: kb.size,
            settings: kb,
            theme: theme.unwrap_or_default(),
            mode: cfg.look.mode,
            typing,
            toggled: BTreeSet::new(),
            hkl,
            follow,
            sys,
            resize: None,
            chars: BTreeMap::new(),
            labels: BTreeMap::new(),
            rec: false,
            turn: Turn::new(LayoutId::new(hkl as u64)),
            cfg,
            themes,
        })
    }

    /// The settings.
    pub fn cfg(&self) -> &SpikeConfig {
        &self.cfg
    }

    /// The themes and their shared sizes.
    pub fn themes(&self) -> &Themes {
        &self.themes
    }

    /// The size now.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// The theme's gap between keys at size 1.
    fn gap(&self) -> f64 {
        let theme = self.themes.list.get(self.theme);
        theme.map_or(0.0, |t| f64::from(t.look.gap_px))
    }

    /// The keyboard's boxes now.
    pub fn board(&self) -> Board {
        board(&self.spec, self.gap(), self.size)
    }

    /// What the face lays out: the product's board with the spike's text, Lucide icons, caps and side keys.
    pub fn view(&self) -> BoardView {
        let mut v = board_view(&self.spec, &self.settings, self.size);
        let inset = self.gap() * self.size / 2.0;
        for k in &mut v.keys {
            let look = self.cfg.keys.get(&k.id);
            k.icon = self.themes.icons.keys.get(&k.id).cloned();
            k.label = look.and_then(|l| l.label.clone());
            k.name = look.map(|l| l.name.clone());
            k.cell = k.cell.grow(-inset);
            if look.is_some_and(|l| l.action.is_some()) {
                (k.dim, k.tip) = (false, None);
            }
        }
        v.dpad.area = v.dpad.area.grow(-inset);
        v
    }

    /// The theme shown, by index.
    pub fn theme(&self) -> usize {
        self.theme
    }

    /// The mode chosen.
    pub fn mode(&self) -> ModeChoice {
        self.mode
    }

    /// The look now.
    pub fn look(&self) -> Option<LookView<'_>> {
        let id = &self.themes.list.get(self.theme)?.look.id;
        look_view(
            &self.themes,
            &self.cfg.look.choice(id, self.mode),
            &self.sys,
        )
    }

    /// Windows' look changed.
    pub fn set_sys(&mut self, sys: SysLook) {
        self.sys = sys;
    }

    /// Shows theme `i`; false when there is none.
    pub fn set_theme(&mut self, i: usize) -> bool {
        let ok = i < self.themes.list.len();
        if ok {
            self.theme = i;
        }
        ok
    }

    /// Light, dark or following Windows.
    pub fn set_mode(&mut self, m: ModeChoice) {
        self.mode = m;
    }

    /// The top bar's switch: the other mode from the one shown.
    pub fn flip_mode(&mut self) {
        let dark = mode_for(self.mode, &self.sys) == Mode::Dark;
        self.mode = if dark {
            ModeChoice::Light
        } else {
            ModeChoice::Dark
        };
    }

    /// Sets the size to `s`; false when it did not change.
    fn resize_to(&mut self, s: f64) -> bool {
        let changed = (s - self.size).abs() > f64::EPSILON;
        self.size = s;
        changed
    }

    /// One size step up or down.
    pub fn step(&mut self, up: bool) -> bool {
        let l = &self.limits;
        let s = if up {
            l.up(self.size)
        } else {
            l.down(self.size)
        };
        self.resize_to(s)
    }

    /// Preset `i`: S, M or L.
    pub fn preset(&mut self, i: usize) -> bool {
        match self.limits.preset(i) {
            Some(s) => self.resize_to(s),
            None => false,
        }
    }

    /// Starts a resize with the pointer at `p`, in the same units as the plate.
    pub fn resize_begin(&mut self, p: (f64, f64)) {
        let plate = self.board().plate;
        self.resize = Some(Resize::begin(self.size, point(p), (plate.w, plate.h)));
    }

    /// Follows the pointer at `p`; true when the size changed.
    pub fn resize_at(&mut self, p: (f64, f64)) -> bool {
        match self.resize {
            Some(r) => self.resize_to(r.at(point(p), &self.limits)),
            None => false,
        }
    }

    /// Ends the resize at the size it reached; Settings will offer a size reset, never Esc (owner, 2026-09-28).
    pub fn resize_end(&mut self) {
        self.resize = None;
    }

    /// True while a resize runs.
    pub fn resizing(&self) -> bool {
        self.resize.is_some()
    }

    /// Whether the mic records, which turns its key red.
    pub fn set_rec(&mut self, rec: bool) {
        self.rec = rec;
    }

    /// Legends, lights and the language key now, with the spike's toggled side keys and recording mic.
    pub fn state(&mut self) -> StateView {
        let (table, list) = (self.table(), self.labels());
        let b = self.board();
        let mut s = state_view(&StateInput {
            keys: &b.keys,
            latches: self.typing.latches(),
            caps: WinLayouts.caps_on(),
            table: &table,
            list: &list,
            turn: self.turn.turn(),
            open: None,
            panel_keys: &[],
            at: None,
        });
        self.light(&mut s);
        s
    }

    /// Lights the toggled side keys and the recording mic, which the product leaves to later versions.
    fn light(&self, s: &mut StateView) {
        for (id, look) in &self.cfg.keys {
            let (Some(action), Some(k)) = (look.action, s.keys.get_mut(id)) else {
                continue;
            };
            let rec = self.rec && action == Action::Mic;
            k.on = rec || self.toggled.contains(id);
            k.paint = if rec { REC } else { paint(KeyKind::Act, k.on) };
        }
    }
}

/// Pointer `p` in whole pixels.
fn point((x, y): (f64, f64)) -> Point {
    Point {
        x: x.round() as i32,
        y: y.round() as i32,
    }
}

#[cfg(test)]
mod tests;

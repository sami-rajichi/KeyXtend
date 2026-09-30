//! The stage-2 keyboard's controller: size, theme, sticky modifiers and the layout in front. Faces draw and forward clicks.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use kx_mod_keyboard::{Board, BoardSpec, KeyboardSettings, Limits, Resize, Typing, board};
use kx_mod_layouts::Follow;
use kx_module_api::ports::{LangLabel, LayoutId, LayoutSource, LayoutTable, Point};
use kx_platform::ui::SysLook;
use kx_platform_windows::{WinLayouts, layouts};
use kx_theme::{LookView, Mode, ModeChoice, Themes, look_view, mode_for};

use crate::config::SpikeConfig;
use crate::kbview::{BoardView, StateInput, StateView, board_view, state_view};
use crate::langinfo::{self, LangInfo};
use crate::{langkey, layout};

mod typing;
pub use typing::Tapped;

/// The keyboard's state; faces own one and ask it what to draw.
pub struct Kb {
    cfg: SpikeConfig,
    themes: Themes,
    spec: BoardSpec,
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
    langs: BTreeMap<isize, LangInfo>,
    /// The mic is recording, so its key turns red.
    rec: bool,
    /// Which way the language key last stepped, until the layout changes: -1 back, 1 on.
    asked: i8,
    /// Which way the language names turn after the last change; 0 when it came another way.
    turn: i8,
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
        Ok(Kb {
            spec,
            limits: Limits::of(&kb),
            size: kb.size,
            theme: theme.unwrap_or_default(),
            mode: cfg.look.mode,
            typing,
            toggled: BTreeSet::new(),
            hkl: follow.shown().map_or(0, |id| id.get() as isize),
            follow,
            sys,
            resize: None,
            chars: BTreeMap::new(),
            langs: BTreeMap::new(),
            rec: false,
            asked: 0,
            turn: 0,
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

    /// The keyboard's boxes now.
    pub fn board(&self) -> Board {
        let gap = self
            .themes
            .list
            .get(self.theme)
            .map_or(0.0, |t| t.look.gap_px);
        board(&self.spec, f64::from(gap), self.size)
    }

    /// What the face lays out.
    pub fn view(&self) -> BoardView {
        board_view(&self.board(), &self.cfg.keys, self.size, &self.limits)
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

    /// The current layout as the product names it.
    fn layout_id(&self) -> LayoutId {
        LayoutId::new(self.hkl as u64)
    }

    /// What each key types in the current layout, read once per layout.
    fn table(&mut self) -> Arc<LayoutTable> {
        let (id, spec) = (self.layout_id(), &self.spec);
        let table = self.chars.entry(self.hkl).or_insert_with(|| {
            let scans = spec.char_scans();
            let keys = scans.iter().copied().zip(WinLayouts.keys(id, &scans));
            let label = LangLabel {
                id,
                ..LangLabel::default()
            };
            let keys = keys.collect();
            Arc::new(LayoutTable { label, keys })
        });
        Arc::clone(table)
    }

    /// The language key's names for layout `hkl`, read once.
    fn lang_of(&mut self, hkl: isize) -> LangInfo {
        let c = &self.cfg.lang;
        self.langs
            .entry(hkl)
            .or_insert_with(|| langinfo::info(hkl, c))
            .clone()
    }

    /// Whether the mic records, which turns its key red.
    pub fn set_rec(&mut self, rec: bool) {
        self.rec = rec;
    }

    /// Legends, lights and the language key now.
    pub fn state(&mut self) -> StateView {
        let (prev, next) = langkey::neighbours(&langkey::installed(), self.hkl);
        let lang = [
            self.lang_of(prev),
            self.lang_of(self.hkl),
            self.lang_of(next),
        ];
        let table = self.table();
        let b = self.board();
        state_view(&StateInput {
            keys: &b.keys,
            looks: &self.cfg.keys,
            latches: self.typing.latches(),
            caps: WinLayouts.caps_on(),
            toggled: &self.toggled,
            chars: &table.keys,
            lang: [&lang[0], &lang[1], &lang[2]],
            rec: self.rec,
            turn: self.turn,
        })
    }

    /// The layout is now `h`; the names turn the way the last step asked, and only once.
    fn switched(&mut self, h: isize) {
        self.hkl = h;
        self.turn = std::mem::take(&mut self.asked);
    }

    /// True when the app in front switched layout since the last look.
    pub fn relabel(&mut self) -> bool {
        let moved = self
            .follow
            .see(layout::other_front(), &WinLayouts.installed());
        if let Some(id) = self.follow.shown().filter(|_| moved) {
            self.switched(id.get() as isize);
        }
        moved
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

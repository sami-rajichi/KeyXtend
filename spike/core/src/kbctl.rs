//! The stage-2 keyboard's controller: size, theme, sticky modifiers and the layout in front. Faces draw and forward clicks.

use std::collections::{BTreeMap, BTreeSet};

use windows::Win32::UI::Input::KeyboardAndMouse::HKL;

use crate::config::SpikeConfig;
use crate::kbgeom::{Board, board};
use crate::kbview::{BoardView, StateInput, StateView, board_view, state_view};
use crate::langinfo::{self, LangInfo};
use crate::latch::Latches;
use crate::legend::KeyChars;
use crate::lookcfg::ModeChoice;
use crate::lookview::{LookView, look_view};
use crate::sizer::{self, Resize};
use crate::sysui::{self, SystemLook};
use crate::theme::Themes;
use crate::{langkey, layout};

mod typing;
pub use typing::{Plan, Tapped};

/// The keyboard's state; faces own one and ask it what to draw.
pub struct Kb {
    cfg: SpikeConfig,
    themes: Themes,
    size: f32,
    theme: usize,
    mode: ModeChoice,
    latches: Latches,
    toggled: BTreeSet<String>,
    follow: layout::Follow,
    hkl: isize,
    sys: SystemLook,
    resize: Option<Resize>,
    chars: BTreeMap<isize, Vec<(u32, KeyChars)>>,
    langs: BTreeMap<isize, LangInfo>,
    /// A dead key was sent, so the next character goes as a key press for the app to add the accent.
    dead: bool,
    /// The mic is recording, so its key turns red.
    rec: bool,
    /// Which way the language key last stepped, until the layout changes: -1 back, 1 on.
    asked: i8,
    /// Which way the language names turn after the last change; 0 when it came another way.
    turn: i8,
}

impl Kb {
    /// Starts at the first size preset, the configured theme and mode, and the layout of the app in front.
    pub fn new(cfg: SpikeConfig, themes: Themes, sys: SystemLook) -> Kb {
        let hkl = layout::foreground_layout();
        let theme = themes.list.iter().position(|t| t.look.id == cfg.look.theme);
        Kb {
            size: cfg.size.start(),
            theme: theme.unwrap_or_default(),
            mode: cfg.look.mode,
            latches: Latches::default(),
            toggled: BTreeSet::new(),
            follow: layout::Follow::new(hkl),
            hkl: hkl.0 as isize,
            sys,
            resize: None,
            chars: BTreeMap::new(),
            langs: BTreeMap::new(),
            dead: false,
            rec: false,
            asked: 0,
            turn: 0,
            cfg,
            themes,
        }
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
    pub fn size(&self) -> f32 {
        self.size
    }

    /// The keyboard's boxes now.
    pub fn board(&self) -> Board {
        let gap = self
            .themes
            .list
            .get(self.theme)
            .map_or(0.0, |t| t.look.gap_px);
        board(&self.cfg.layout, gap, self.size)
    }

    /// What the face lays out.
    pub fn view(&self) -> BoardView {
        board_view(&self.board(), &self.cfg.keys, self.size, &self.cfg.size)
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
        look_view(
            &self.themes,
            self.theme,
            self.mode,
            &self.sys,
            &self.cfg.look,
        )
    }

    /// Windows' look changed.
    pub fn set_sys(&mut self, sys: SystemLook) {
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
        let dark = sysui::mode_for(self.mode, &self.sys) == crate::theme::Mode::Dark;
        self.mode = if dark {
            ModeChoice::Light
        } else {
            ModeChoice::Dark
        };
    }

    /// Sets the size to `s`; false when it did not change.
    fn resize_to(&mut self, s: f32) -> bool {
        let changed = (s - self.size).abs() > f32::EPSILON;
        self.size = s;
        changed
    }

    /// One size step up or down.
    pub fn step(&mut self, up: bool) -> bool {
        let c = &self.cfg.size;
        let s = if up {
            sizer::step_up(self.size, c)
        } else {
            sizer::step_down(self.size, c)
        };
        self.resize_to(s)
    }

    /// Preset `i`: S, M or L.
    pub fn preset(&mut self, i: usize) -> bool {
        match self.cfg.size.presets.get(i).copied() {
            Some(s) => self.resize_to(s),
            None => false,
        }
    }

    /// Starts a resize with the pointer at `p`, in the same units as the plate.
    pub fn resize_begin(&mut self, p: (f32, f32)) {
        let plate = self.board().plate;
        self.resize = Some(Resize::begin(self.size, p, (plate.w, plate.h)));
    }

    /// Follows the pointer at `p`; true when the size changed.
    pub fn resize_at(&mut self, p: (f32, f32)) -> bool {
        match self.resize {
            Some(r) => self.resize_to(r.at(p, &self.cfg.size)),
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

    /// The current layout's handle.
    fn layout(&self) -> HKL {
        HKL(self.hkl as *mut core::ffi::c_void)
    }

    /// Reads what each key types in the current layout, once per layout.
    fn load_chars(&mut self) {
        let (hkl, keys) = (self.layout(), &self.cfg.layout);
        self.chars.entry(self.hkl).or_insert_with(|| {
            let codes = keys.rows.iter().flatten().filter_map(|i| match i {
                crate::kbgeom::RowItem::Chars(c) => Some(c.chars.clone()),
                crate::kbgeom::RowItem::Key(_) => None,
            });
            codes
                .flatten()
                .map(|sc| (sc, layout::key_chars(sc, hkl)))
                .collect()
        });
    }

    /// What each key types in the current layout; empty before `load_chars`.
    fn chars_here(&self) -> &[(u32, KeyChars)] {
        self.chars.get(&self.hkl).map_or(&[], Vec::as_slice)
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
        self.load_chars();
        let b = self.board();
        state_view(&StateInput {
            keys: &b.keys,
            looks: &self.cfg.keys,
            latches: &self.latches,
            caps: layout::caps_on(),
            toggled: &self.toggled,
            chars: self.chars_here(),
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
        match self.follow.changed() {
            Some(h) => {
                self.switched(h.0 as isize);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests;

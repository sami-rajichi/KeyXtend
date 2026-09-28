//! What the faces draw: each key's box, look and name at one size, and the legends and states that change as keys are used.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::facecfg::KeysConfig;
use crate::kbgeom::{Board, KeyBox, KeyKind};
use crate::langinfo::LangInfo;
use crate::latch::{Latch, Latches};
use crate::legend::{KeyChars, Legend, LegendMods, is_arabic, legend, typed};
use crate::paint::{Paint, paint};
use crate::place::Place;
use crate::sizer::SizeConfig;

/// One key as the face lays it out; it changes only with the size.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeyView {
    /// Id, which keys its state.
    pub id: String,
    /// Kind, which sets its colours.
    pub kind: KeyKind,
    /// The cap's box.
    #[serde(flatten)]
    pub cap: Place,
    /// Lucide icon name.
    pub icon: Option<String>,
    /// Text drawn instead of an icon.
    pub label: Option<String>,
    /// Name for screen readers; character keys take theirs from the state.
    pub name: String,
    /// In the side block.
    pub side: bool,
    /// Its colours while off.
    pub paint: Paint,
}

/// The keyboard at one size.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BoardView {
    /// The plate.
    pub plate: Place,
    /// The D-pad's round face.
    pub dpad: Place,
    /// Every key.
    pub keys: Vec<KeyView>,
    /// The top bar.
    pub bar: Place,
    /// The size factor.
    pub size: f32,
    /// The size preset this size is, if any.
    pub preset: Option<usize>,
}

/// One key's changing part: legends for character keys, and whether it is on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct KeyState {
    /// Legends; empty for named keys.
    #[serde(flatten)]
    pub legend: Legend,
    /// What a character key types now, for screen readers.
    pub name: String,
    /// Latched, locked or toggled on.
    pub on: bool,
    /// Its colours now.
    pub paint: Paint,
}

/// The language key: the layouts before, now and after, by short name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LangView {
    /// Previous layout.
    pub prev: String,
    /// The layout of the app in front.
    pub cur: String,
    /// Next layout.
    pub next: String,
    /// Which of the three names are Arabic, for the font.
    pub ar: [bool; 3],
    /// The current layout is right to left.
    pub rtl: bool,
    /// The space bar text.
    pub space: String,
}

/// Everything that changes as keys are used.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StateView {
    /// Each key's state, by id.
    pub keys: BTreeMap<String, KeyState>,
    /// The language key and space bar.
    pub lang: LangView,
}

/// The keyboard `b` at `size` as the face draws it, with the named keys' looks from `looks`.
pub fn board_view(b: &Board, looks: &KeysConfig, size: f32, s: &SizeConfig) -> BoardView {
    let view = |k: &KeyBox| {
        let look = looks.get(&k.id);
        let side = k.kind == KeyKind::Act;
        KeyView {
            id: k.id.clone(),
            kind: k.kind,
            cap: k.cap,
            icon: look.and_then(|l| l.icon.clone()),
            label: look.and_then(|l| l.label.clone()),
            name: look.map(|l| l.name.clone()).unwrap_or_default(),
            side,
            paint: paint(k.kind, side, false),
        }
    };
    BoardView {
        plate: b.plate,
        dpad: b.dpad,
        keys: b.keys.iter().map(view).collect(),
        bar: b.bar,
        size,
        preset: s
            .presets
            .iter()
            .position(|p| (p - size).abs() < f32::EPSILON),
    }
}

/// What the state is made from.
pub struct StateInput<'a> {
    /// Every key.
    pub keys: &'a [KeyBox],
    /// The named keys' looks.
    pub looks: &'a KeysConfig,
    /// Latched modifiers.
    pub latches: &'a Latches,
    /// Caps Lock is on.
    pub caps: bool,
    /// Side keys toggled on.
    pub toggled: &'a BTreeSet<String>,
    /// What each character key types in the current layout, by scan code.
    pub chars: &'a [(u32, KeyChars)],
    /// The current layout and its neighbours.
    pub lang: [&'a LangInfo; 3],
}

/// The legend modifiers from `latches` and Caps Lock: AltGr counts only on layouts that have AltGr characters.
pub fn legend_mods(latches: &Latches, caps: bool, all: &[(u32, KeyChars)]) -> LegendMods {
    let has_altgr = all.iter().any(|(_, c)| !c.altgr.is_empty());
    LegendMods {
        shift: latches.is_on(Latch::Shift),
        caps,
        altgr: has_altgr && latches.is_on(Latch::AltGr),
    }
}

/// What key `sc` types in `all`; nothing when the layout lacks it.
pub fn chars_of(all: &[(u32, KeyChars)], sc: u32) -> KeyChars {
    let found = all.iter().find(|(c, _)| *c == sc);
    found.map(|(_, k)| k.clone()).unwrap_or_default()
}

/// Every key's state and the language key.
pub fn state_view(i: &StateInput) -> StateView {
    let m = legend_mods(i.latches, i.caps, i.chars);
    let rtl = i.lang[1].rtl;
    let mut keys = BTreeMap::new();
    for k in i.keys {
        let found = i
            .chars
            .iter()
            .find(|(sc, _)| Some(*sc) == k.sc && k.kind == KeyKind::Char);
        let s = match found {
            Some((_, c)) => KeyState {
                legend: legend(c, m, k.row == 0, rtl),
                name: typed(c, m).to_string(),
                on: false,
                paint: paint(k.kind, false, false),
            },
            None => {
                let on = is_on(i, &k.id);
                let side = k.kind == KeyKind::Act;
                KeyState {
                    on,
                    paint: paint(k.kind, side, on),
                    ..KeyState::default()
                }
            }
        };
        keys.insert(k.id.clone(), s);
    }
    StateView {
        keys,
        lang: lang_view(i.lang),
    }
}

/// The language key's three names and the space bar text.
fn lang_view([prev, cur, next]: [&LangInfo; 3]) -> LangView {
    LangView {
        prev: prev.short.clone(),
        cur: cur.short.clone(),
        next: next.short.clone(),
        ar: [&prev.short, &cur.short, &next.short].map(|s| is_arabic(s)),
        rtl: cur.rtl,
        space: cur.name.clone(),
    }
}

/// True when named key `id` is latched, Caps Lock while it is on, or a toggled side key.
fn is_on(i: &StateInput, id: &str) -> bool {
    let look = i.looks.get(id);
    let latched = look
        .and_then(|l| l.latch)
        .is_some_and(|l| i.latches.is_on(l));
    let caps = look.is_some_and(|l| l.caps) && i.caps;
    latched || caps || i.toggled.contains(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kbgeom::board;
    use crate::legend::testkeys::{letter, plain};

    fn cfg() -> crate::config::SpikeConfig {
        crate::config::load().expect("spike.toml loads")
    }

    /// A tiny layout: 1 and !, e with € on AltGr, and x on every other key.
    fn key(sc: u32) -> KeyChars {
        match sc {
            0x02 => plain("1", "!", ""),
            0x12 => KeyChars {
                altgr: "€".into(),
                ..letter("e", "E")
            },
            _ => letter("x", "X"),
        }
    }

    fn state(latches: &Latches, caps: bool) -> StateView {
        let c = cfg();
        let b = board(&c.layout, 4.0, 1.0);
        let lang = LangInfo {
            short: "EN".into(),
            name: "English".into(),
            rtl: false,
        };
        let toggled = BTreeSet::from(["grab".to_string()]);
        let chars: Vec<_> = b
            .keys
            .iter()
            .filter(|k| k.kind == KeyKind::Char)
            .filter_map(|k| k.sc.map(|sc| (sc, key(sc))))
            .collect();
        let i = StateInput {
            keys: &b.keys,
            looks: &c.keys,
            latches,
            caps,
            toggled: &toggled,
            chars: &chars,
            lang: [&lang, &lang, &lang],
        };
        state_view(&i)
    }

    #[test]
    fn the_view_gives_every_key_its_box_and_look() {
        let c = cfg();
        let b = board(&c.layout, 4.0, 1.25);
        let v = board_view(&b, &c.keys, 1.25, &c.size);
        assert_eq!(v.preset, Some(1), "M");
        assert_eq!(v.keys.len(), b.keys.len());
        let esc = v.keys.iter().find(|k| k.id == "esc").expect("esc");
        assert_eq!(esc.label.as_deref(), Some("Esc"));
        let mic = v.keys.iter().find(|k| k.id == "mic").expect("mic");
        assert!(mic.side && mic.icon.as_deref() == Some("mic"));
    }

    #[test]
    fn a_latched_shift_lights_both_shift_keys_and_shifts_the_legends() {
        let mut m = Latches::default();
        m.toggle(Latch::Shift, 0x2A);
        let s = state(&m, false);
        assert!(s.keys["shift"].on && s.keys["rshift"].on);
        assert_eq!(s.keys["c-0-1"].legend.main, "!", "the digit row shifts");
        assert_eq!(s.keys["c-1-2"].name, "E");
    }

    #[test]
    fn caps_lights_its_key_and_toggled_side_keys_light_up() {
        let s = state(&Latches::default(), true);
        assert!(s.keys["caps"].on && s.keys["grab"].on);
        assert!(!s.keys["mic"].on);
        assert_eq!(s.keys["c-1-2"].legend.main, "E");
    }

    #[test]
    fn altgr_counts_when_the_layout_has_altgr_characters() {
        let mut m = Latches::default();
        m.toggle(Latch::AltGr, 0xE038);
        let s = state(&m, false);
        assert_eq!(s.keys["c-1-2"].legend.main, "€");
        assert_eq!(s.lang.space, "English");
    }
}

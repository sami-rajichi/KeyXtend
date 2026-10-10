//! Any run of clicks on the board, in both latch modes: "release" lets every latch go after the next key,
//! "stay" keeps each until it is clicked again, an empty key keeps them, and no key ever holds a modifier that was
//! not latched.

use crate::typing::Rig;
use kx_mod_keyboard::legend::is_empty;
use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{Latch, LegendMods, Modifiers};
use kx_module_api::ports::{LayoutSource, Scan};
use kx_platform_fake::Stroke;
use proptest::prelude::*;
use std::collections::BTreeMap;

const ALL: [Latch; 5] = [
    Latch::Shift,
    Latch::Ctrl,
    Latch::Alt,
    Latch::AltGr,
    Latch::Win,
];

/// What a click on a key does to the latches.
#[derive(Copy, Clone, Debug)]
enum Kind {
    /// Latches or lets go of a modifier, from a key sending this code.
    Latch(Latch, u16),
    /// Sends one stroke, which uses the latches.
    Key,
    /// A character key with this scan code: one stroke, or nothing while it is empty.
    Char(u16),
    /// Taps Caps Lock, which keeps the latches.
    Caps,
    /// Sends nothing.
    Quiet,
}

/// The keys a run clicks, with what each does.
fn keys() -> Vec<(String, Kind)> {
    let latch = |id: &str, l, code| (id.to_owned(), Kind::Latch(l, code));
    let chars =
        [0x10, 0x11, 0x1E, 0x1F, 0x2E, 0x02].map(|c| (char_id(Scan::new(c)), Kind::Char(c)));
    let named = ["esc", "space", "left", "bksp", "enter"].map(|id| (id.to_owned(), Kind::Key));
    let mut all = vec![
        latch("shift", Latch::Shift, 0x2A),
        latch("rshift", Latch::Shift, 0x36),
        latch("ctrl", Latch::Ctrl, 0x1D),
        latch("alt", Latch::Alt, 0x38),
        latch("altgr", Latch::AltGr, 0xE038),
        latch("win", Latch::Win, 0xE05B),
        ("caps".to_owned(), Kind::Caps),
        ("lang".to_owned(), Kind::Quiet),
        ("mic".to_owned(), Kind::Quiet),
    ];
    all.extend(chars);
    all.extend(named);
    all
}

/// The modifier codes each stroke of `strokes` holds.
fn held(strokes: &[Stroke]) -> Vec<Scan> {
    let chords = strokes.iter().filter_map(|s| match s {
        Stroke::Chord(held, _) => Some(held.clone()),
        Stroke::Tap(_) | Stroke::Text(..) => None,
    });
    chords.flatten().collect()
}

/// True when character key `code` is empty now, by the core's one rule.
fn empty(r: &Rig, code: u16) -> bool {
    let mods = LegendMods::of(r.typing.latches(), r.source.caps_on(), &r.table.keys);
    let key = r.table.keys.get(&Scan::new(code));
    key.is_some_and(|k| is_empty(k, mods))
}

/// Clicks `picks` of `keys()` on `layout` in `mode`, checking each click against a model of the latches.
fn run(layout: &str, mode: Modifiers, picks: &[usize]) -> Result<(), TestCaseError> {
    let keys = keys();
    let mut r = Rig::new(layout, |s| s.modifiers = mode);
    let mut model: BTreeMap<Latch, Scan> = BTreeMap::new();
    for &i in picks {
        let (id, kind) = &keys[i % keys.len()];
        let quiet = matches!(*kind, Kind::Char(code) if empty(&r, code));
        r.tap(id).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let sent = r.sent();
        let latched: Vec<Scan> = model.values().copied().collect();
        prop_assert!(held(&sent).iter().all(|k| latched.contains(k)), "{}", id);
        match *kind {
            Kind::Latch(l, code) => {
                prop_assert!(sent.is_empty());
                if model.remove(&l).is_none() {
                    model.insert(l, Scan::new(code));
                }
            }
            Kind::Char(_) if quiet => prop_assert!(sent.is_empty(), "{}: empty", id),
            Kind::Key | Kind::Char(_) => {
                prop_assert_eq!(sent.len(), 1);
                if mode == Modifiers::Release {
                    model.clear();
                }
            }
            Kind::Caps => prop_assert_eq!(sent.len(), 1),
            Kind::Quiet => prop_assert!(sent.is_empty()),
        }
        for l in ALL {
            prop_assert_eq!(
                r.typing.latches().is_on(l),
                model.contains_key(&l),
                "{:?}",
                l
            );
        }
    }
    Ok(())
}

fn picks() -> impl Strategy<Value = Vec<usize>> {
    prop::collection::vec(0..keys().len(), 0..30)
}

/// The layouts a run takes: English, French with its dead keys, and Arabic, whose س types nothing with Shift.
const LAYOUTS: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

proptest! {
    #[test]
    fn release_lets_every_latch_go_after_the_next_key(p in picks(), lay in 0..LAYOUTS.len()) {
        run(LAYOUTS[lay], Modifiers::Release, &p)?;
    }

    #[test]
    fn stay_keeps_every_latch_until_it_is_clicked_again(p in picks(), lay in 0..LAYOUTS.len()) {
        run(LAYOUTS[lay], Modifiers::Stay, &p)?;
    }
}

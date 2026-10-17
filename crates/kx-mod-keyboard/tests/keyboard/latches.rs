//! Any run of key and mouse clicks, in both latch modes: "release" lets every latch go after the next key or click,
//! "stay" keeps each until it is clicked again, and an empty key keeps them. No key holds a modifier that was not
//! latched, and only Win, let go with nothing used since it latched, sends itself alone.

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
    /// Latches or lets go of a modifier, from a key sending this code; a lone one sends itself when let go unused.
    Latch(Latch, u16, bool),
    /// Sends one stroke, which uses the latches.
    Key,
    /// A character key with this scan code: one stroke, or nothing while it is empty.
    Char(u16),
    /// Taps Caps Lock, which keeps the latches.
    Caps,
    /// Sends nothing.
    Quiet,
    /// A mouse click with the latches held (modifier + click), which uses them.
    Click,
}

/// The keys a run clicks, and the mouse, with what each does.
fn keys() -> Vec<(String, Kind)> {
    let latch = |id: &str, l, code| (id.to_owned(), Kind::Latch(l, code, l == Latch::Win));
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
        ("mouse".to_owned(), Kind::Click),
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

/// The latches on, each with its key's code and whether anything used it since it latched.
type Model = BTreeMap<Latch, (Scan, bool)>;

/// Marks every latch on in the model as used.
fn spend(model: &mut Model) {
    for (_, used) in model.values_mut() {
        *used = true;
    }
}

/// Something went out that uses the latches: each is used, and `release` lets them all go.
fn used(model: &mut Model, release: bool) {
    spend(model);
    if release {
        model.clear();
    }
}

/// A click on a latch key with `code`: what it must send, as the model says, and the model after it.
/// A lone press uses the other latches as any key does, but goes out without them.
fn toggle(model: &mut Model, l: Latch, code: u16, lone: bool, release: bool) -> Vec<Stroke> {
    match model.remove(&l) {
        Some((_, false)) if lone => {
            used(model, release);
            vec![Stroke::Tap(Scan::new(code))]
        }
        Some(_) => Vec::new(),
        None => {
            spend(model);
            model.insert(l, (Scan::new(code), false));
            Vec::new()
        }
    }
}

/// Checks every latch of `r` against the model: on, and unused since it latched.
fn agrees(r: &Rig, model: &Model) -> Result<(), TestCaseError> {
    let latches = r.typing.latches();
    for l in ALL {
        let on = model.get(&l);
        prop_assert_eq!(latches.is_on(l), on.is_some(), "{:?}", l);
        let unused = on.is_some_and(|&(_, used)| !used);
        prop_assert_eq!(latches.unused(l), unused, "{:?} unused", l);
    }
    Ok(())
}

/// Clicks key `id`, or the mouse for `Kind::Click`: what went out, and for the mouse whether any latch let go.
fn click(r: &mut Rig, id: &str, kind: Kind) -> Result<(Vec<Stroke>, Option<bool>), TestCaseError> {
    let freed = if matches!(kind, Kind::Click) {
        Some(r.typing.clicked())
    } else {
        r.tap(id).map_err(|e| TestCaseError::fail(e.to_string()))?;
        None
    };
    Ok((r.sent(), freed))
}

/// Clicks `picks` of `keys()` on `layout` in `mode`, checking each click against a model of the latches.
fn run(layout: &str, mode: Modifiers, picks: &[usize]) -> Result<(), TestCaseError> {
    let keys = keys();
    let release = mode == Modifiers::Release;
    let mut r = Rig::new(layout, |s| s.modifiers = mode);
    let mut model = Model::new();
    for &i in picks {
        let (id, kind) = &keys[i % keys.len()];
        let quiet = matches!(*kind, Kind::Char(code) if empty(&r, code));
        let (sent, freed) = click(&mut r, id, *kind)?;
        let latched: Vec<Scan> = model.values().map(|&(key, _)| key).collect();
        prop_assert!(held(&sent).iter().all(|k| latched.contains(k)), "{}", id);
        match *kind {
            Kind::Latch(l, code, lone) => {
                prop_assert_eq!(sent, toggle(&mut model, l, code, lone, release), "{}", id);
            }
            Kind::Char(_) if quiet => prop_assert!(sent.is_empty(), "{}: empty", id),
            Kind::Key | Kind::Char(_) => {
                prop_assert_eq!(sent.len(), 1);
                used(&mut model, release);
            }
            Kind::Click => {
                prop_assert!(sent.is_empty(), "a mouse click sends no key");
                prop_assert_eq!(freed, Some(release && !model.is_empty()));
                used(&mut model, release);
            }
            Kind::Caps => {
                prop_assert_eq!(sent.len(), 1);
                spend(&mut model);
            }
            Kind::Quiet => prop_assert!(sent.is_empty()),
        }
        agrees(&r, &model)?;
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

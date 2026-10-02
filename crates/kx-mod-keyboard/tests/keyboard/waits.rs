//! Any run of clicks on French AZERTY with its dead keys (`^` and `AltGr` `~`): the dead-key wait starts on a dead
//! key, ends on a key that types, is kept only by a key that types nothing, and a latch or Caps Lock leaves it.
//! Every Unicode send is the key's legend without its dotted circle, so a key types what it shows.

use crate::typing::Rig;
use kx_mod_keyboard::legend::{legend, slot};
use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{Latch, LegendMods, Modifiers};
use kx_module_api::ports::{LayoutSource, Scan};
use kx_platform_fake::Stroke;
use proptest::prelude::*;

/// The French layout, whose `^` key is dead and whose `é` key has a dead `~` on `AltGr`.
const LAYOUT: &str = "fr-fr";
/// What a legend draws under a mark that has no letter before it.
const DOTTED_CIRCLE: char = '\u{25CC}';

/// What a click on a key is.
#[derive(Copy, Clone, Debug)]
enum Kind {
    /// Latches or lets go of a modifier.
    Latch,
    /// Taps Caps Lock, which Windows then turns on or off.
    Caps,
    /// A character key with this scan code.
    Char(u16),
    /// A named key that sends its scan code.
    Named,
}

/// What a click does to the wait, read before it.
#[derive(Copy, Clone, Debug)]
enum Rule {
    /// It stays as it was.
    Keep,
    /// A dead key starts it.
    Start,
    /// A key that may type nothing keeps it only if it typed nothing.
    Ask,
    /// A key that types ends it.
    End,
}

/// The keys a run clicks: the two dead keys, letters, a digit key, the ISO key, latches, named keys and arrows,
/// which type nothing and so keep a wait.
fn keys() -> Vec<(String, Kind)> {
    let codes = [0x1A, 0x03, 0x10, 0x12, 0x16, 0x02, 0x56];
    let chars = codes.map(|c| (char_id(Scan::new(c)), Kind::Char(c)));
    let named =
        ["space", "esc", "bksp", "enter", "left", "up"].map(|id| (id.to_owned(), Kind::Named));
    let latches = ["shift", "altgr", "ctrl"].map(|id| (id.to_owned(), Kind::Latch));
    let mut all = vec![("caps".to_owned(), Kind::Caps)];
    all.extend(chars);
    all.extend(named);
    all.extend(latches);
    all
}

/// The rule a click of `kind` follows with the latches now and `mods`, as the plan reads the layout.
fn rule(r: &Rig, kind: Kind, mods: LegendMods) -> Rule {
    let on = |l| r.typing.latches().is_on(l);
    let right_alt = on(Latch::AltGr) && !mods.altgr;
    let shortcut = on(Latch::Ctrl) || on(Latch::Alt) || on(Latch::Win) || right_alt;
    let key = match kind {
        Kind::Latch | Kind::Caps => return Rule::Keep,
        Kind::Named => return Rule::Ask,
        Kind::Char(code) => r.table.keys.get(&Scan::new(code)),
    };
    match key {
        Some(k) if !shortcut => {
            let s = slot(k, mods);
            if k.is_dead(s) {
                Rule::Start
            } else if k.get(s).is_empty() {
                Rule::Ask
            } else {
                Rule::End
            }
        }
        _ => Rule::Ask,
    }
}

/// True when the one stroke in `sent` typed anything in the layout, as the layout source reads it.
fn typed(r: &Rig, sent: &[Stroke]) -> bool {
    let layout = r.table.label.id;
    match sent {
        [Stroke::Tap(key)] => r.source.types_any(layout, *key, &[]),
        [Stroke::Chord(held, key)] => r.source.types_any(layout, *key, held),
        _ => true,
    }
}

/// The legend key `kind` shows with `mods`, without its dotted circle; empty for a key that is not a character.
fn bare_legend(r: &Rig, kind: Kind, mods: LegendMods) -> String {
    let Kind::Char(code) = kind else {
        return String::new();
    };
    let shown = r
        .table
        .keys
        .get(&Scan::new(code))
        .map(|k| legend(k, mods, false, false).main);
    shown
        .unwrap_or_default()
        .chars()
        .filter(|&c| c != DOTTED_CIRCLE)
        .collect()
}

/// Clicks `picks` of `keys()` with `latching` as the modifiers setting, checking the wait and each Unicode send
/// against the model.
fn run(latching: Modifiers, picks: &[usize]) -> Result<(), TestCaseError> {
    let keys = keys();
    let mut r = Rig::new(LAYOUT, |s| s.modifiers = latching);
    for &i in picks {
        let (id, kind) = keys[i % keys.len()].clone();
        let before = r.typing.waiting();
        let mods = LegendMods::of(r.typing.latches(), r.source.caps_on(), &r.table.keys);
        let (rule, shows) = (rule(&r, kind, mods), bare_legend(&r, kind, mods));
        r.tap(&id).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let sent = r.sent();
        if matches!(kind, Kind::Caps) {
            r.source.set_caps(!r.source.caps_on());
        }
        if let [Stroke::Text(text, _)] = sent.as_slice() {
            prop_assert!(!before, "{}: text while a dead key waits", id);
            prop_assert_eq!(text.expose(), &shows, "{}", id);
        }
        let want = match rule {
            Rule::Keep => before,
            Rule::Start => true,
            Rule::Ask => before && !typed(&r, &sent),
            Rule::End => false,
        };
        prop_assert_eq!(r.typing.waiting(), want, "{} by {:?}", id, rule);
    }
    Ok(())
}

fn picks() -> impl Strategy<Value = Vec<usize>> {
    prop::collection::vec(0..keys().len(), 0..40)
}

proptest! {
    #[test]
    fn the_dead_key_wait_and_each_text_follow_the_model(p in picks(), stay in any::<bool>()) {
        run(if stay { Modifiers::Stay } else { Modifiers::Release }, &p)?;
    }
}

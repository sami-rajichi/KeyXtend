//! What a character key gives the clipboard panel's search: the text it would type now in every golden layout, with
//! Shift, Caps Lock and `AltGr`; nothing for a dead or empty key; and with Ctrl, Alt or Win (or `AltGr` as Right Alt)
//! it types as usual.

use super::{Clip, text, unicode};
use crate::host::GOLDEN;
use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{BoardSpec, Io, KeyboardSettings, LegendMods, ToPanel, Typing};
use kx_module_api::ports::{LayoutTable, PanelKey, Scan, TextRoute};
use kx_platform_fake::{FakeInjector, FakeLayoutSource, FakeShortcuts, LayoutFile, Stroke};

/// The latches clicked and whether Caps Lock is on: none, Shift, Caps Lock, `AltGr`, then Shift with `AltGr`.
const STATES: [(&[&str], bool); 5] = [
    (&[], false),
    (&["shift"], false),
    (&[], true),
    (&["altgr"], false),
    (&["shift", "altgr"], false),
];

/// One golden layout's fakes, shared by fresh typing states.
struct Desk {
    injector: FakeInjector,
    source: FakeLayoutSource,
    table: LayoutTable,
    shortcuts: FakeShortcuts,
}

impl Desk {
    fn new(layout: &str, caps: bool) -> Self {
        let source = FakeLayoutSource::golden(&[layout]).unwrap();
        source.set_caps(caps);
        Self {
            injector: FakeInjector::new(),
            source,
            table: LayoutTable::from(&LayoutFile::golden(layout).unwrap()),
            shortcuts: FakeShortcuts::new(),
        }
    }

    fn io(&self) -> Io<'_> {
        Io {
            injector: &self.injector,
            source: &self.source,
            table: &self.table,
            shortcuts: &self.shortcuts,
            layer: &[],
        }
    }

    /// A fresh typing state with `latches` clicked.
    fn typing(&self, spec: &BoardSpec, latches: &[&str]) -> Typing {
        let settings = KeyboardSettings::from_defaults().unwrap();
        let mut typing = Typing::new(spec, &settings);
        for id in latches {
            typing.tap(&self.io(), id, false).unwrap();
        }
        typing
    }
}

#[test]
fn every_character_key_searches_exactly_what_it_would_type_now() {
    let spec = BoardSpec::shipped().unwrap();
    for layout in GOLDEN {
        for (latches, caps) in STATES {
            let desk = Desk::new(layout, caps);
            for sc in spec.char_scans() {
                desk.injector.clear();
                let typed = desk
                    .typing(&spec, latches)
                    .tap(&desk.io(), &char_id(sc), false);
                let strokes = desk.injector.strokes();
                let mut typing = desk.typing(&spec, latches);
                let shortcut = LegendMods::shortcut(typing.latches(), &desk.table.keys);
                let got = typing.search(&desk.io(), sc);
                let want = match strokes.as_slice() {
                    _ if shortcut => ToPanel::Pass,
                    [Stroke::Text(t, TextRoute::Unicode)] => {
                        ToPanel::Key(PanelKey::Text(t.clone()))
                    }
                    _ => ToPanel::Nothing,
                };
                let at = format!("{layout} {latches:?} caps {caps} {sc}: {typed:?}");
                assert_eq!(got, want, "{at}");
            }
        }
    }
}

#[test]
fn ctrl_alt_or_win_leave_a_character_key_to_the_app() {
    let spec = BoardSpec::shipped().unwrap();
    let desk = Desk::new("en-us", false);
    for latch in ["ctrl", "alt", "win"] {
        let mut typing = desk.typing(&spec, &[latch]);
        assert_eq!(
            typing.search(&desk.io(), Scan::new(0x10)),
            ToPanel::Pass,
            "{latch}"
        );
        assert!(
            !typing.latches().held(&[]).is_empty(),
            "{latch} stays latched"
        );
    }
}

#[test]
fn a_search_letter_lets_one_shot_latches_go_as_after_typing() {
    let c = Clip::quiet(None);
    c.open(true);
    c.b.rig.tap("shift").unwrap();
    c.b.rig.press("c-0x0010").unwrap();
    assert_eq!(c.heard(), [text("Q")]);
    assert!(!c.on("shift"), "Shift let go");
    c.b.rig.switch("fr-fr");
    c.b.rig.tap("altgr").unwrap();
    c.b.rig.press("c-0x000B").unwrap();
    c.b.rig.fakes.source.set_caps(true);
    c.b.rig.press("c-0x0010").unwrap();
    assert_eq!(c.heard(), [text("@"), text("A")]);
    assert!(c.strokes().is_empty());
}

#[test]
fn a_dead_key_in_the_search_sends_nothing_and_ends_its_wait() {
    let c = Clip::quiet(None);
    c.b.rig.switch("fr-fr");
    c.b.rig.press("c-0x001A").unwrap();
    assert_eq!(c.strokes(), [Stroke::Tap(Scan::new(0x1A))], "a wait starts");
    c.open(true);
    c.b.rig.press("c-0x001A").unwrap();
    assert!(c.heard().is_empty() && c.strokes().is_empty());
    c.open(false);
    c.b.rig.press("c-0x0012").unwrap();
    assert_eq!(
        c.strokes(),
        [unicode("e")],
        "no wait left: text, not a key press"
    );
}

#[test]
fn an_arabic_letter_and_its_harakat_go_to_the_search_as_typed() {
    let c = Clip::quiet(None);
    c.b.rig.switch("ar-tn");
    c.open(true);
    c.b.rig.press("c-0x0010").unwrap();
    c.b.rig.tap("altgr").unwrap();
    c.b.rig.press("c-0x0010").unwrap();
    assert_eq!(
        c.heard(),
        [text("ض"), text("\u{651}\u{64E}")],
        "shadda, then fatha"
    );
}

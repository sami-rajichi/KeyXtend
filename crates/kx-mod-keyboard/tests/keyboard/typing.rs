//! Typing through the fakes on the captured EN, FR and AR tables: each click's route, dead keys, Caps Lock,
//! `AltGr`, key apps, the language key and a send refused whole.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use kx_mod_keyboard::spec::char_id;
use kx_mod_keyboard::{
    BoardSpec, Io, KeyboardSettings, Latch, Modifiers, TapError, Tapped, Typing,
};
use kx_module_api::Redacted;
use kx_module_api::ports::{InjectError, LayoutTable, Scan, TextRoute, Toggle};
use kx_platform_fake::{FakeInjector, FakeLayoutSource, FakeShortcuts, LayoutFile, Stroke};

/// One layout's fakes and a typing state on the shipped board.
pub struct Rig {
    pub injector: FakeInjector,
    pub source: FakeLayoutSource,
    pub table: LayoutTable,
    pub shortcuts: FakeShortcuts,
    pub typing: Typing,
}

impl Rig {
    /// The golden `layout`, such as `fr-fr`, with the default settings changed by `change`.
    pub fn new(layout: &str, change: impl FnOnce(&mut KeyboardSettings)) -> Self {
        let mut settings = KeyboardSettings::from_defaults().unwrap();
        change(&mut settings);
        Self {
            injector: FakeInjector::new(),
            source: FakeLayoutSource::golden(&[layout]).unwrap(),
            table: LayoutTable::from(&LayoutFile::golden(layout).unwrap()),
            shortcuts: FakeShortcuts::new(),
            typing: Typing::new(&BoardSpec::shipped().unwrap(), &settings),
        }
    }

    /// A click on key `id`.
    pub fn tap(&mut self, id: &str) -> Result<Tapped, TapError> {
        let io = Io {
            injector: &self.injector,
            source: &self.source,
            table: &self.table,
            shortcuts: &self.shortcuts,
            layer: &[],
        };
        self.typing.tap(&io, id, false)
    }

    /// A click on the character key that sends `code`.
    pub fn char(&mut self, code: u16) -> Result<Tapped, TapError> {
        self.tap(&char_id(Scan::new(code)))
    }

    /// The strokes since the last look.
    pub fn sent(&self) -> Vec<Stroke> {
        let strokes = self.injector.strokes();
        self.injector.clear();
        strokes
    }
}

/// Text typed by `by`.
pub fn text(t: &str, by: TextRoute) -> Stroke {
    Stroke::Text(Redacted::new(t.into()), by)
}

/// Text typed as Unicode.
pub fn unicode(t: &str) -> Stroke {
    text(t, TextRoute::Unicode)
}

/// A key alone.
pub fn tap(code: u16) -> Stroke {
    Stroke::Tap(Scan::new(code))
}

/// Keys `held` around a key.
pub fn chord(held: &[u16], code: u16) -> Stroke {
    let held = held.iter().map(|&c| Scan::new(c)).collect();
    Stroke::Chord(held, Scan::new(code))
}

#[test]
fn letters_go_as_unicode_and_named_keys_as_scan_codes() {
    let mut r = Rig::new("en-us", |_| ());
    for id in [char_id(Scan::new(0x10)), "esc".into(), "caps".into()] {
        assert_eq!(r.tap(&id), Ok(Tapped::Done));
    }
    assert_eq!(r.sent(), [unicode("q"), tap(0x01), tap(0x3A)]);
    r.tap("shift").unwrap();
    assert!(r.sent().is_empty(), "a latch sends nothing");
    r.char(0x10).unwrap();
    r.char(0x10).unwrap();
    assert_eq!(r.sent(), [unicode("Q"), unicode("q")], "Shift lets go");
}

#[test]
fn caps_lock_types_what_the_layout_gives_with_it() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.source.set_caps(true);
    r.char(0x03).unwrap();
    r.char(0x10).unwrap();
    assert_eq!(r.sent(), [unicode("2"), unicode("A")], "AZERTY: é gives 2");
}

#[test]
fn a_french_dead_key_and_the_letter_after_it_go_as_key_presses() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    assert!(r.typing.waiting());
    r.char(0x12).unwrap();
    assert!(!r.typing.waiting());
    r.char(0x12).unwrap();
    assert_eq!(r.sent(), [tap(0x1A), tap(0x12), unicode("e")], "ê, then e");
}

#[test]
fn an_arrow_keeps_the_accent_waiting_and_space_ends_it() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    r.tap("left").unwrap();
    assert!(r.typing.waiting(), "Left types nothing");
    r.tap("space").unwrap();
    assert!(!r.typing.waiting(), "Space types the accent");
    assert_eq!(r.sent(), [tap(0x1A), tap(0xE04B), tap(0x39)]);
}

#[test]
fn altgr_types_its_character_and_holds_for_its_dead_keys() {
    let mut r = Rig::new("fr-fr", |_| ());
    r.tap("altgr").unwrap();
    r.char(0x12).unwrap();
    r.tap("altgr").unwrap();
    r.char(0x03).unwrap();
    assert!(r.typing.waiting(), "AltGr+é is the dead ~");
    r.char(0x31).unwrap();
    let want = [unicode("€"), chord(&[0xE038], 0x03), tap(0x31)];
    assert_eq!(r.sent(), want, "the app makes ñ");
}

#[test]
fn altgr_on_english_is_right_alt_and_ctrl_makes_a_shortcut() {
    let mut r = Rig::new("en-us", |_| ());
    r.tap("altgr").unwrap();
    r.char(0x10).unwrap();
    r.tap("ctrl").unwrap();
    r.tap("shift").unwrap();
    r.char(0x2E).unwrap();
    let want = [chord(&[0xE038], 0x10), chord(&[0x2A, 0x1D], 0x2E)];
    assert_eq!(r.sent(), want, "Right Alt+Q, then Ctrl+Shift+C");
}

#[test]
fn arabic_types_lam_alef_and_harakat_whole_and_in_order() {
    let mut r = Rig::new("ar-tn", |_| ());
    r.char(0x30).unwrap();
    r.tap("shift").unwrap();
    r.char(0x10).unwrap();
    r.tap("altgr").unwrap();
    r.char(0x10).unwrap();
    let want = [
        unicode("\u{644}\u{627}"),
        unicode("\u{64E}"),
        unicode("\u{651}\u{64E}"),
    ];
    assert_eq!(r.sent(), want, "lam-alef, fatha, then shadda before fatha");
}

#[test]
fn an_empty_key_sends_nothing_and_keeps_the_latches_and_the_wait() {
    let mut r = Rig::new("ar-tn", |_| ());
    r.tap("shift").unwrap();
    assert_eq!(
        r.char(0x1F),
        Ok(Tapped::Empty),
        "س types nothing with Shift"
    );
    assert!(r.typing.latches().is_on(Latch::Shift));
    r.tap("shift").unwrap();
    r.tap("altgr").unwrap();
    assert_eq!(r.char(0x14), Ok(Tapped::Empty), "ف has no AltGr character");
    assert!(r.typing.latches().is_on(Latch::AltGr));
    assert!(r.sent().is_empty());
    let mut r = Rig::new("fr-fr", |_| ());
    r.char(0x1A).unwrap();
    r.tap("altgr").unwrap();
    assert_eq!(r.char(0x10), Ok(Tapped::Empty), "a has no AltGr character");
    assert!(r.typing.waiting(), "the ^ still waits");
    assert_eq!(r.sent(), [tap(0x1A)]);
}

#[test]
fn a_key_app_gets_text_as_key_presses_matched_without_case() {
    let apps = |s: &mut KeyboardSettings| {
        s.key_apps = vec!["notepad.exe".into(), "ApplicationFrameHost.exe".into()];
    };
    let mut r = Rig::new("en-us", apps);
    for app in ["NOTEPAD.EXE", "applicationframehost.exe", "chrome.exe"] {
        r.injector.set_target(Some(app));
        r.char(0x10).unwrap();
    }
    r.injector.set_target(None);
    r.char(0x10).unwrap();
    let keys = text("q", TextRoute::Keys);
    let want = [keys.clone(), keys, unicode("q"), unicode("q")];
    assert_eq!(
        r.sent(),
        want,
        "UWP apps all report ApplicationFrameHost.exe"
    );
}

#[test]
fn the_language_key_later_keys_and_mouse_assist_keys_send_nothing() {
    let mut r = Rig::new("en-us", |_| ());
    let io = Io {
        injector: &r.injector,
        source: &r.source,
        table: &r.table,
        shortcuts: &r.shortcuts,
        layer: &[],
    };
    assert_eq!(
        r.typing.tap(&io, "lang", true),
        Ok(Tapped::Lang { back: true })
    );
    assert_eq!(r.tap("mic"), Ok(Tapped::Later));
    assert_eq!(r.tap("rclick"), Ok(Tapped::Assist(Toggle::RightClick)));
    assert_eq!(r.tap("shortcuts"), Ok(Tapped::Layer));
    assert_eq!(r.tap("power"), Ok(Tapped::Power));
    assert_eq!(r.tap("no-such-key"), Err(TapError::UnknownKey));
    assert_eq!(
        r.tap("sc-copy"),
        Err(TapError::UnknownKey),
        "no layer drawn"
    );
    assert!(r.sent().is_empty());
}

#[test]
fn a_send_refused_whole_keeps_the_latches_and_the_wait() {
    let mut r = Rig::new("fr-fr", |s| s.modifiers = Modifiers::Release);
    r.tap("shift").unwrap();
    let refused = InjectError::Refused { sent: 0, total: 2 };
    r.injector.set_error(Some(refused));
    assert_eq!(r.char(0x1A), Err(TapError::Inject(refused)));
    assert!(r.typing.latches().is_on(Latch::Shift) && !r.typing.waiting());
    r.injector.set_error(None);
    r.char(0x10).unwrap();
    assert_eq!(r.sent(), [unicode("A")]);
    assert!(!r.typing.latches().is_on(Latch::Shift));
}

#[test]
fn stay_keeps_shift_for_every_letter_until_it_is_clicked_again() {
    let mut r = Rig::new("en-us", |s| s.modifiers = Modifiers::Stay);
    r.tap("shift").unwrap();
    r.char(0x10).unwrap();
    r.char(0x11).unwrap();
    r.tap("rshift").unwrap();
    r.char(0x10).unwrap();
    assert_eq!(r.sent(), [unicode("Q"), unicode("W"), unicode("q")]);
}

#[test]
fn a_debug_print_shows_no_latch_dead_key_or_key_map() {
    let mut r = Rig::new("fr-fr", |_| {});
    r.char(0x1A).unwrap();
    r.tap("shift").unwrap();
    assert!(r.typing.waiting() && r.typing.latches().is_on(Latch::Shift));
    assert_eq!(format!("{:?}", r.typing), "Typing { .. }");
}

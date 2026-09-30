use super::*;
use crate::legend::testkeys::{dead, letter};
use crate::settings::Modifiers;
use std::cell::Cell;

const ESC: Scan = Scan::new(0x01);
const Q: Scan = Scan::new(0x10);
const HAT: Scan = Scan::new(0x11);
const E: Scan = Scan::new(0x12);
const NONE_HERE: Scan = Scan::new(0x2C);
const SHIFT: Scan = Scan::new(0x2A);
const RSHIFT: Scan = Scan::new(0x36);
const CTRL: Scan = Scan::new(0x1D);
const ALTGR: Scan = Scan::new(0xE038);
const CAPS: Scan = Scan::new(0x3A);
const LEFT: Scan = Scan::new(0xE04B);

/// A made-up layout: q, a dead ^ with no Shift or `AltGr` character, and e with € on `AltGr` when `altgr`.
fn keys(altgr: bool) -> BTreeMap<Scan, KeyChars> {
    let euro = if altgr { "€" } else { "" };
    let e = KeyChars {
        altgr: euro.into(),
        ..letter("e", "E")
    };
    BTreeMap::from([(Q, letter("q", "Q")), (HAT, dead("^")), (E, e)])
}

/// What a plan is made from, owned so a test can change one part, and how often the app in front was asked.
struct Given {
    keys: BTreeMap<Scan, KeyChars>,
    latches: Latches,
    caps: bool,
    dead: bool,
    apps: Vec<String>,
    app: Option<String>,
    asked: Cell<usize>,
}

impl Given {
    fn new(altgr: bool) -> Self {
        Self {
            keys: keys(altgr),
            latches: Latches::new(Modifiers::Release),
            caps: false,
            dead: false,
            apps: Vec::new(),
            app: None,
            asked: Cell::new(0),
        }
    }

    fn latch(mut self, l: Latch, key: Scan) -> Self {
        self.latches.toggle(l, key);
        self
    }

    fn plan(&self, role: Role, sc: Option<Scan>) -> Plan {
        let app = || {
            self.asked.set(self.asked.get() + 1);
            self.app.clone()
        };
        let now = Now {
            keys: &self.keys,
            latches: &self.latches,
            caps: self.caps,
            dead: self.dead,
            apps: &self.apps,
            app: &app,
        };
        plan(role, sc, &now)
    }

    fn char(&self, sc: Scan) -> Plan {
        self.plan(Role::Char, Some(sc))
    }
}

fn send(route: Route, wait: Wait) -> Plan {
    Plan::Send { route, wait }
}

fn text(t: &str, by: TextRoute) -> Plan {
    send(Route::Text(Redacted::new(t.into()), by), Wait::End)
}

fn unicode(t: &str) -> Plan {
    text(t, TextRoute::Unicode)
}

#[test]
fn named_keys_send_latch_tap_caps_or_ask_the_face() {
    let g = Given::new(false);
    assert_eq!(
        g.plan(Role::Send, Some(ESC)),
        send(Route::Tap(ESC), Wait::End)
    );
    let shift = g.plan(Role::Latch(Latch::Shift), Some(RSHIFT));
    assert_eq!(shift, Plan::Latch(Latch::Shift, RSHIFT));
    assert_eq!(g.plan(Role::Caps, Some(CAPS)), Plan::Caps(CAPS));
    assert_eq!(g.plan(Role::Lang, None), Plan::Lang);
    assert_eq!(g.plan(Role::Later, None), Plan::Later);
    assert_eq!(g.plan(Role::Send, None), Plan::Later, "no code to send");
}

#[test]
fn a_named_key_holds_every_latch() {
    let g = Given::new(false).latch(Latch::Ctrl, CTRL);
    let g = g.latch(Latch::Shift, SHIFT);
    let plan = g.plan(Role::Send, Some(LEFT));
    assert_eq!(plan, send(Route::Chord(vec![SHIFT, CTRL], LEFT), Wait::End));
}

#[test]
fn a_character_key_types_its_text_or_a_shortcut() {
    let mut g = Given::new(true);
    assert_eq!(g.char(Q), unicode("q"));
    g.caps = true;
    assert_eq!(g.char(Q), unicode("Q"), "Caps Lock");
    let g = Given::new(true).latch(Latch::Shift, SHIFT);
    assert_eq!(g.char(Q), unicode("Q"), "Shift is in the text, not held");
    let g = Given::new(true).latch(Latch::AltGr, ALTGR);
    assert_eq!(g.char(E), unicode("€"));
    let g = Given::new(true).latch(Latch::Ctrl, CTRL);
    let shortcut = send(Route::Chord(vec![CTRL], Q), Wait::End);
    assert_eq!(g.char(Q), shortcut, "Ctrl+Q is a shortcut");
}

#[test]
fn a_dead_key_and_the_key_after_it_go_as_key_presses_so_the_app_adds_the_accent() {
    let mut g = Given::new(false);
    assert_eq!(g.char(HAT), send(Route::Tap(HAT), Wait::Start));
    g.dead = true;
    assert_eq!(
        g.char(E),
        send(Route::Tap(E), Wait::End),
        "the layout makes ê"
    );
    g.dead = false;
    assert_eq!(g.char(E), unicode("e"));
}

#[test]
fn a_latch_clicked_between_a_dead_key_and_its_letter_is_held_for_the_letter() {
    let mut g = Given::new(false).latch(Latch::Shift, SHIFT);
    g.dead = true;
    assert_eq!(
        g.char(E),
        send(Route::Chord(vec![SHIFT], E), Wait::End),
        "Ê"
    );
}

#[test]
fn a_shift_the_dead_key_ignores_is_not_held() {
    let g = Given::new(false).latch(Latch::Shift, SHIFT);
    assert_eq!(g.char(HAT), send(Route::Tap(HAT), Wait::Start));
}

#[test]
fn a_latch_the_legend_ignores_is_not_held_around_a_dead_key_or_its_letter() {
    let g = Given::new(true).latch(Latch::AltGr, ALTGR);
    assert_eq!(
        g.char(HAT),
        send(Route::Tap(HAT), Wait::Start),
        "^ has no AltGr character"
    );
    let mut g = g.latch(Latch::Shift, SHIFT);
    g.dead = true;
    let q = send(Route::Chord(vec![SHIFT], Q), Wait::End);
    assert_eq!(g.char(Q), q, "Shift is kept for Q, AltGr is not");
}

#[test]
fn altgr_on_a_layout_without_altgr_characters_is_right_alt() {
    let g = Given::new(false).latch(Latch::AltGr, ALTGR);
    assert_eq!(g.char(Q), send(Route::Chord(vec![ALTGR], Q), Wait::End));
}

#[test]
fn a_key_sent_while_a_dead_key_waits_asks_the_layout_whether_it_typed() {
    let mut g = Given::new(false);
    g.dead = true;
    let left = g.plan(Role::Send, Some(LEFT));
    assert_eq!(left, send(Route::Tap(LEFT), Wait::Ask));
    let mut g = Given::new(false).latch(Latch::Ctrl, CTRL);
    g.dead = true;
    assert_eq!(g.char(Q), send(Route::Chord(vec![CTRL], Q), Wait::Ask));
}

#[test]
fn a_key_that_types_nothing_in_the_layout_goes_as_its_scan_code() {
    let mut g = Given::new(false).latch(Latch::Shift, SHIFT);
    g.keys.insert(NONE_HERE, KeyChars::default());
    let plan = g.char(NONE_HERE);
    assert_eq!(
        plan,
        send(Route::Tap(NONE_HERE), Wait::End),
        "as its legend"
    );
    let mut g = Given::new(false);
    g.keys.insert(NONE_HERE, KeyChars::default());
    g.dead = true;
    let plan = g.char(NONE_HERE);
    assert_eq!(
        plan,
        send(Route::Tap(NONE_HERE), Wait::Ask),
        "the accent may still wait"
    );
}

#[test]
fn a_key_missing_from_the_table_holds_every_latch() {
    let g = Given::new(false).latch(Latch::Shift, SHIFT);
    let shifted = send(Route::Chord(vec![SHIFT], NONE_HERE), Wait::End);
    assert_eq!(
        g.char(NONE_HERE),
        shifted,
        "the app's own layout reads Shift"
    );
    let g = Given::new(true).latch(Latch::AltGr, ALTGR);
    let altgr = send(Route::Chord(vec![ALTGR], NONE_HERE), Wait::End);
    assert_eq!(g.char(NONE_HERE), altgr);
}

#[test]
fn key_apps_get_text_as_key_presses() {
    let mut g = Given::new(true);
    (g.apps, g.app) = (vec!["notepad.exe".into()], Some("Notepad.exe".into()));
    assert_eq!(g.char(Q), text("q", TextRoute::Keys));
    let plan = g.plan(Role::Send, Some(ESC));
    assert_eq!(plan, send(Route::Tap(ESC), Wait::End), "keys stay keys");
}

#[test]
fn the_app_in_front_is_asked_only_for_text_and_only_when_key_apps_names_one() {
    let mut g = Given::new(true);
    g.char(Q);
    assert_eq!(g.asked.get(), 0, "no key apps");
    (g.apps, g.app) = (vec!["notepad.exe".into()], Some("chrome.exe".into()));
    g.plan(Role::Send, Some(ESC));
    g.plan(Role::Latch(Latch::Shift), Some(SHIFT));
    g.char(HAT);
    assert_eq!(g.asked.get(), 0, "no text");
    assert_eq!(g.char(Q), unicode("q"));
    assert_eq!(g.asked.get(), 1);
}

#[test]
fn a_key_app_is_matched_by_file_name_without_case() {
    let apps = vec![
        "notepad.exe".to_owned(),
        " ApplicationFrameHost.exe ".to_owned(),
    ];
    let route = |app: Option<&str>| text_route(&apps, || app.map(str::to_owned));
    assert_eq!(route(Some("Notepad.EXE")), TextRoute::Keys);
    assert_eq!(
        route(Some("applicationframehost.exe")),
        TextRoute::Keys,
        "every UWP app"
    );
    assert_eq!(route(Some("notepad")), TextRoute::Unicode);
    assert_eq!(route(Some("chrome.exe")), TextRoute::Unicode);
    assert_eq!(route(None), TextRoute::Unicode, "no app in front");
    let none = text_route(&[], || Some("notepad.exe".into()));
    assert_eq!(none, TextRoute::Unicode);
}

#[test]
fn a_route_never_shows_its_keys_or_text() {
    let secret = Redacted::new("hunter2".to_owned());
    let routes = [
        Route::Tap(Q),
        Route::Chord(vec![CTRL], Q),
        Route::Text(secret, TextRoute::Keys),
    ];
    let shown = format!("{routes:?}");
    for leak in ["hunter2", "16", "10", "Scan"] {
        assert!(!shown.contains(leak), "{shown}");
    }
    assert!(shown.contains(REDACTED_MARKER) && shown.contains("Keys"));
}

#[test]
fn a_plan_never_shows_a_scan_code() {
    let plans = [
        Plan::Caps(CAPS),
        Plan::Latch(Latch::Shift, SHIFT),
        send(Route::Tap(Q), Wait::Start),
    ];
    let shown = format!("{plans:?}");
    for leak in ["58", "3A", "42", "2A", "16", "Scan"] {
        assert!(!shown.contains(leak), "{shown}");
    }
    for part in ["Caps", "Latch(Shift)", "Start"] {
        assert!(shown.contains(part), "{shown}");
    }
}

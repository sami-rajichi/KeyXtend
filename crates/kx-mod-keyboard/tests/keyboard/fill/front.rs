//! A fill never reaches another app (design "Filling"): the app in front as its first part goes must still be in front
//! for each part after a Tab, else the rest is dropped with a no; an app the system cannot name counts as another.

use super::{APP, GAP, Quick, gap_ms, line, login, tab, typed, unicode};
use crate::typing::Rig;
use kx_mod_keyboard::{Filled, Io};
use kx_module_api::ports::{FillPart, FillType, InjectError, InputInjector, Scan, TextRoute};
use kx_module_api::{Mono, Redacted};
use kx_platform_fake::FakeInjector;

/// The app a quick switch brings to the front.
const OTHER: &str = "chat.exe";

/// An injector whose app in front changes as its first text goes out, as a switch before the Tab does.
struct Switching(FakeInjector);

impl InputInjector for Switching {
    fn tap(&self, key: Scan) -> Result<(), InjectError> {
        self.0.tap(key)
    }

    fn chord(&self, held: &[Scan], key: Scan) -> Result<(), InjectError> {
        self.0.chord(held, key)
    }

    fn text(&self, text: &Redacted<String>, route: TextRoute) -> Result<(), InjectError> {
        self.0.set_target(Some(OTHER));
        self.0.text(text, route)
    }

    fn target_app(&self) -> Option<String> {
        self.0.target_app()
    }

    fn front_window(&self) -> Option<u64> {
        self.0.front_window()
    }
}

#[test]
fn a_switch_before_the_tab_stops_the_part_after_it() {
    let mut rig = Rig::new("en-us", |_| ());
    let injector = Switching(FakeInjector::new());
    injector.0.set_target(Some(APP));
    let ask = FillType {
        round: 1,
        parts: login(),
        tab_gap: GAP,
    };
    let mut fill = rig.typing.fill(&ask).unwrap();
    let io = Io {
        injector: &injector,
        source: &rig.source,
        table: &rig.table,
        shortcuts: &rig.shortcuts,
        layer: &[],
    };
    let Filled::Wait(due) = rig.typing.fill_on(&io, &mut fill, Mono::default()) else {
        panic!("a fill with a part after its Tab waits");
    };
    let done = rig.typing.fill_on(&io, &mut fill, due);
    assert_eq!(done, Filled::Done(false));
    assert_eq!(
        injector.0.strokes(),
        [unicode("alex"), tab()],
        "the password never reaches the other app"
    );
}

#[test]
fn another_app_in_front_after_the_tab_drops_the_rest() {
    let q = Quick::quiet(None);
    q.fill(1, login());
    assert_eq!(q.strokes(), [unicode("alex"), tab()]);
    q.front(Some(OTHER));
    q.relabel_in(gap_ms());
    assert!(
        q.strokes().is_empty(),
        "the password never reaches the other app"
    );
    assert_eq!(q.heard(), [typed(1, false)]);
}

#[test]
fn the_same_app_in_front_gets_every_part_after_each_tab() {
    let q = Quick::quiet(None);
    let parts = vec![
        line("a"),
        FillPart::Tab,
        line("b"),
        FillPart::Tab,
        line("c"),
    ];
    q.fill(2, parts);
    q.relabel_in(gap_ms());
    q.relabel_in(gap_ms());
    let all = [unicode("a"), tab(), unicode("b"), tab(), unicode("c")];
    assert_eq!(q.strokes(), all);
    assert_eq!(q.heard(), [typed(2, true)]);
}

#[test]
fn an_app_switch_after_the_second_tab_drops_only_what_follows_it() {
    let q = Quick::quiet(None);
    let parts = vec![
        line("a"),
        FillPart::Tab,
        line("b"),
        FillPart::Tab,
        line("c"),
    ];
    q.fill(3, parts);
    q.relabel_in(gap_ms());
    assert_eq!(q.strokes(), [unicode("a"), tab(), unicode("b"), tab()]);
    q.front(Some(OTHER));
    q.relabel_in(gap_ms());
    assert!(q.strokes().is_empty());
    assert_eq!(q.heard(), [typed(3, false)]);
}

#[test]
fn an_app_the_system_cannot_name_counts_as_another() {
    let q = Quick::quiet(None);
    q.fill(4, login());
    q.strokes();
    q.front(None);
    q.relabel_in(gap_ms());
    assert!(q.strokes().is_empty(), "unnamed after the Tab");
    assert_eq!(q.heard(), [typed(4, false)]);
    q.fill(5, login());
    q.strokes();
    q.front(Some(APP));
    q.relabel_in(gap_ms());
    assert!(q.strokes().is_empty(), "unnamed as the fill started");
    assert_eq!(q.heard(), [typed(5, false)]);
}

#[test]
fn a_fill_with_no_wait_never_needs_the_app_named() {
    let q = Quick::quiet(None);
    q.front(None);
    q.fill(6, vec![line("solo"), FillPart::Tab]);
    assert_eq!(q.strokes(), [unicode("solo"), tab()]);
    assert_eq!(q.heard(), [typed(6, true)]);
}

/// Two unknowns never match, so a fill into an app the system cannot name stops at its first wait.
#[test]
fn an_app_unnamed_at_the_start_and_at_the_check_still_drops_the_rest() {
    let q = Quick::quiet(None);
    q.front(None);
    q.fill(7, login());
    assert_eq!(q.strokes(), [unicode("alex"), tab()]);
    q.relabel_in(gap_ms());
    assert!(q.strokes().is_empty(), "no app to match");
    assert_eq!(q.heard(), [typed(7, false)]);
}

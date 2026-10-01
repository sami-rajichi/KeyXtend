//! The keyboard side: the face's JSON and actions, redraws, the guard, the window's shape, the pointer and the bubble.

use crate::{Rig, main_screen};
use kx_module_api::keys;
use kx_module_api::ports::{Corner, FaceAction, FaceError, FaceReply, KeyId, Point, Rect};
use kx_platform::ui::UiError;
use kx_platform_fake::GuardCall;
use kx_ui::{Area, Redraw, Resize, Wake};

#[test]
fn without_a_face_there_is_nothing_to_draw_or_do() {
    let rig = Rig::new();
    assert_eq!(
        (rig.hub.board_json(), rig.hub.state_json()),
        ("null".into(), "null".into())
    );
    assert_eq!(rig.hub.act(FaceAction::Relabel), Redraw::default());
}

#[test]
fn the_face_draws_and_says_what_to_redraw() {
    let (rig, face, _host) = Rig::with_face();
    assert_eq!(rig.hub.board_json(), r#"{"board":1}"#);
    assert_eq!(rig.hub.state_json(), r#"{"state":1}"#);
    face.answer(Ok(FaceReply {
        board: true,
        state: false,
        notice: None,
    }));
    let redraw = rig.hub.act(FaceAction::Bigger);
    assert_eq!(
        redraw,
        Redraw {
            board: true,
            state: false
        }
    );
    assert_eq!(redraw.to_json(), r#"{"board":true,"state":false}"#);
    assert_eq!(face.acts(), [FaceAction::Bigger]);
}

#[test]
fn a_notice_in_the_reply_goes_to_the_banner() {
    let (rig, face, _host) = Rig::with_face();
    let reply = FaceReply {
        board: false,
        state: true,
        notice: Some(keys::KEYS_NOT_SENT),
    };
    face.answer(Ok(reply));
    let id = KeyId::new("a");
    assert!(rig.hub.act(FaceAction::Tap { id, back: false }).state);
    assert_eq!(rig.wakes(), [Wake::Notices]);
    let shown = rig.hub.take_notices();
    assert_eq!(shown.len(), 1);
    assert!(!shown[0].retry);
}

#[test]
fn a_refused_action_redraws_nothing() {
    let (rig, face, _host) = Rig::with_face();
    face.answer(Err(FaceError::NoChoice(9)));
    assert_eq!(
        rig.hub.act(FaceAction::LangPick { index: 9 }),
        Redraw::default()
    );
    assert!(rig.hub.take_notices().is_empty());
}

#[test]
fn a_panel_key_carries_the_page_rows_setting() {
    let (rig, face, _host) = Rig::with_face();
    rig.hub.panel_key("down");
    let rows = rig.hub.settings().page_rows;
    assert_eq!(
        face.acts(),
        [FaceAction::PanelKey {
            key: KeyId::new("down"),
            rows
        }]
    );
}

#[test]
fn a_placed_window_carries_the_screens_the_port_reports() {
    let (rig, face, _host) = Rig::with_face();
    let rect = Rect {
        x: 10,
        y: 20,
        w: 900,
        h: 300,
    };
    rig.hub.placed(rect);
    let screens = vec![main_screen()];
    assert_eq!(face.acts(), [FaceAction::Placed { rect, screens }]);
}

#[test]
fn a_resize_follows_the_pointer_in_physical_pixels() {
    let (rig, face, _host) = Rig::with_face();
    rig.hub.resize(Resize::Begin);
    rig.screens.set_cursor(Point { x: 50, y: 60 });
    rig.hub.resize(Resize::At);
    rig.hub.resize(Resize::End);
    let want = [
        FaceAction::ResizeBegin { x: 5, y: 6 },
        FaceAction::ResizeAt { x: 50, y: 60 },
        FaceAction::ResizeEnd,
    ];
    assert_eq!(face.acts(), want);
}

#[test]
fn an_unknown_pointer_sends_no_resize_step() {
    let (rig, face, _host) = Rig::with_face();
    rig.screens.set_cursor_error(Some(UiError::Refused(5)));
    assert_eq!(rig.hub.cursor(), None);
    assert_eq!(rig.hub.resize(Resize::At), Redraw::default());
    assert!(face.acts().is_empty());
}

#[test]
fn the_bubble_takes_the_theme_size_at_the_screen_dpi() {
    let rig = Rig::new();
    let at = rig.hub.bubble_at(Point { x: 100, y: 100 }, Corner::Right);
    assert_eq!(
        at,
        Some(Point {
            x: 1920 - 46 - 12,
            y: 1040 - 46 - 12
        })
    );
    let mut big = main_screen();
    big.dpi = 192;
    rig.screens.set_areas(vec![big]);
    let left = rig.hub.bubble_at(Point { x: 100, y: 100 }, Corner::Left);
    assert_eq!(
        left,
        Some(Point {
            x: 24,
            y: 1040 - 92 - 24
        })
    );
    rig.screens.set_areas(Vec::new());
    assert_eq!(rig.hub.bubble_at(Point::default(), Corner::Left), None);
}

#[test]
fn the_window_is_cut_to_its_boxes_rounded_outwards() {
    let rig = Rig::new();
    let boxes = [Area {
        x: 0.5,
        y: 0.0,
        w: 10.0,
        h: 4.2,
    }];
    assert!(rig.hub.shape("kb", &boxes, 2.0));
    let cut = GuardCall::Shape(
        "kb".into(),
        vec![Rect {
            x: 1,
            y: 0,
            w: 20,
            h: 9,
        }],
    );
    assert!(rig.guard.calls().contains(&cut));
    rig.guard.set_error(Some(UiError::NoWindow));
    assert!(!rig.hub.shape("kb", &boxes, 2.0), "not shown yet");
    rig.guard.set_error(Some(UiError::Refused(3)));
    assert!(!rig.hub.shape("kb", &boxes, 2.0));
}

#[test]
fn a_shown_window_is_lifted_alone_by_its_title() {
    let rig = Rig::new();
    assert!(rig.hub.lift_alone("tip"));
    assert!(
        rig.guard
            .calls()
            .contains(&GuardCall::LiftAlone("tip".into()))
    );
    rig.guard.set_error(Some(UiError::NoWindow));
    assert!(!rig.hub.lift_alone("tip"), "not shown");
    rig.guard.set_error(Some(UiError::Refused(4)));
    assert!(!rig.hub.lift_alone("tip"));
}

#[test]
fn the_guard_tries_until_a_window_shows_then_gives_the_focus_back_once() {
    let rig = Rig::new();
    let tries = rig.hub.settings().guard_tries;
    assert!(rig.hub.guard(1), "no window yet");
    rig.guard.set_count(2);
    assert!(!rig.hub.guard(2));
    let calls = rig.guard.calls();
    let GuardCall::Remember(focus) = calls[0] else {
        panic!("the hub notes the focus first: {calls:?}");
    };
    let backs: Vec<_> = calls
        .iter()
        .filter(|c| matches!(c, GuardCall::GiveBack(_)))
        .collect();
    assert_eq!(backs, [&GuardCall::GiveBack(focus)]);
    assert!(tries > 2);
}

#[test]
fn the_guard_stops_after_its_last_try_or_an_error() {
    let rig = Rig::new();
    let tries = rig.hub.settings().guard_tries;
    assert!(!rig.hub.guard(tries), "out of tries");
    rig.guard.set_error(Some(UiError::Refused(1)));
    assert!(!rig.hub.guard(1));
    let backs = rig
        .guard
        .calls()
        .iter()
        .filter(|c| matches!(c, GuardCall::GiveBack(_)))
        .count();
    assert_eq!(backs, 2);
}

#[test]
fn the_face_may_send_a_notice_or_read_the_texts_while_it_acts() {
    let (rig, _face, _host) = Rig::with_face();
    let hub = std::sync::Arc::clone(&rig.hub);
    let face = std::sync::Arc::new(Echo(hub));
    let mut host = kx_test_support::host::FakeHost::new(&kx_ui::MANIFEST);
    rig.hub.attach(
        kx_ui::UiSettings::from_defaults().unwrap(),
        face,
        host.handle(),
    );
    rig.hub.act(FaceAction::Relabel);
    assert_eq!(
        rig.hub.take_notices().len(),
        1,
        "no lock is held while the face acts"
    );
    assert_eq!(
        rig.hub.i18n().lang().tag,
        ECHO_LANG,
        "nor while it names its language"
    );
}

/// The language of the layout `Echo` shows.
const ECHO_LANG: &str = "fr";

/// A face that calls the hub from inside `act` and `lang`, as a failed settings save would send a notice.
struct Echo(std::sync::Arc<kx_ui::Hub>);

impl kx_module_api::ports::KeyboardFace for Echo {
    fn board_json(&self) -> String {
        String::new()
    }

    fn state_json(&self) -> String {
        String::new()
    }

    fn lang(&self) -> String {
        let _ = self.0.i18n();
        ECHO_LANG.to_owned()
    }

    fn act(&self, _: FaceAction) -> Result<FaceReply, FaceError> {
        let key = keys::SETTINGS_UNSAVED;
        self.0.notice(kx_module_api::Notice {
            key,
            module: None,
            args: Vec::new(),
        });
        Ok(FaceReply {
            state: true,
            ..FaceReply::default()
        })
    }
}

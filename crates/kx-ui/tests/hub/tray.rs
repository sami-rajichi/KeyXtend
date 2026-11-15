//! The tray icon: its picture and tooltip, its clicks as wakes, hiding it, and the menu's Dictation row.

use crate::{Rig, TRAY_PX, texts, voice};
use kx_module_api::ports::Point;
use kx_module_api::ports::VoiceOpen;
use kx_platform::ui::{TrayEvent, TrayImage, UiError};
use kx_ui::i18n::keys::{TRAY_VOICE, WINDOW_KEYBOARD};
use kx_ui::{AUTO, Hub, Mode, Wake};
use proptest::collection::vec;
use proptest::option;
use proptest::prelude::*;
use std::sync::{Arc, Mutex};

/// The Dictation key's name, which the keyboard's language speaks.
const KEY_MIC: &str = "key.mic";

fn image(fill: u8) -> TrayImage {
    TrayImage::new(1, 1, vec![fill; 4]).unwrap()
}

#[test]
fn the_icon_shows_the_picture_with_the_keyboard_title_and_changes_in_place() {
    let rig = Rig::new();
    assert_eq!(rig.hub.tray_px(), TRAY_PX);
    assert!(rig.hub.tray_show(&image(1)));
    let tip = rig.hub.tray_i18n().text(WINDOW_KEYBOARD);
    assert_eq!(rig.tray.shown(), [(image(1), tip.clone())]);
    assert!(rig.hub.tray_show(&image(2)));
    assert_eq!(rig.tray.shown(), [(image(2), tip)], "one icon, changed");
}

#[test]
fn clicks_on_the_icon_and_elsewhere_reach_the_ui_as_wakes() {
    let rig = Rig::new();
    rig.hub.tray_show(&image(1));
    rig.wakes();
    let at = Point { x: 1700, y: 1050 };
    rig.tray.click(at);
    rig.tray.away();
    let want = [
        Wake::Tray(TrayEvent::Click(at)),
        Wake::Tray(TrayEvent::Away),
    ];
    assert_eq!(rig.wakes(), want);
}

#[test]
fn a_hidden_icon_is_gone_and_hears_nothing() {
    let rig = Rig::new();
    rig.hub.tray_show(&image(1));
    rig.hub.tray_hide();
    assert!(rig.tray.shown().is_empty());
    rig.wakes();
    rig.tray.click(Point { x: 0, y: 0 });
    assert!(rig.wakes().is_empty());
    rig.hub.tray_hide();
}

#[test]
fn a_hidden_icon_never_comes_back_as_the_app_quits() {
    let rig = Rig::new();
    rig.hub.tray_show(&image(1));
    rig.hub.tray_hide();
    assert!(!rig.hub.tray_show(&image(2)), "a picture drawn late");
    assert!(rig.tray.shown().is_empty());
}

#[test]
fn a_refused_icon_says_so_and_a_later_try_shows_it() {
    let rig = Rig::new();
    rig.tray.set_error(Some(UiError::Refused(5)));
    assert!(!rig.hub.tray_show(&image(1)));
    assert!(rig.tray.shown().is_empty());
    rig.tray.set_error(None);
    assert!(rig.hub.tray_show(&image(1)));
    rig.tray.set_error(Some(UiError::Refused(5)));
    assert!(!rig.hub.tray_show(&image(2)), "a refused change");
    assert_eq!(rig.tray.shown().len(), 1, "the icon stays");
}

#[test]
fn the_icon_never_keeps_the_hub_alive() {
    let rig = Rig::new();
    rig.hub.tray_show(&image(1));
    let Rig { hub, tray, .. } = rig;
    let weak = std::sync::Arc::downgrade(&hub);
    drop(hub);
    assert!(weak.upgrade().is_none());
    assert!(tray.shown().is_empty(), "the hub's icon went with it");
}

/// One thing the app, the taskbar or the user does to the tray icon.
#[derive(Copy, Clone, Debug)]
enum Op {
    /// The UI shows this picture, or changes the icon to it.
    Show(u8),
    /// The app hides the icon for good as it quits.
    Hide,
    /// The taskbar refuses later calls with this code, or takes them again.
    Refuse(Option<i32>),
    /// The user clicks the icon here.
    Click(Point),
}

fn op() -> impl Strategy<Value = Op> {
    let at = (any::<i32>(), any::<i32>()).prop_map(|(x, y)| Point { x, y });
    prop_oneof![
        3 => any::<u8>().prop_map(Op::Show),
        1 => Just(Op::Hide),
        1 => option::of(any::<i32>()).prop_map(Op::Refuse),
        1 => at.prop_map(Op::Click),
    ]
}

/// The slot as it should be: the picture shown, if any, whether it closed, and whether the taskbar refuses.
#[derive(Default)]
struct Want {
    shown: Option<u8>,
    closed: bool,
    refused: bool,
}

/// Does `op` on `rig` and on `want`, and checks what a show answered and the wakes a click gave.
fn apply(rig: &Rig, want: &mut Want, op: Op) -> Result<(), TestCaseError> {
    match op {
        Op::Show(fill) => {
            let shown = rig.hub.tray_show(&image(fill));
            prop_assert_eq!(
                shown,
                !want.closed && !want.refused,
                "a closed slot never shows"
            );
            if !want.closed && (shown || want.shown.is_some()) {
                want.shown = Some(fill);
            }
        }
        Op::Hide => {
            rig.hub.tray_hide();
            (want.shown, want.closed) = (None, true);
        }
        Op::Refuse(code) => {
            rig.tray.set_error(code.map(UiError::Refused));
            want.refused = code.is_some();
        }
        Op::Click(at) => {
            rig.wakes();
            rig.tray.click(at);
            let click = Wake::Tray(TrayEvent::Click(at));
            let heard: Vec<Wake> = want.shown.map(|_| click).into_iter().collect();
            prop_assert_eq!(rig.wakes(), heard, "only a shown icon hears a click");
        }
    }
    Ok(())
}

proptest! {
    #[test]
    fn one_icon_at_most_shows_the_last_picture_and_a_closed_slot_never_shows_again(ops in vec(op(), 0..30)) {
        let (rig, mut want) = (Rig::new(), Want::default());
        let tip = rig.hub.tray_i18n().text(WINDOW_KEYBOARD);
        for op in ops {
            apply(&rig, &mut want, op)?;
            let shown: Vec<_> = want.shown.map(|fill| (image(fill), tip.clone())).into_iter().collect();
            prop_assert_eq!(rig.tray.shown(), shown);
        }
        if want.closed {
            rig.tray.set_error(None);
            let (hub, _requests) = Hub::new(rig.ports()).unwrap();
            prop_assert!(hub.tray_show(&image(0)), "a new hub makes its slot anew");
        }
    }
}

#[test]
fn the_dictation_row_shows_a_hidden_keyboard_first_then_opens_the_panel() {
    for mode in [Mode::Away, Mode::Minimised] {
        let rig = Rig::new();
        let heard = voice::heard(&rig);
        rig.hub.keep_mode(mode);
        rig.wakes();
        let (wakes, seen) = (Arc::clone(&rig.wakes), Arc::new(Mutex::new(Vec::new())));
        let at_open = Arc::clone(&seen);
        let _open = heard
            .bus
            .subscribe(move |_: &VoiceOpen| {
                at_open.lock().unwrap().push(wakes.lock().unwrap().clone());
            })
            .unwrap();
        rig.hub.voice_open();
        let want = vec![vec![Wake::Show]];
        assert_eq!(
            *seen.lock().unwrap(),
            want,
            "{mode:?}: shown before the panel opens"
        );
        assert_eq!(heard.sent.opens(), 1);
    }
}

#[test]
fn with_the_keyboard_shown_the_dictation_row_only_opens_the_panel() {
    let rig = Rig::new();
    let heard = voice::heard(&rig);
    rig.hub.voice_open();
    assert!(rig.wakes().is_empty());
    assert_eq!(heard.sent.opens(), 1);
    assert!(heard.sent.acts().is_empty(), "no click on any view");
}

#[test]
fn the_dictation_row_speaks_the_systems_language_and_the_key_the_keyboards() {
    let (rig, _face, _host) = texts::rig("fr", "ar", AUTO);
    assert_eq!(rig.hub.tray_i18n().text(TRAY_VOICE), "Dictée");
    assert_eq!(rig.hub.i18n().text(KEY_MIC), "إملاء");
}

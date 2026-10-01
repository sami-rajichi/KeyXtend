//! The tray icon: its picture and tooltip, its clicks as wakes, and hiding it.

use crate::{Rig, TRAY_PX};
use kx_module_api::ports::Point;
use kx_platform::ui::{TrayEvent, TrayImage, UiError};
use kx_ui::Wake;
use kx_ui::i18n::keys::WINDOW_KEYBOARD;

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

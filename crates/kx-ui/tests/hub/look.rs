//! The look: the look view JSON, following the system look, the theme button, the light or dark switch, backdrops and
//! the board row the drawn check measures by.

use crate::Rig;
use kx_module_api::WriteError;
use kx_platform::ui::{BackdropKind, SysLook, UiError};
use kx_theme::{ModeChoice, Themes};
use kx_ui::{Backdrop, UiSettings, Wake};
use serde_json::Value;

/// The look JSON read back.
fn look(rig: &Rig) -> Value {
    serde_json::from_str(&rig.hub.look_json()).unwrap()
}

#[test]
fn the_look_view_draws_the_chosen_theme_in_the_mode_shown() {
    let rig = Rig::new();
    let v = look(&rig);
    assert_eq!(v["dark"], false);
    assert_eq!(v["look"]["id"], "native");
    rig.hub.set_sys(SysLook {
        dark: true,
        ..SysLook::default()
    });
    assert_eq!(rig.wakes(), [Wake::Look]);
    assert_eq!(look(&rig)["dark"], true, "auto follows the system");
}

#[test]
fn the_watch_follows_the_system_until_it_drops() {
    let rig = Rig::new();
    let watch = rig.hub.watch().unwrap();
    rig.look.set(SysLook {
        dark: true,
        ..SysLook::default()
    });
    assert_eq!(rig.wakes(), [Wake::Look]);
    drop(watch);
    rig.look.set(SysLook::default());
    assert!(rig.wakes().is_empty());
    assert_eq!(look(&rig)["dark"], true, "the last look heard stays");
    rig.look.set_watch_error(Some(UiError::Refused(2)));
    assert_eq!(rig.hub.watch().err(), Some(UiError::Refused(2)));
}

#[test]
fn the_theme_button_cycles_every_theme_in_order_and_saves_it() {
    let (rig, _face, host) = Rig::with_face();
    let ids: Vec<String> = Themes::shipped()
        .unwrap()
        .list
        .iter()
        .map(|t| t.look.id.clone())
        .collect();
    for want in ids.iter().cycle().skip(1).take(ids.len()) {
        assert!(rig.hub.step_theme());
        assert_eq!(&rig.hub.settings().theme, want);
        assert_eq!(look(&rig)["look"]["id"], want.as_str());
    }
    assert_eq!(host.sink.read_as::<UiSettings>().unwrap().theme, ids[0]);
    assert_eq!(rig.wakes().len(), ids.len());
}

#[test]
fn the_switch_shows_the_other_mode_from_what_shows_now() {
    let (rig, _face, host) = Rig::with_face();
    rig.hub.set_sys(SysLook {
        dark: true,
        ..SysLook::default()
    });
    assert!(rig.hub.flip_mode(), "auto shows dark, so light");
    assert_eq!(rig.hub.settings().mode, ModeChoice::Light);
    assert!(rig.hub.flip_mode());
    assert_eq!(
        host.sink.read_as::<UiSettings>().unwrap().mode,
        ModeChoice::Dark
    );
}

#[test]
fn a_look_change_needs_the_module_and_holds_when_unsaved() {
    let rig = Rig::new();
    assert!(!rig.hub.step_theme(), "no module, no handle");
    let (rig, _face, host) = Rig::with_face();
    host.sink.refuse(Some(WriteError::Unsaved));
    assert!(rig.hub.flip_mode(), "it holds for this run");
    assert_eq!(rig.hub.settings().mode, ModeChoice::Dark);
    host.sink.refuse(Some(WriteError::Refused(kx_ui::ID)));
    assert!(!rig.hub.step_theme());
    assert_eq!(rig.hub.settings().theme, "native");
}

#[test]
fn a_backdrop_is_tried_only_when_chosen_and_a_refusal_leaves_the_plate_solid() {
    let (rig, face, mut host) = Rig::with_face();
    assert!(!rig.hub.backdrop("kb"), "none by default");
    assert!(rig.backdrop.applied().is_empty());
    let mut s = UiSettings::from_defaults().unwrap();
    s.backdrop = Backdrop::Mica;
    s.mode = ModeChoice::Dark;
    rig.hub.attach(s, face, host.handle());
    assert!(rig.hub.backdrop("kb"));
    assert_eq!(
        rig.backdrop.applied(),
        [("kb".into(), BackdropKind::Mica, true)]
    );
    rig.backdrop.set_error(Some(UiError::Refused(1)));
    assert!(!rig.hub.backdrop("kb"));
}

/// The keyboard's board, whose row height the theme's drawn check measures caps by.
const BOARD: &str = "../kx-mod-keyboard/board.toml";

#[test]
fn the_drawn_check_measures_caps_one_board_row_high() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(BOARD);
    let board: toml::Table = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let row = Themes::shipped().unwrap().shape.drawn.row_px;
    assert_eq!(board["row_px"].as_float(), Some(f64::from(row)));
}

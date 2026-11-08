//! The selection chips over the whole app: the module starts with its watch and Quit lets it go, Translate opens the
//! page in the system's display language, and switched off it watches and opens nothing.

use crate::rig::{Rig, WAIT, hear};
use keyxtend::boot::Boot;
use keyxtend::drain;
use kx_mod_selection::TRANSLATE_OK_KEY;
use kx_module_api::ModuleState::Active;
use kx_module_api::ports::{BASE_DPI, ChipAct, ChipsView, Rect, Screen, SelChip, SelKind, SelSpot};
use std::sync::Arc;
use std::sync::mpsc::Receiver;

/// The system's display language in the Translate test, so its target differs from the fake look's English.
const ARABIC: &str = "ar-TN";
/// What the page's link holds for that target, as the site table spells it.
const TO_ARABIC: &str = "tl=ar&";
/// The text the user selected.
const HELLO: &str = "hello";

/// A read-only box with a selection on one line, on the rig's one screen.
fn selected() -> SelSpot {
    let line = Rect {
        x: 100,
        y: 40,
        w: 200,
        h: 20,
    };
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    SelSpot {
        box_id: 0,
        kind: SelKind::ReadOnly,
        selected: true,
        first: Some(line),
        last: Some(line),
        caret: None,
        screen: Screen {
            rect,
            dpi: BASE_DPI,
            primary: true,
        },
    }
}

/// The round of the next strip shown with no tick, passing over the other views.
fn shown(views: &Receiver<ChipsView>) -> u32 {
    loop {
        if let ChipsView::Shown {
            round, acted: None, ..
        } = views.recv_timeout(WAIT).unwrap()
        {
            return round;
        }
    }
}

#[test]
fn the_selection_module_starts_with_its_watch_and_quit_lets_it_go() {
    let rig = Rig::new();
    let mut boot = rig.load();
    boot.start().unwrap();
    assert_eq!(boot.kernel.state(kx_mod_selection::ID), Some(Active));
    assert!(rig.sel.watching(), "it watches the selection");
    let Boot {
        kernel,
        hub,
        requests,
        heard,
        ..
    } = boot;
    let thread = drain::spawn(kernel, Arc::clone(&hub), requests, None).unwrap();
    assert!(hub.quit());
    thread.join().unwrap();
    assert!(!rig.sel.watching(), "Quit let go");
    drop(heard);
}

#[test]
fn translate_opens_the_page_in_the_systems_display_language() {
    let rig = Rig::new();
    rig.look.set_language(ARABIC);
    let section = format!("[{}]\n{TRANSLATE_OK_KEY} = true\n", kx_mod_selection::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    rig.sel.set_text(HELLO);
    let mut boot = rig.load();
    let bus = boot.kernel.bus();
    let (_v, views) = hear::<ChipsView>(&bus);
    boot.start().unwrap();
    rig.sel.focus(selected());
    let round = shown(&views);
    let chip = SelChip::Translate;
    bus.publish(&ChipAct { round, chip }).unwrap();
    assert!(rig.web.wait_opened(1, WAIT), "the page opened");
    let opened = rig.web.opened();
    let target = opened[0].expose().contains(TO_ARABIC);
    assert!(target, "the page asks for the system's display language");
}

#[test]
fn with_the_selection_module_switched_off_nothing_watches_or_opens() {
    let rig = Rig::new();
    let section = format!("[kernel]\ndisabled = [\"{}\"]\n", kx_mod_selection::ID);
    std::fs::write(rig.files().settings, section).unwrap();
    let mut boot = rig.load();
    boot.start().unwrap();
    let state = boot.kernel.state(kx_mod_selection::ID);
    assert!(
        state.is_some_and(|s| s != Active),
        "shipped, but off: {state:?}"
    );
    assert_eq!(boot.kernel.state(kx_mod_keyboard::ID), Some(Active));
    assert_eq!(rig.sel.calls().watches, 0, "no watch");
    assert!(rig.web.opened().is_empty(), "no link");
}

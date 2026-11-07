//! The keyboard asks the mouse module for presses while a panel is open (ADR-0040 decision 5), and says so again when
//! that module starts late.

use super::Quick;
use crate::assist::ear;
use kx_module_api::Module;
use kx_module_api::ports::{
    ClipPanel, FillAsk, FillLive, FillPanel, PanelPressesWanted, PressesAsk,
};
use std::sync::{Arc, Mutex};

/// The wishes told so far.
type Wishes = Arc<Mutex<Vec<bool>>>;

/// The keyboard with the vault module running, and what it wishes from here on.
fn quiet() -> (Quick, Wishes) {
    let wishes: Wishes = Arc::default();
    let w = Arc::clone(&wishes);
    let q = Quick::after(None, move |bus| {
        vec![ear(bus, &w, |e: &PanelPressesWanted| e.on)]
    });
    q.live(true);
    q.heard();
    took(&wishes);
    (q, wishes)
}

/// The wishes since the last look.
fn took(wishes: &Wishes) -> Vec<bool> {
    std::mem::take(&mut *wishes.lock().unwrap())
}

#[test]
fn presses_are_wanted_from_open_to_close_and_told_once() {
    let (q, wishes) = quiet();
    q.open(true);
    assert_eq!(took(&wishes), [true]);
    q.open(true);
    q.away();
    q.aim();
    assert!(took(&wishes).is_empty(), "only the panel's state counts");
    q.open(false);
    assert_eq!(took(&wishes), [false]);
    q.open(false);
    assert!(took(&wishes).is_empty());
}

#[test]
fn the_other_panel_taking_over_keeps_them_wanted() {
    let (q, wishes) = quiet();
    q.open(true);
    q.b.send(&ClipPanel { open: true });
    assert_eq!(took(&wishes), [true]);
    q.open(false);
    assert!(took(&wishes).is_empty(), "the clipboard's panel stays open");
    q.b.send(&ClipPanel { open: false });
    assert_eq!(took(&wishes), [false]);
}

#[test]
fn a_late_mouse_module_hears_the_wish_again() {
    let (q, wishes) = quiet();
    q.open(true);
    took(&wishes);
    q.b.send(&PressesAsk);
    assert_eq!(took(&wishes), [true]);
    q.open(false);
    took(&wishes);
    q.b.send(&PressesAsk);
    assert_eq!(took(&wishes), [false]);
}

#[test]
fn the_vault_stopping_ends_the_wish() {
    let (q, wishes) = quiet();
    q.open(true);
    took(&wishes);
    q.live(false);
    assert_eq!(took(&wishes), [false]);
}

#[test]
fn the_stop_says_off_and_a_panel_after_the_restart_says_on() {
    let (mut q, wishes) = quiet();
    q.open(true);
    took(&wishes);
    q.b.rig.module.stop();
    assert_eq!(took(&wishes), [false]);
    q.b.rig.restart(None);
    assert!(took(&wishes).is_empty(), "the panel counts as shut");
    q.live(true);
    q.open(true);
    assert_eq!(took(&wishes), [true], "off was told, so on is told again");
}

#[test]
fn a_stop_with_no_panel_open_says_nothing() {
    let (mut q, wishes) = quiet();
    q.b.rig.module.stop();
    assert!(took(&wishes).is_empty());
}

/// The keyboard subscribes before it asks, so a panel already open wants presses as soon as the module runs.
#[test]
fn a_panel_already_open_at_start_wants_them() {
    let wishes: Wishes = Arc::default();
    let w = Arc::clone(&wishes);
    let _q = Quick::after(None, move |bus| {
        let reply = bus.clone();
        let answer = move |_: &FillAsk| {
            reply.publish(&FillLive { live: true }).unwrap();
            reply.publish(&FillPanel { open: true }).unwrap();
        };
        vec![
            ear(bus, &w, |e: &PanelPressesWanted| e.on),
            bus.subscribe(answer).unwrap(),
        ]
    });
    assert_eq!(took(&wishes), [true]);
}

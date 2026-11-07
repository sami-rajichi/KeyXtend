//! The password box's Quick-fill chip: the lane says it shows only while the vault runs, as `FillLive` last
//! said, so a click never sends `FillOpen` to nobody; each change wakes the top bar, and the hub asks as it listens.

use super::{heard, words};
use crate::Rig;
use crate::assist::woke;
use kx_module_api::Bus;
use kx_module_api::ports::{FillAsk, FillLive, FillOpen};
use kx_test_support::host::FakeCore;
use kx_ui::Wake;
use kx_ui::asks::FILL;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The vault module, running.
const LIVE: FillLive = FillLive { live: true };

#[test]
fn the_lane_shows_the_chip_only_while_the_vault_runs_and_each_change_wakes_the_top_bar() {
    let rig = Rig::new();
    assert_eq!(words(&rig)["fill"], false, "before the vault speaks");
    let (bus, _sent, _subs) = heard(&rig);
    assert_eq!(words(&rig)["fill"], false, "listening, not yet told");
    bus.publish(&LIVE).unwrap();
    assert_eq!(words(&rig)["fill"], true);
    assert_eq!(woke(&rig, Wake::Words), 1, "the chip comes back at once");
    bus.publish(&LIVE).unwrap();
    assert_eq!(woke(&rig, Wake::Words), 0, "nothing changed");
    bus.publish(&FillLive::OFF).unwrap();
    assert_eq!(words(&rig)["fill"], false);
    assert_eq!(woke(&rig, Wake::Words), 1, "the chip goes at once");
}

#[test]
fn a_click_sends_nothing_while_the_vault_is_not_live_and_one_open_once_it_is() {
    let rig = Rig::new();
    let (bus, sent, _subs) = heard(&rig);
    sent.asks();
    let opens = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&opens);
    let open = move |_: &FillOpen| {
        count.fetch_add(1, Ordering::SeqCst);
    };
    let _open = bus.subscribe(open).unwrap();
    let opened = || opens.load(Ordering::SeqCst);
    rig.hub.words_ask(FILL, 0);
    assert_eq!(opened(), 0, "before the vault speaks");
    bus.publish(&LIVE).unwrap();
    rig.hub.words_ask(FILL, 0);
    assert_eq!(opened(), 1, "it came back");
    bus.publish(&FillLive::OFF).unwrap();
    rig.hub.words_ask(FILL, 0);
    assert_eq!(opened(), 1, "after the vault stops");
    assert!(sent.asks().is_empty(), "no words ask");
}

/// The hub subscribes before it asks, so a vault that answers at once is heard.
#[test]
fn a_vault_already_running_shows_the_chip_as_the_hub_starts_listening() {
    let rig = Rig::new();
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let reply = bus.clone();
    let _vault = bus
        .subscribe(move |_: &FillAsk| reply.publish(&LIVE).unwrap())
        .unwrap();
    let _subs = rig.hub.listen(&bus).unwrap();
    assert_eq!(words(&rig)["fill"], true);
}

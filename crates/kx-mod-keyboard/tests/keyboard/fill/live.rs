//! The fill key while the vault module is off (ADR-0037 decision 2): dimmed with a tip, sending nothing, before the
//! vault speaks and after it stops; a stop also shuts its panel, so our keys type again.

use super::{Heard, Quick};
use kx_mod_keyboard::view::FILL_OFF_TIP;
use kx_module_api::ports::{FillAsk, FillLive, Scan};
use kx_platform_fake::Stroke;
use serde_json::{Value, json};

/// The fill key.
const KEY: &str = "fill";

/// The fill key's dim flag, tip and light.
fn key(q: &Quick) -> Value {
    let k = q.b.key(KEY);
    json!({ "dim": k["dim"], "tip": k["tip"], "on": k["on"] })
}

/// The fill key while the vault module is off.
fn off() -> Value {
    json!({ "dim": true, "tip": FILL_OFF_TIP, "on": false })
}

#[test]
fn before_the_vault_speaks_the_fill_key_is_dim_says_so_and_sends_nothing() {
    let q = Quick::after(None, |_| Vec::new());
    assert_eq!(q.heard(), [Heard::Ask]);
    assert_eq!(key(&q), off());
    q.b.rig.tap(KEY).unwrap();
    assert!(q.heard().is_empty());
    assert!(q.strokes().is_empty());
}

/// The keyboard subscribes before it asks, so a vault module that answers at once is heard.
#[test]
fn a_vault_already_running_makes_the_fill_key_work_at_start() {
    let q = Quick::after(None, |bus| {
        let reply = bus.clone();
        let answer = move |_: &FillAsk| reply.publish(&FillLive { live: true }).unwrap();
        vec![bus.subscribe(answer).unwrap()]
    });
    assert_eq!(q.heard(), [Heard::Ask]);
    assert_eq!(key(&q), json!({ "dim": false, "tip": null, "on": false }));
    q.b.rig.tap(KEY).unwrap();
    assert_eq!(q.heard(), [Heard::Open]);
}

#[test]
fn a_vault_that_stops_dims_the_key_shuts_its_panel_and_its_clicks_do_nothing() {
    let q = Quick::opened(None);
    q.b.send(&FillLive::OFF);
    assert_eq!(key(&q), off());
    q.b.rig.tap(KEY).unwrap();
    assert!(q.heard().is_empty());
    q.b.rig.tap("space").unwrap();
    assert_eq!(q.strokes(), [Stroke::Tap(Scan::new(0x39))], "to the app");
}

#[test]
fn a_restart_dims_the_key_until_the_vault_says_it_runs_again() {
    let mut q = Quick::quiet(None);
    q.b.rig.restart(Some(("lang_key", "cycle".into())));
    assert_eq!(q.heard(), [Heard::Ask]);
    assert_eq!(key(&q), off());
    q.live(true);
    assert_eq!(key(&q)["dim"], false);
}

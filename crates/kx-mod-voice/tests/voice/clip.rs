//! The key page's Paste through the module's own relay: an answer the clipboard module gave another module's take of
//! the same round is never taken, so a vault password cannot become a cloud key; the voice module's own answer is.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};

use kx_mod_voice::{ID, VoiceModule};
use kx_module_api::ports::{
    ClipTake, ClipText, CloudButton, ServiceSheet, VoiceAct, VoiceClick, VoiceSheet,
};
use kx_module_api::{Bus, ModuleId, Redacted, Subscription, Zeroizing};

use crate::rig::{Fakes, Heard, Said, WAIT, click, first_service, start, stop};

/// A key-like text of `n` sevens, built as the test runs.
fn key(n: usize) -> String {
    ["kx", &"7".repeat(n)].join("-")
}

/// `key` as the clipboard's text.
fn text(key: &str) -> ClipText {
    ClipText::Text(Redacted::new(Zeroizing::new(key.to_owned())))
}

/// Each take the bus carries, while the guard lives.
fn takes(bus: &Bus) -> (Receiver<ClipTake>, Subscription) {
    let (tx, rx) = mpsc::channel();
    let ear = bus.subscribe(move |t: &ClipTake| {
        let _ = tx.send(*t);
    });
    (rx, ear.unwrap())
}

/// Clicks Paste on the first service's key page; the take it asked.
fn paste(bus: &Bus, heard: &Heard, takes: &Receiver<ClipTake>) -> ClipTake {
    let round = first_service(bus, heard);
    let lines = |s: &VoiceSheet| matches!(s, VoiceSheet::Service(_));
    let round = click(
        bus,
        heard,
        round,
        VoiceClick::Service(CloudButton::Lines),
        lines,
    );
    let click = VoiceClick::Service(CloudButton::Paste);
    bus.publish(&VoiceAct { round, click }).unwrap();
    takes.recv_timeout(WAIT).unwrap()
}

/// Waits for the key page to show a key of `dots` characters.
fn shows_dots(heard: &Heard, dots: usize) {
    let dots = Redacted::new(u32::try_from(dots).unwrap());
    heard.wait(|s| match s {
        Said::View(page) => match page.panel.map(|p| p.sheet) {
            Some(VoiceSheet::Service(ServiceSheet { key, .. })) => (key == dots).then_some(()),
            _ => None,
        },
        _ => None,
    });
}

#[test]
fn another_modules_answer_of_the_same_round_is_never_sealed_and_its_own_is() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    host.put("cloud", true.into());
    let row = kx_mod_voice::services::table()
        .unwrap()
        .offered()
        .next()
        .unwrap();
    let entry = format!(
        r#""{}" = {{ on = 2026-11-01, terms = {} }}"#,
        row.id, row.terms
    );
    host.put("services", toml::Value::Table(entry.parse().unwrap()));
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let (asked, _ear) = takes(&bus);
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    let take = paste(&bus, &heard, &asked);
    assert_eq!(take.asker, ID);
    let vaults = ClipTake {
        asker: ModuleId::new("vault"),
        ..take
    };
    let (other, own) = (key(30), key(20));
    bus.publish(&vaults.answer(text(&other))).unwrap();
    bus.publish(&take.answer(text(&own))).unwrap();
    shows_dots(&heard, own.len());
    stop(&mut module, &mut host);
}

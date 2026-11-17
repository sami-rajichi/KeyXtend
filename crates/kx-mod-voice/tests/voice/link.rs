//! A card's link through the module's own service lookup (edge 23): the manifest requires the web link, so Link on
//! Groq's card opens its facts page in the browser.

use std::sync::Arc;

use kx_mod_voice::VoiceModule;
use kx_module_api::Bus;
use kx_module_api::ports::{CloudButton, VoiceAct, VoiceClick, VoiceOpen, VoiceSheet, VoiceTab};

use crate::rig::{Fakes, Heard, Said, WAIT, start, stop};

/// Clicks `click` on the view of round `round`, then waits for a view whose page `shows`; its round.
fn click(
    bus: &Bus,
    heard: &Heard,
    round: u32,
    click: VoiceClick,
    shows: fn(&VoiceSheet) -> bool,
) -> u32 {
    bus.publish(&VoiceAct { round, click }).unwrap();
    heard.wait(|s| match s {
        Said::View(page) => page
            .panel
            .as_ref()
            .is_some_and(|p| shows(&p.sheet))
            .then_some(page.round),
        _ => None,
    })
}

#[test]
fn link_on_a_cards_facts_reaches_the_browser_through_the_modules_own_lookup() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    host.put("cloud", true.into());
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    bus.publish(&VoiceOpen).unwrap();
    let round = heard.wait(|s| match s {
        Said::View(page) if page.panel.is_some() => Some(page.round),
        _ => None,
    });
    let cloud = |s: &VoiceSheet| matches!(s, VoiceSheet::Cloud(_));
    let round = click(
        &bus,
        &heard,
        round,
        VoiceClick::Open(VoiceTab::Cloud),
        cloud,
    );
    let ticked =
        |s: &VoiceSheet| matches!(s, VoiceSheet::Cloud(c) if c.acts.contains(&CloudButton::Facts));
    let round = click(&bus, &heard, round, VoiceClick::Row(0), ticked);
    let card = |s: &VoiceSheet| matches!(s, VoiceSheet::Card(_));
    let facts = VoiceClick::Service(CloudButton::Facts);
    let round = click(&bus, &heard, round, facts, card);
    let link = VoiceAct {
        round,
        click: VoiceClick::Service(CloudButton::Link),
    };
    bus.publish(&link).unwrap();
    assert!(fakes.link.wait_opened(1, WAIT), "Link reached the browser");
    assert!(fakes.link.opened()[0].expose().starts_with("https://"));
    stop(&mut module, &mut host);
}

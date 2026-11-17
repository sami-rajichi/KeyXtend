//! A card's link through the module's own service lookup (edge 23): the manifest requires the web link, so Link on
//! Groq's card opens its facts page in the browser.

use std::sync::Arc;

use kx_mod_voice::VoiceModule;
use kx_module_api::Bus;
use kx_module_api::ports::{CloudButton, VoiceAct, VoiceClick, VoiceSheet};

use crate::rig::{Fakes, Heard, WAIT, click, first_service, start, stop};

#[test]
fn link_on_a_cards_facts_reaches_the_browser_through_the_modules_own_lookup() {
    let fakes = Fakes::new();
    let mut host = fakes.host();
    host.put("cloud", true.into());
    let (heard, bus) = (Heard::listen(&host), Bus::new(Arc::clone(&host.core)));
    let mut module = VoiceModule::new();
    start(&mut module, &mut host).unwrap();
    let round = first_service(&bus, &heard);
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

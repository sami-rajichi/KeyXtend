//! The actor over the fakes: the key and the light (`key`), the views and their times (`view`), the faults (`faults`),
//! the text and Copy (`typing`), Paste at once (`paste`), the store (`store`), the panel (`panel`), the cards' checks
//! (`cards`, `auto`), Compare (`compare`), cloud dictation (`cloud`) and the logs (`logs`); `rig` drives it without a
//! thread, hearing it through `taps`. Here: the start and the stop.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod auto;
mod cards;
mod cloud;
mod compare;
mod fakes;
mod faults;
mod key;
mod logs;
mod panel;
mod paste;
mod rig;
mod store;
mod taps;
mod typing;
mod view;

use kx_module_api::Event;
use kx_module_api::ports::{LayoutAsk, VoiceBar, VoiceLit};
use kx_platform_fake::FileCall;

use super::Msg;
use rig::Rig;
use taps::Heard;

#[test]
fn the_start_tells_a_hidden_bar_and_no_light_then_asks_for_the_layout() {
    let mut rig = Rig::unbegun(|_, _| {});
    rig.actor.begin();
    rig.pump();
    let heard = rig.heard();
    assert!(
        matches!(&heard[0], Heard::View(page) if page.bar == VoiceBar::Hidden && page.panel.is_none())
    );
    assert_eq!(heard[1], Heard::Lit(VoiceLit::Off));
    assert_eq!(heard.last(), Some(&Heard::Ask(LayoutAsk::NAME)));
}

#[test]
fn the_start_reads_the_models_folder_and_watches_the_box_and_the_session() {
    let rig = Rig::new();
    assert_eq!(rig.fake.files.calls(FileCall::Scan), 1);
    assert_eq!(rig.fake.files.calls(FileCall::Fingerprint), 1);
    assert!(rig.fake.field.watching() && rig.fake.session.watching());
}

#[test]
fn the_stop_closes_the_microphone_hides_the_bar_and_drops_the_watches() {
    let mut rig = Rig::new();
    rig.listen();
    rig.send(Msg::Quit);
    let run = &rig.fake.speech.runs()[0];
    assert!(run.closed && !run.held, "the recording is wiped");
    assert_eq!(
        (rig.bar(), rig.lits().last()),
        (VoiceBar::Hidden, Some(&VoiceLit::Off))
    );
    assert!(!rig.fake.field.watching() && !rig.fake.session.watching());
}

#[test]
fn nothing_is_handled_after_the_stop() {
    let mut rig = Rig::new();
    rig.send(Msg::Quit);
    let told = rig.heard().len();
    rig.key();
    rig.send(Msg::Open);
    rig.send(Msg::Quit);
    assert_eq!(rig.heard().len(), told);
    assert!(rig.fake.speech.runs().is_empty());
}

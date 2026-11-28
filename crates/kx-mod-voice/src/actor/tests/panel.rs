//! The panel's effects on the ports: the models folder shown through the folder reveal, a setting written and
//! followed, a download started and its notes heard, a file fingerprinted once, and the devices read off the actor.

use std::path::Path;

use kx_module_api::ports::{
    BarButton, BarHint, FetchNote, FolderError, LayoutChanged, LayoutId, PanelButton, VoiceBar,
    VoiceClick, VoiceTab,
};
use kx_platform_fake::FileCall;

use super::Msg;
use super::rig::{Rig, dir};
use super::taps::{held, release, waiting};

/// The note band of the panel told last.
fn band(rig: &Rig) -> Option<BarHint> {
    rig.views().pop().and_then(|v| v.panel).and_then(|p| p.note)
}

#[test]
fn open_shows_the_models_folder_through_the_folder_reveal() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Models));
    rig.act(VoiceClick::Button(PanelButton::Open));
    let shown = rig.fake.reveal.shown();
    assert!(matches!(&shown[..], [f] if f.expose() == Path::new(&dir())));
    assert_eq!(band(&rig), None);
}

#[test]
fn a_refused_reveal_shows_the_folder_note() {
    let mut rig = Rig::new();
    rig.fake.reveal.set_error(Some(FolderError::Missing));
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Models));
    rig.act(VoiceClick::Button(PanelButton::Open));
    assert_eq!(band(&rig), Some(BarHint::Folder));
}

#[test]
fn a_switch_the_panel_flips_is_written_and_followed() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Options));
    rig.act(VoiceClick::Flip(0));
    assert!(!rig.saved().sounds && !rig.actor.settings().sounds);
    assert!(!rig.views().pop().unwrap().sounds, "the flow follows it");
}

#[test]
fn download_on_the_bar_starts_the_languages_model_and_its_end_reads_the_folder() {
    let mut rig = Rig::new();
    let arabic = LayoutChanged {
        id: LayoutId::new(2),
        iso: "ar".into(),
        name: "العربية".into(),
        rtl: true,
    };
    rig.send(Msg::Layout(arabic));
    rig.key();
    assert!(matches!(rig.bar(), VoiceBar::Note(n) if n.act == Some(BarButton::Download)));
    rig.act(VoiceClick::Bar(BarButton::Download));
    assert_eq!(rig.fake.files.fetches().len(), 1);
    let scans = rig.fake.files.calls(FileCall::Scan);
    assert!(rig.fake.files.tell(FetchNote::Done));
    rig.pump();
    assert_eq!(rig.fake.files.calls(FileCall::Scan), scans + 1);
}

#[test]
fn a_file_is_fingerprinted_once_however_often_the_folder_is_read() {
    let mut rig = Rig::unbegun(|_, _| {});
    rig.actor.ports.spawn = held;
    rig.actor.begin();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Models));
    assert_eq!(rig.fake.files.calls(FileCall::Scan), 2);
    while waiting() {
        release();
        rig.pump();
    }
    assert_eq!(rig.fake.files.calls(FileCall::Fingerprint), 1);
}

#[test]
fn the_device_page_reads_the_devices_with_the_jobs_limits() {
    let mut rig = Rig::new();
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Device));
    let asks = rig.fake.speech.device_asks();
    assert_eq!(asks, [rig.actor.settings().job()]);
}

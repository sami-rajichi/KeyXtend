//! The cards' checks on the actor: a change of the language in front checks its model on the picked card, one check
//! at a time and never during a dictation or Compare; a check held back starts once they end, and nothing loops.

use std::path::Path;
use std::sync::Arc;

use kx_module_api::ports::{
    CardId, CheckDone, Device, DeviceKind, LayoutChanged, LayoutId, MAX_BLOB_BYTES, PanelButton,
    SpeechError, SpeechNote, SpeechStage, VoiceClick, VoiceTab,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use kx_platform_fake::SpeechCall;

use super::Msg;
use super::rig::{DIR, Rig, done, english, english_layout};
use super::taps::{held, release, waiting};
use crate::counts::{Checked, Checks};
use crate::models::{Row, table};
use crate::panel::rig::{card, cpu};
use crate::store::Store;

mod langs;

/// The card the rig's devices hold, as `device` names it.
pub(super) const CARD: &str = "Vulkan0";
/// A second card, which only the tests that list it hold.
pub(super) const CARD2: &str = "Vulkan1";
/// The processor's memory, roomy for every model.
const CPU_MB: u32 = 32_768;
/// A card's free memory, roomy for every model and the margin.
const CARD_MB: u32 = 4_096;

/// Arabic's default model, on a model of its own.
pub(super) fn arabic() -> &'static Row {
    table().unwrap().default_for("ar").unwrap()
}

/// The Arabic layout.
pub(super) fn arabic_layout() -> LayoutChanged {
    LayoutChanged {
        id: LayoutId::new(2),
        iso: "ar".into(),
        name: "العربية".into(),
        rtl: true,
    }
}

pub(super) fn card_id() -> CardId {
    CardId::new(CARD).unwrap()
}

/// The processor and the roomy card.
pub(super) fn devices() -> Vec<Device> {
    vec![cpu(CPU_MB), card(CARD_MB)]
}

/// The processor and two roomy cards under the same driver.
pub(super) fn two_cards() -> Vec<Device> {
    let mut second = card(CARD_MB);
    second.kind = DeviceKind::Card(CardId::new(CARD2).unwrap());
    vec![cpu(CPU_MB), card(CARD_MB), second]
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// A check where the card wrote the clip as the processor did, three times faster.
pub(super) fn passed() -> CheckDone {
    CheckDone {
        card_ms: 1_000,
        cpu_ms: 3_000,
        card_text: secret("hello"),
        cpu_text: secret("hello"),
    }
}

/// A check where the card wrote two words the processor did not, past `card_diff`.
pub(super) fn differs() -> CheckDone {
    CheckDone {
        card_text: secret("hello there world"),
        ..passed()
    }
}

/// Cohere Arabic serving English too, as `picks` reads it.
pub(super) const BOTH: &str = "en = \"cohere-arabic-q8\"";

/// A begun rig with `device` set, `kept` on the shelf from an earlier run, the three clips, Arabic's model beside
/// English's and each card's check answering `check`; `devices` answer the reads when given.
pub(super) fn build(
    device: &str,
    devices: Option<Vec<Device>>,
    check: Result<CheckDone, SpeechError>,
    kept: Vec<Checked>,
) -> Rig {
    build_picked(device, devices, check, kept, None)
}

/// As `build`, with `picks` set from a settings table's text when given.
pub(super) fn build_picked(
    device: &str,
    devices: Option<Vec<Device>>,
    check: Result<CheckDone, SpeechError>,
    kept: Vec<Checked>,
    picks: Option<&str>,
) -> Rig {
    Rig::build(|fake, host| {
        host.put("device", device.into());
        if let Some(picks) = picks {
            host.put("picks", toml::Value::Table(picks.parse().unwrap()));
        }
        if let Some(devices) = devices {
            fake.speech.set_devices(Ok(devices));
        }
        fake.speech.set_clips(&["en", "fr", "ar"]);
        let id = |id: &str| CardId::new(id).unwrap();
        fake.speech.set_check(&id(CARD), Vec::new(), check.clone());
        fake.speech.set_check(&id(CARD2), Vec::new(), check);
        let row = arabic();
        let file = Path::new(DIR).join(&row.file);
        fake.files.add_file(&file, row.bytes, row.print);
        let opened = Store::open(Arc::new(fake.shelf.clone()), MAX_BLOB_BYTES).unwrap();
        opened.store.save_checks(&Checks::new(kept)).unwrap();
    })
}

/// The rig with `device` set, its devices read at the start, each check passing and none kept.
fn picked(device: &str) -> Rig {
    build(device, Some(devices()), Ok(passed()), Vec::new())
}

/// The model file each check was asked for, in order.
pub(super) fn checked(rig: &Rig) -> Vec<String> {
    let asks = rig.fake.speech.check_asks();
    let name = |p: &Path| p.file_name().unwrap().to_string_lossy().into_owned();
    asks.iter().map(|a| name(a.model.file.expose())).collect()
}

/// English's and Arabic's model files.
pub(super) fn files() -> (&'static str, &'static str) {
    (english().file.as_str(), arabic().file.as_str())
}

/// The card the newest dictation runs on; `None` for the processor.
pub(super) fn served(rig: &Rig) -> Option<CardId> {
    let runs = rig.fake.speech.runs();
    runs.last().unwrap().ask.card.as_ref().map(|c| c.id.clone())
}

/// Ends the jobs held so far and hands their answers to the actor; the jobs those start stay held.
pub(super) fn finish(rig: &mut Rig) {
    release();
    rig.pump();
}

/// Opens the panel on the Device page, which reads the devices.
fn on_device(rig: &mut Rig) {
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Device));
}

/// Opens Compare's page and starts its recording.
pub(super) fn start_compare(rig: &mut Rig) {
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Compare));
    let open = vec![SpeechNote::Stage(SpeechStage::Open)];
    rig.fake.speech.on(SpeechCall::Open, open);
    rig.act(VoiceClick::Compare(Redacted::new("hello world".to_owned())));
}

/// Dictates past a change to the Arabic layout made while it listens.
pub(super) fn dictate_past_arabic(rig: &mut Rig) {
    rig.listen();
    rig.send(Msg::Layout(arabic_layout()));
    rig.key();
}

/// The running dictation writes its text and ends.
pub(super) fn written(rig: &mut Rig) {
    let text = secret("hello");
    rig.tell(SpeechNote::Piece { index: 0, text });
    rig.tell(SpeechNote::Done(done(1)));
}

#[test]
fn a_new_language_checks_its_model_on_the_picked_card_and_then_dictates_on_it() {
    let (en, ar) = files();
    let mut rig = picked(CARD);
    on_device(&mut rig);
    assert_eq!(checked(&rig), [en], "English's model is checked");
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(checked(&rig), [en, ar]);
    rig.listen();
    assert_eq!(served(&rig), Some(card_id()));
}

#[test]
fn the_first_layout_checks_its_model_when_the_devices_came_first() {
    let rig = picked(CARD);
    assert_eq!(checked(&rig), [files().0]);
}

#[test]
fn a_change_during_a_check_waits_for_it_and_switching_back_and_forth_piles_up_none() {
    let (en, ar) = files();
    let mut rig = build(CARD, None, Ok(passed()), Vec::new());
    rig.actor.ports.spawn = held;
    rig.fake.speech.set_devices(Ok(devices()));
    on_device(&mut rig);
    finish(&mut rig);
    for layout in [arabic_layout(), english_layout(), arabic_layout()] {
        rig.send(Msg::Layout(layout));
    }
    finish(&mut rig);
    assert_eq!(checked(&rig), [en], "one check at a time");
    assert!(waiting(), "Arabic's check starts as English's ends");
    finish(&mut rig);
    assert!(!waiting(), "no check piles up");
    rig.send(Msg::Layout(english_layout()));
    rig.send(Msg::Layout(arabic_layout()));
    assert!(!waiting(), "a checked model is not checked again");
    assert_eq!(checked(&rig), [en, ar]);
}

#[test]
fn a_change_during_a_dictation_checks_once_it_has_written() {
    let (en, ar) = files();
    let mut rig = picked(CARD);
    dictate_past_arabic(&mut rig);
    assert_eq!(checked(&rig), [en], "nothing while it listens or writes");
    written(&mut rig);
    assert_eq!(checked(&rig), [en, ar]);
}

#[test]
fn a_change_during_compare_checks_once_it_ends() {
    let (en, ar) = files();
    let mut rig = picked(CARD);
    start_compare(&mut rig);
    rig.send(Msg::Layout(arabic_layout()));
    assert_eq!(checked(&rig), [en], "nothing while Compare runs");
    rig.act(VoiceClick::Button(PanelButton::Cancel));
    assert_eq!(checked(&rig), [en, ar]);
}

#[test]
fn the_processor_pick_and_automatic_with_no_card_passed_check_nothing_on_a_switch() {
    for device in ["cpu", "auto"] {
        let mut rig = picked(device);
        on_device(&mut rig);
        rig.send(Msg::Layout(arabic_layout()));
        assert!(checked(&rig).is_empty(), "{device}");
        rig.listen();
        assert_eq!(served(&rig), None, "{device}");
    }
}

#[test]
fn a_check_that_faults_is_not_run_again_by_itself_and_the_processor_serves() {
    let (en, ar) = files();
    let mut rig = build(CARD, Some(devices()), Err(SpeechError::Timeout), Vec::new());
    rig.send(Msg::Layout(arabic_layout()));
    rig.send(Msg::Ask);
    rig.listen();
    assert_eq!(checked(&rig), [en, ar], "a fault never loops");
    assert_eq!(served(&rig), None);
}

#[test]
fn a_failed_verdict_is_kept_so_switching_back_and_forth_checks_nothing_again() {
    let (en, ar) = files();
    let failed = Err(SpeechError::CardFailed);
    let mut rig = build(CARD, Some(devices()), failed, Vec::new());
    for layout in [arabic_layout(), english_layout(), arabic_layout()] {
        rig.send(Msg::Layout(layout));
    }
    assert_eq!(checked(&rig), [en, ar]);
    rig.listen();
    assert_eq!(served(&rig), None);
}

//! The Device page through the panel: Automatic, the processor and the cards; Use writes `device` and checks a card
//! not checked for the model in use; the check's warm-up shows on its row and its verdict is kept.

use kx_module_api::ports::{
    BarHint, CardCheck, CardId, CheckDone, DeviceItem, DeviceSheet, PanelButton, SpeechError,
    Verdict, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::rig::{Rig, card, row};
use super::{Check, Do, Panel};
use crate::compare::tests::secret;
use crate::settings::DEVICE;

pub(super) mod clips;

/// The Device page as shown.
fn device(rig: &mut Rig) -> DeviceSheet {
    match rig.view().unwrap().sheet {
        VoiceSheet::Device(sheet) => sheet,
        other => panic!("not Device: {other:?}"),
    }
}

/// The rig on the Device page with Parakeet in the folder and a card of 4 GB free.
fn on_device() -> Rig {
    let mut rig = Rig::foldered();
    rig.devices.push(card(4_096));
    rig.holds(&[row("parakeet-v3-q8")]);
    rig.click(&VoiceClick::Open(VoiceTab::Device));
    rig
}

/// Ticks the card, unless it is, and presses Check, giving the check asked.
fn check_card(rig: &mut Rig) -> Check {
    if rig.view().unwrap().ticked == 0 {
        rig.click(&VoiceClick::Row(2));
    }
    let out = rig.click(&VoiceClick::Button(PanelButton::Check));
    let [Do::Check(check)] = &out[..] else {
        panic!("{out:?}");
    };
    (**check).clone()
}

#[test]
fn the_device_page_lists_automatic_the_processor_and_the_card() {
    let mut rig = on_device();
    let sheet = device(&mut rig);
    assert_eq!(sheet.rows.len(), 3);
    assert!(sheet.rows[0].used && sheet.rows[0].item == DeviceItem::Auto);
    assert!(matches!(
        sheet.rows[1].item,
        DeviceItem::Cpu { cores: 6, .. }
    ));
    assert!(matches!(
        sheet.rows[2].item,
        DeviceItem::Card {
            check: CardCheck::Unchecked,
            built_in: false,
            ..
        }
    ));
    assert_eq!(sheet.failed, None);
}

#[test]
fn built_in_graphics_shows_its_mark_from_the_port() {
    let mut rig = on_device();
    rig.devices[1].built_in = true;
    let sheet = device(&mut rig);
    assert!(matches!(
        sheet.rows[2].item,
        DeviceItem::Card { built_in: true, .. }
    ));
}

#[test]
fn a_model_knowing_no_shipped_clip_is_not_checked() {
    let mut rig = on_device();
    rig.clips = vec!["ar".into()];
    rig.click(&VoiceClick::Row(2));
    let out = rig.click(&VoiceClick::Button(PanelButton::Check));
    assert!(out.is_empty(), "no shipped clip it knows: {out:?}");
}

#[test]
fn use_on_an_unchecked_card_writes_device_and_checks_it() {
    let parakeet = row("parakeet-v3-q8");
    let mut rig = on_device();
    rig.click(&VoiceClick::Row(2));
    let acts = rig.view().unwrap().acts;
    assert!(acts.contains(&PanelButton::Use) && acts.contains(&PanelButton::Check));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    assert_eq!(
        out[0],
        Do::Set(DEVICE, toml::Value::String("Vulkan0".into()))
    );
    let Some(Do::Check(check)) = out.get(1) else {
        panic!("{out:?}");
    };
    assert_eq!(
        (check.clip.as_str(), check.print, check.driver.as_str()),
        ("en", parakeet.print, "1.0")
    );
    assert_eq!(
        (check.ask.card.need_mb, check.ask.model.lang.as_str()),
        (parakeet.card_mb, "en")
    );
}

#[test]
fn a_running_check_shows_its_warm_up_and_dims_check() {
    let mut rig = on_device();
    check_card(&mut rig);
    rig.panel.warming(&CardId::new("Vulkan0").unwrap(), 40);
    let sheet = device(&mut rig);
    assert!(matches!(
        sheet.rows[2].item,
        DeviceItem::Card {
            check: CardCheck::Running { percent: 40 },
            ..
        }
    ));
    assert!(!rig.view().unwrap().acts.contains(&PanelButton::Check));
}

#[test]
fn a_check_ends_with_its_verdict_kept() {
    let mut rig = on_device();
    let check = check_card(&mut rig);
    let done = CheckDone {
        card_ms: 1_000,
        cpu_ms: 3_000,
        card_text: secret("hello"),
        cpu_text: secret("hello"),
    };
    let out = rig.with(|p, f| p.checked(&check, Ok(done), f));
    let [Do::Checked(checked)] = &out[..] else {
        panic!("{out:?}");
    };
    assert_eq!(
        checked.verdict,
        Verdict::Passed {
            gain: kx_module_api::ports::Hundredths(300)
        }
    );
    rig.checks.put(checked.clone());
    let sheet = device(&mut rig);
    assert!(matches!(
        sheet.rows[2].item,
        DeviceItem::Card {
            check: CardCheck::Done(Verdict::Passed { .. }),
            ..
        }
    ));
}

#[test]
fn a_failed_check_shows_in_the_band_and_a_fault_only_a_note() {
    let mut rig = on_device();
    let check = check_card(&mut rig);
    let out = rig.with(|p, f| p.checked(&check, Err(SpeechError::CardFailed), f));
    let [Do::Checked(checked)] = &out[..] else {
        panic!("{out:?}");
    };
    rig.checks.put(checked.clone());
    assert_eq!(
        device(&mut rig).failed.map(|f| f.verdict),
        Some(Verdict::Failed)
    );
    let again = check_card(&mut rig);
    assert!(
        rig.with(|p, f| p.checked(&again, Err(SpeechError::Timeout), f))
            .is_empty()
    );
    assert_eq!(
        rig.view().unwrap().note,
        Some(kx_module_api::ports::BarHint::Fault)
    );
}

#[test]
fn automatic_checks_a_card_again_after_its_driver_changed() {
    let mut rig = on_device();
    let check = check_card(&mut rig);
    let out = rig.with(|p, f| p.checked(&check, Ok(passed()), f));
    let [Do::Checked(checked)] = &out[..] else {
        panic!("{out:?}");
    };
    rig.checks.put(checked.clone());
    assert!(
        rig.with(Panel::recheck).is_empty(),
        "checked under this driver"
    );
    rig.devices[1].driver = "2.0".into();
    let out = rig.with(Panel::recheck);
    let [Do::Check(again)] = &out[..] else {
        panic!("{out:?}");
    };
    assert_eq!(again.driver, "2.0");
}

#[test]
fn a_card_gone_during_its_check_keeps_no_verdict() {
    let mut rig = on_device();
    let check = check_card(&mut rig);
    let out = rig.with(|p, f| p.checked(&check, Err(SpeechError::NoCard), f));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(rig.view().unwrap().note, Some(BarHint::Fault));
}

#[test]
fn a_recheck_held_back_by_a_dictation_or_a_check_runs_once_through_dequeue() {
    let mut rig = on_device();
    rig.settings.device = "Vulkan0".into();
    rig.dictating = true;
    assert!(rig.with(Panel::recheck).is_empty(), "not while it dictates");
    assert!(rig.with(Panel::dequeue).is_empty(), "still dictating");
    rig.dictating = false;
    let out = rig.with(Panel::dequeue);
    let [Do::Check(check)] = &out[..] else {
        panic!("{out:?}");
    };
    let check = (**check).clone();
    assert!(rig.with(Panel::recheck).is_empty(), "one check at a time");
    let out = rig.with(|p, f| p.checked(&check, Ok(passed()), f));
    let [Do::Checked(checked)] = &out[..] else {
        panic!("{out:?}");
    };
    rig.checks.put(checked.clone());
    assert!(rig.with(Panel::dequeue).is_empty(), "the model is checked");
    assert!(!rig.panel.queued, "nothing waits any more");
}

/// A check where the card wrote the clip three times faster.
fn passed() -> CheckDone {
    CheckDone {
        card_ms: 1_000,
        cpu_ms: 3_000,
        card_text: secret("hello"),
        cpu_text: secret("hello"),
    }
}

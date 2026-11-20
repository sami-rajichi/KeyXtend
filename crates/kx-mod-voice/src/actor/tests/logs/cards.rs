//! The device lines: why a dictation runs on the processor although a card is picked, and how each card's check ended.
//! Each holds kinds and numbers only.

use kx_module_api::ports::{FieldKind, Hundredths, SpeechError, Verdict};
use kx_platform_fake::FileCall;

use super::super::Msg;
use super::super::cards::{CARD, build, card_id, devices, passed, served};
use super::super::rig::{Rig, english_layout};
use super::super::taps::{held, in_place, release, waiting};
use super::lines_of;
use crate::device::Why;
use crate::device::told::{
    self, CARD_MS, CHECK_ENDED, CPU_MS, CPU_SERVES, ERROR, FREE_MB, NEED_MB, VERDICT, WHY,
};

/// A card the rig's devices do not hold.
const GONE: &str = "Vulkan9";
/// What a model and the margin need on a card, in MB.
const NEED: u32 = 900;
/// A card's free memory, too little for that, in MB.
const FREE: u32 = 300;

/// A text field as a captured line shows it.
fn text(name: &str, value: &str) -> String {
    format!("{name}={value:?} ")
}

/// A number field as a captured line shows it.
fn number(name: &str, value: u32) -> String {
    format!("{name}={value} ")
}

/// The fields of each of `lines` with `message`, the message left out.
fn fields(lines: &[String], message: &str) -> Vec<String> {
    let mark = format!("message={message} ");
    let of = |l: &String| l.contains(&mark).then(|| l.replacen(&mark, "", 1));
    lines.iter().filter_map(of).collect()
}

#[test]
fn the_processor_serving_a_picked_card_logs_its_why_kind_alone() {
    let slow = Verdict::Slow {
        gain: Hundredths(150),
        need: Hundredths(200),
    };
    let lines = lines_of(|| {
        told::cpu(Why::Gone);
        told::cpu(Why::Unchecked);
        told::cpu(Why::Unread);
        told::cpu(Why::Failed(slow));
        told::cpu(Why::NoRoom {
            need_mb: NEED,
            free_mb: FREE,
        });
    });
    let want = [
        text(WHY, "gone"),
        text(WHY, "unchecked"),
        text(WHY, "unread"),
        text(WHY, "failed") + &text(VERDICT, "slow"),
        text(WHY, "no_room") + &number(NEED_MB, NEED) + &number(FREE_MB, FREE),
    ];
    assert_eq!(fields(&lines, CPU_SERVES), want);
}

#[test]
fn a_dictation_says_why_the_processor_serves_only_when_a_card_is_picked() {
    let lines = lines_of(|| {
        let mut rig = build(GONE, Some(devices()), Ok(passed()), Vec::new());
        rig.listen();
        assert_eq!(served(&rig), None);
    });
    assert_eq!(fields(&lines, CPU_SERVES), [text(WHY, "gone")]);
    for device in ["cpu", "auto", CARD] {
        let lines = lines_of(|| {
            let mut rig = build(device, Some(devices()), Ok(passed()), Vec::new());
            rig.listen();
        });
        assert!(fields(&lines, CPU_SERVES).is_empty(), "{device}: {lines:?}");
    }
}

#[test]
fn a_card_checks_end_logs_its_verdict_and_times_or_its_error_kind() {
    let lines = lines_of(|| {
        let mut rig = build(CARD, Some(devices()), Ok(passed()), Vec::new());
        rig.listen();
        assert_eq!(served(&rig), Some(card_id()));
    });
    let done = passed();
    let times = number(CARD_MS, done.card_ms) + &number(CPU_MS, done.cpu_ms);
    let want = text(VERDICT, "passed") + &times;
    assert_eq!(fields(&lines, CHECK_ENDED), [want]);
    for error in [SpeechError::Timeout, SpeechError::CardRoom] {
        let lines = lines_of(|| drop(build(CARD, Some(devices()), Err(error), Vec::new())));
        assert_eq!(fields(&lines, CHECK_ENDED), [text(ERROR, error.kind())]);
    }
}

/// A begun rig with `device` picked whose model's fingerprint is still being read: that job alone is held.
fn unread(device: &str) -> Rig {
    let mut rig = Rig::unbegun(|fake, host| {
        host.put("device", device.into());
        fake.speech.set_devices(Ok(devices()));
    });
    rig.actor.ports.spawn = held;
    rig.actor.begin();
    // Jobs end until the folder's scan has, so the fingerprint's job it starts stays held; later jobs run.
    while rig.fake.files.calls(FileCall::Scan) == 0 && waiting() {
        release();
        rig.pump();
    }
    rig.actor.ports.spawn = in_place;
    rig.fake.field.focus(FieldKind::Text);
    rig.pump();
    rig.send(Msg::Layout(english_layout()));
    rig
}

#[test]
fn a_picked_card_says_its_model_is_unread_while_the_fingerprint_is_read() {
    let lines = lines_of(|| {
        let mut rig = unread(CARD);
        assert!(waiting(), "the fingerprint is still read");
        rig.listen();
        assert_eq!(served(&rig), None);
        release();
    });
    assert_eq!(fields(&lines, CPU_SERVES), [text(WHY, "unread")]);
    let lines = lines_of(|| {
        unread("cpu").listen();
        release();
    });
    assert!(fields(&lines, CPU_SERVES).is_empty(), "{lines:?}");
}

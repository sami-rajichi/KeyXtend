//! The Voice panel's Device page the Qt Quick tests draw: Automatic, the processor and three cards, each check shown.

use kx_module_api::ports::{
    CardCheck, DeviceItem, DeviceRow, DeviceSheet, FailedCheck, Hundredths, Memory, Verdict,
    VoiceSheet,
};

/// A card named `name` with `mb` of memory, `[total, free]`, and its check.
fn card(name: &str, built_in: bool, mb: [u32; 2], check: CardCheck) -> DeviceItem {
    DeviceItem::Card {
        name: name.to_owned(),
        built_in,
        memory: Memory {
            total_mb: mb[0],
            free_mb: mb[1],
        },
        check,
    }
}

/// The built-in graphics whose check failed, and why.
const AMD: &str = "AMD Radeon 780M Graphics";
const DIFFERS: Verdict = Verdict::Differs {
    words: 3,
    allowed: 1,
};

/// The cards: one that passed, the built-in graphics that failed and one being checked.
fn cards() -> [DeviceItem; 3] {
    let passed = CardCheck::Done(Verdict::Passed {
        gain: Hundredths(900),
    });
    let nvidia = "NVIDIA GeForce RTX 4060 Laptop GPU";
    let checking = CardCheck::Running { percent: 40 };
    [
        card(nvidia, false, [8192, 7270], passed),
        card(AMD, true, [4096, 2662], CardCheck::Done(DIFFERS)),
        card("Intel Arc A370M", false, [4096, 3900], checking),
    ]
}

/// Automatic in use, the processor, then the cards, the last one ticked.
pub fn device() -> VoiceSheet {
    let row = |item, used, tick| DeviceRow { item, used, tick };
    let cpu = DeviceItem::Cpu {
        cores: 8,
        memory: Memory {
            total_mb: 16384,
            free_mb: 9626,
        },
    };
    let [nvidia, amd, intel] = cards();
    let rows = vec![
        row(DeviceItem::Auto, true, false),
        row(cpu, false, false),
        row(nvidia, false, false),
        row(amd, false, false),
        row(intel, false, true),
    ];
    let failed = Some(FailedCheck {
        name: AMD.to_owned(),
        verdict: DIFFERS,
    });
    VoiceSheet::Device(DeviceSheet { rows, failed })
}

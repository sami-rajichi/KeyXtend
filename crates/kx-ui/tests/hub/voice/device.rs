//! The Device page: Automatic, the processor with its cores and memory, each card or built-in graphics with its memory
//! and check, In use over a verdict, a running check over any mark, and the failed check's band with its reason.

use super::{acting, button, buttons, heard, panel, panel_json};
use crate::Rig;
use crate::texts::rig;
use kx_module_api::ports::{
    CardCheck, DeviceItem, DeviceRow, DeviceSheet, FailedCheck, Hundredths, Memory, PanelButton,
    Verdict, VoiceSheet,
};
use serde_json::{Value, json};

/// Row `item`, unticked.
const fn row(item: DeviceItem, used: bool) -> DeviceRow {
    DeviceRow {
        item,
        used,
        tick: false,
    }
}

/// A card named `name` with `total_mb` and `free_mb`, checked as `check`.
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

/// The Device page with `rows` and `failed`, as QML reads it in English.
fn page(rows: Vec<DeviceRow>, failed: Option<FailedCheck>) -> Value {
    let rig = Rig::new();
    let heard = heard(&rig);
    let acts = [PanelButton::Use];
    panel_json(
        &rig,
        &heard,
        acting(VoiceSheet::Device(DeviceSheet { rows, failed }), &acts),
    )
}

/// Automatic in use, the processor, a passed card, a failed card and unchecked built-in graphics.
fn devices() -> Vec<DeviceRow> {
    let cpu = DeviceItem::Cpu {
        cores: 8,
        memory: Memory {
            total_mb: 16_384,
            free_mb: 9_626,
        },
    };
    let passed = CardCheck::Done(Verdict::Passed {
        gain: Hundredths(900),
    });
    let failed = CardCheck::Done(Verdict::NoRoom);
    vec![
        row(DeviceItem::Auto, true),
        row(cpu, false),
        row(card("RTX 4060", false, [8192, 7270], passed), false),
        row(card("RTX 2050", false, [4096, 100], failed), false),
        row(
            card("Radeon 780M", true, [4096, 3000], CardCheck::Unchecked),
            false,
        ),
    ]
}

#[test]
fn the_page_names_automatic_and_the_processor_then_check_and_use() {
    let p = page(devices(), None);
    assert_eq!(
        (&p["page"], &p["title"]),
        (&json!("device"), &json!("Device"))
    );
    assert_eq!(p["sub"], "Cards are checked before use");
    let rows = &p["sheet"]["rows"];
    let auto = json!({"kind": "auto", "name": "Automatic", "line": "The processor until a card passes its check",
        "mark": {"id": "used", "text": "In use"}, "check": null, "tick": false, "used": true});
    assert_eq!(rows[0], auto);
    assert_eq!(
        (&rows[1]["kind"], &rows[1]["name"]),
        (&json!("cpu"), &json!("Processor"))
    );
    assert_eq!(rows[1]["line"], "8 cores · 16 GB memory, 9.4 GB free");
    assert_eq!(rows[1]["mark"], Value::Null);
    let foot = [
        button("button", "check", "Check", false, "plain"),
        button("button", "use", "Use", true, "primary"),
    ];
    assert_eq!(buttons(&p["foot"]), foot);
}

#[test]
fn each_card_shows_its_memory_and_its_checks_verdict() {
    let p = page(devices(), None);
    let rows = &p["sheet"]["rows"];
    assert_eq!(rows[2]["kind"], "card");
    assert_eq!(rows[2]["line"], "Graphics card · 8 GB, 7.1 GB free");
    assert_eq!(
        rows[2]["mark"],
        json!({"id": "passed", "text": "Passed · 9× faster"})
    );
    assert_eq!(
        rows[3]["mark"],
        json!({"id": "failed", "text": "Check failed"})
    );
    assert_eq!(rows[4]["kind"], "built-in");
    assert_eq!(
        rows[4]["line"],
        "Built-in graphics · 4 GB shared, 2.9 GB free"
    );
    assert_eq!(rows[4]["mark"], Value::Null);
}

#[test]
fn a_running_check_shows_its_share_over_any_mark() {
    let running = card(
        "RTX 4060",
        false,
        [8192, 7270],
        CardCheck::Running { percent: 40 },
    );
    let p = page(vec![row(running, true)], None);
    let row = &p["sheet"]["rows"][0];
    assert_eq!(
        row["check"],
        json!({"line": "Checking: warming up the card", "share": 0.4})
    );
    assert_eq!(row["mark"], Value::Null, "not even In use");
}

#[test]
fn the_band_says_why_a_check_failed_with_its_numbers() {
    let cases = [
        (
            Verdict::Differs {
                words: 3,
                allowed: 1,
            },
            "RTX 2050: check failed. Its text differed by 3 words; 1 word allowed.",
        ),
        (
            Verdict::Slow {
                gain: Hundredths(120),
                need: Hundredths(150),
            },
            "RTX 2050: check failed. It was only 1.2× faster; 1.5× needed.",
        ),
        (
            Verdict::NoRoom,
            "RTX 2050: check failed. The card had too little free memory.",
        ),
        (
            Verdict::Failed,
            "RTX 2050: check failed. The card reported an error.",
        ),
    ];
    for (verdict, want) in cases {
        let failed = FailedCheck {
            name: "RTX 2050".to_owned(),
            verdict,
        };
        let p = page(Vec::new(), Some(failed));
        assert_eq!(
            p["top"],
            json!({"id": "failed", "text": want, "name": null})
        );
    }
}

#[test]
fn arabic_counts_cores_and_says_how_much_faster_in_the_accusative() {
    let (rig, _face, _host) = rig("en", "ar", "auto");
    let heard = heard(&rig);
    let cpu = DeviceItem::Cpu {
        cores: 2,
        memory: Memory::default(),
    };
    let passed = CardCheck::Done(Verdict::Passed {
        gain: Hundredths(200),
    });
    let rows = vec![
        row(cpu, false),
        row(card("RTX", false, [0, 0], passed), false),
    ];
    let sheet = VoiceSheet::Device(DeviceSheet { rows, failed: None });
    let p = panel_json(&rig, &heard, panel(sheet));
    let line = p["sheet"]["rows"][0]["line"].as_str().unwrap().to_owned();
    assert!(line.starts_with("نواتان"), "{line}");
    let mark = p["sheet"]["rows"][1]["mark"]["text"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(mark.ends_with("أسرع مرتين"), "{mark}");
}

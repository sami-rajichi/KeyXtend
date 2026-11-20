//! The Voice panel's pages: the home list's values, model names and folders that never print, a model not for this PC
//! and the Device page's numbers. Compare and History are in `compare`, Options in `options`.

use super::{english, hidden, keeps_secrets};
use crate::ports::voice::*;
use crate::ports::{Date, Hundredths, Memory, Pages, Verdict};

/// The tests' one made-up day, which facts, runs and history carry.
pub(super) fn day() -> Date {
    Date::new(2026, 11, 12).unwrap()
}

/// A made-up cloud service, as a line names it.
pub(super) fn service() -> ServiceName {
    ServiceName::Named("Service A".into())
}

/// A panel showing `sheet` in English, nothing ticked and Try again acting.
fn panel(sheet: VoiceSheet) -> VoicePanel {
    VoicePanel {
        sheet,
        lang: english(),
        pages: Pages { at: 0, count: 3 },
        ticked: 0,
        acts: vec![PanelButton::Again],
        note: None,
    }
}

/// `sheet`'s panel as `Debug` prints it, which must hold no secret.
pub(super) fn shown(sheet: VoiceSheet) -> String {
    let shown = format!("{:?}", panel(sheet));
    assert!(keeps_secrets(&shown), "{shown}");
    shown
}

/// Asserts `shown` holds each of `parts`.
pub(super) fn holds(shown: &str, parts: &[&str]) {
    for part in parts {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn the_home_list_names_its_language_compare_day_options_and_device() {
    let home = VoiceHome {
        model: Some(hidden(2)),
        device: DeviceLabel::Auto(DeviceName::Cpu),
        compared: Some(day()),
        history: 12,
        changed: 0,
        cloud: Some(2),
        tabs: vec![VoiceTab::Models],
    };
    let shown = shown(VoiceSheet::Home(home));
    let parts = [
        "English",
        "Auto(Cpu)",
        "month: 11",
        "changed: 0",
        "cloud: Some(2)",
        "Again",
    ];
    holds(&shown, &parts);
}

/// A model row in `state`, its name hidden, with `fetch` under way.
fn model_row(state: ModelState, fetch: Option<FetchState>) -> ModelRow {
    ModelRow {
        name: hidden(2),
        bytes: 1_530_000_000,
        default: false,
        state,
        used: false,
        tick: false,
        fetch,
        cloud: None,
    }
}

#[test]
fn model_names_and_folders_never_print_but_sizes_and_room_do() {
    let fetch = FetchState {
        done: 340_000_000,
        total: 740_000_000,
        paused: true,
    };
    let folder = || FolderLine {
        path: hidden(2),
        free: 212_000_000_000,
    };
    let models = ModelsSheet {
        rows: vec![
            model_row(ModelState::Other, None),
            model_row(ModelState::Missing, Some(fetch)),
        ],
        best: Some("Parakeet TDT 0.6b v3".into()),
        untested: false,
        slow: false,
        folder: Some(folder()),
        offer: Some(folder()),
        side: ModelSide::Local,
        band: None,
    };
    let shown = shown(VoiceSheet::Models(models));
    let parts = [
        "bytes: 1530000000",
        "free: 212000000000",
        "paused: true",
        "Parakeet",
    ];
    holds(&shown, &parts);
}

#[test]
fn a_model_not_for_this_pc_carries_its_need_the_total_and_the_free_memory() {
    let unfit = ModelState::NotHere {
        need_mb: 5_120,
        have_mb: 4_096,
        free_mb: 2_867,
    };
    let row = ModelRow {
        name: hidden(2),
        bytes: 2_410_655_136,
        default: true,
        state: unfit,
        used: false,
        tick: false,
        fetch: None,
        cloud: None,
    };
    let models = ModelsSheet {
        rows: vec![row],
        best: None,
        untested: false,
        slow: true,
        folder: None,
        offer: None,
        side: ModelSide::Local,
        band: None,
    };
    let shown = shown(VoiceSheet::Models(models));
    let parts = [
        "need_mb: 5120",
        "have_mb: 4096",
        "free_mb: 2867",
        "slow: true",
    ];
    holds(&shown, &parts);
}

/// The mock-up's devices: Automatic, an 8-core processor, a card that passed 9× faster and built-in graphics whose
/// text differed by 3 words.
fn devices() -> DeviceSheet {
    let memory = |total_mb, free_mb| Memory { total_mb, free_mb };
    let differs = Verdict::Differs {
        words: 3,
        allowed: 1,
    };
    let card = |name: &str, built_in, verdict| DeviceItem::Card {
        name: name.into(),
        built_in,
        memory: memory(8_192, 7_270),
        check: CardCheck::Done(verdict),
    };
    let gain = Hundredths(900);
    let cpu = DeviceItem::Cpu {
        cores: 8,
        memory: memory(16_384, 9_626),
    };
    let items = [
        DeviceItem::Auto,
        cpu,
        card("RTX 4060", false, Verdict::Passed { gain }),
        card("Radeon 780M", true, differs),
    ];
    let rows = items.map(|item| DeviceRow {
        item,
        used: false,
        tick: false,
    });
    let failed = Some(FailedCheck {
        name: "Radeon 780M".into(),
        verdict: differs,
    });
    DeviceSheet {
        rows: rows.into(),
        failed,
    }
}

#[test]
fn the_device_page_carries_cores_built_in_memory_and_the_checks_numbers() {
    let shown = shown(VoiceSheet::Device(devices()));
    let parts = [
        "cores: 8",
        "built_in: true",
        "Hundredths(900)",
        "words: 3, allowed: 1",
        "failed: Some",
    ];
    holds(&shown, &parts);
}

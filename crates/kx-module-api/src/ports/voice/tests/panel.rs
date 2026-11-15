//! The Voice panel's pages: the home list's order and values, model names and folders that never print, a model not
//! for this PC, the Device page's numbers, Compare's steps, day and device, and History's counts. Options are in
//! `options`.

use super::{SECRETS, english, hidden, keeps_secrets};
use crate::ports::voice::*;
use crate::ports::{Date, Hundredths, Memory, Pages, Verdict, When};

/// 12 Nov 2026, Compare's last run in the mock-up.
fn day() -> Date {
    Date::new(2026, 11, 12).unwrap()
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
fn the_panels_pages_keep_the_home_lists_order() {
    use VoiceTab::{Compare, Device, History, Models, Options};
    assert_eq!(VoiceTab::ALL, [Models, Device, Compare, History, Options]);
}

#[test]
fn the_home_list_names_its_language_compare_day_options_and_device() {
    let home = VoiceHome {
        model: Some(hidden(2)),
        device: DeviceLabel::Auto(DeviceName::Cpu),
        compared: Some(day()),
        history: 12,
        changed: 0,
    };
    let shown = shown(VoiceSheet::Home(home));
    let parts = ["English", "Auto(Cpu)", "month: 11", "changed: 0", "Again"];
    holds(&shown, &parts);
}

#[test]
fn model_names_and_folders_never_print_but_sizes_and_room_do() {
    let fetch = FetchState {
        done: 340_000_000,
        total: 740_000_000,
        paused: true,
    };
    let row = |state, fetch| ModelRow {
        name: hidden(2),
        bytes: 1_530_000_000,
        default: false,
        state,
        used: false,
        tick: false,
        fetch,
    };
    let folder = || FolderLine {
        path: hidden(2),
        free: 212_000_000_000,
    };
    let models = ModelsSheet {
        rows: vec![
            row(ModelState::Other, None),
            row(ModelState::Missing, Some(fetch)),
        ],
        best: Some("Parakeet TDT 0.6b v3".into()),
        untested: false,
        slow: false,
        folder: Some(folder()),
        offer: Some(folder()),
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
    };
    let models = ModelsSheet {
        rows: vec![row],
        best: None,
        untested: false,
        slow: true,
        folder: None,
        offer: None,
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

#[test]
fn comparing_shows_each_models_step_a_failed_one_too() {
    let run = |step| CompareRun {
        name: hidden(2),
        step,
    };
    let running = CompareSheet::Running {
        at: 3,
        of: 6,
        name: hidden(2),
        rows: vec![
            run(RunStep::Done { wait_ms: 2_100 }),
            run(RunStep::Failed),
            run(RunStep::Writing),
            run(RunStep::Waiting),
        ],
    };
    let shown = shown(VoiceSheet::Compare(running));
    let parts = [
        "at: 3",
        "of: 6",
        "wait_ms: 2100",
        "Failed",
        "Writing",
        "Waiting",
    ];
    holds(&shown, &parts);
}

#[test]
fn compare_shows_its_ready_models_and_results_with_their_day_and_device() {
    let line = CompareRow {
        name: hidden(2),
        wrong: None,
        words: 0,
        wait_ms: 2_900,
        mb: 900,
        text: Some(hidden(0)),
        tick: false,
    };
    let results = CompareSheet::Results {
        day: day(),
        device: DeviceLabel::Card("RTX 4060".into()),
        rows: vec![line],
    };
    let ready = CompareSheet::Ready {
        models: 6,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
    };
    let shown = [ready, results].map(|s| shown(VoiceSheet::Compare(s)));
    holds(&shown[0], &["models: 6", "Auto(Cpu)", "recording: false"]);
    holds(&shown[1], &["day: Date", "RTX 4060", "wait_ms: 2900"]);
}

#[test]
fn history_carries_its_counts_for_the_header_pin_and_delete_all() {
    let when = When {
        days_ago: 0,
        date: day(),
        hour: 9,
        minute: 41,
    };
    let row = HistoryRow {
        id: 4,
        text: hidden(1),
        rtl: true,
        arabic: true,
        when,
        lang: "ar".into(),
        model: hidden(2),
        pinned: true,
        tick: Some(1),
    };
    let sheet = HistorySheet {
        rows: vec![row],
        total: 12,
        unpinned: 0,
        unpin: true,
        confirm: false,
    };
    let shown = shown(VoiceSheet::History(sheet));
    let parts = ["total: 12", "unpinned: 0", "unpin: true", "pinned: true"];
    holds(&shown, &parts);
    assert!(!shown.contains(SECRETS[1]), "{shown}");
}

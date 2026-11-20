//! The Voice panel's Compare and History pages: each model's step, a failed one too, the ready models and results with
//! their day and device, and History's counts for its header, Pin and Delete all.

use super::panel::{day, holds, service, shown};
use super::{SECRETS, hidden, secret};
use crate::ports::When;
use crate::ports::voice::*;
use crate::{Redacted, Zeroizing};

#[test]
fn comparing_shows_each_models_step_a_failed_one_too() {
    let run = |step| CompareRun {
        name: hidden(2),
        step,
        why: None,
    };
    let refused = CompareRun {
        why: Some(CloudWhy::Key),
        ..run(RunStep::Failed)
    };
    let running = CompareSheet::Running {
        at: 3,
        of: 6,
        name: hidden(2),
        rows: vec![
            run(RunStep::Done { wait_ms: 2_100 }),
            refused,
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
        "why: Some(Key)",
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
        text: Some(secret(0)),
        tick: false,
        cloud: None,
    };
    let held: Option<&Zeroizing<String>> = line.text.as_ref().map(Redacted::expose);
    assert_eq!(held.map(|t| t.as_str()), Some(SECRETS[0]), "wiped on drop");
    let results = CompareSheet::Results {
        day: day(),
        device: DeviceLabel::Card("RTX 4060".into()),
        rows: vec![line],
    };
    let ready = CompareSheet::Ready {
        models: 6,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
        preparing: false,
        cloud: Vec::new(),
        send: Vec::new(),
    };
    let shown = [ready, results].map(|s| shown(VoiceSheet::Compare(s)));
    holds(
        &shown[0],
        &[
            "models: 6",
            "Auto(Cpu)",
            "recording: false",
            "preparing: false",
        ],
    );
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
        text: secret(1),
        rtl: true,
        arabic: true,
        when,
        lang: "ar".into(),
        model: hidden(2),
        pinned: true,
        tick: Some(1),
        cloud: Some(service()),
    };
    let held: &Zeroizing<String> = row.text.expose();
    assert_eq!(held.as_str(), SECRETS[1], "wiped on drop");
    let sheet = HistorySheet {
        rows: vec![row],
        total: 12,
        unpinned: 0,
        unpin: true,
        confirm: false,
    };
    let shown = shown(VoiceSheet::History(sheet));
    let parts = [
        "total: 12",
        "unpinned: 0",
        "unpin: true",
        "pinned: true",
        "Named(\"Service A\")",
    ];
    holds(&shown, &parts);
    assert!(!shown.contains(SECRETS[1]), "{shown}");
}

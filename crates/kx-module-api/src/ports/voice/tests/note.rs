//! A cloud dictation's note on the bar (ADR-0047): why its voice brought no text back or never left, with Try again
//! and "Use the local model", which stays dimmed with Download when no local model is ready.

use super::panel::holds;
use super::*;
use std::time::Duration;

/// A cloud note for `why`, with "Use the local model" as `local` and Try again.
fn cloud(why: CloudWhy, local: LocalUse) -> BarNote {
    BarNote {
        hint: BarHint::Cloud { why, local },
        act: Some(BarButton::Again),
        mic: false,
    }
}

#[test]
fn the_bar_offers_try_again_beside_its_other_buttons() {
    assert!(BarButton::ALL.contains(&BarButton::Again), "a cloud note's");
    assert!(
        PanelButton::ALL.contains(&PanelButton::Again),
        "the panel's"
    );
}

#[test]
fn a_cloud_note_names_why_with_try_again_and_the_local_models_state() {
    let renews_in = Some(Duration::from_secs(3_600));
    let notes = [
        cloud(CloudWhy::Allowance { renews_in }, LocalUse::Ready),
        cloud(CloudWhy::Key, LocalUse::Download { bytes: 740_000_000 }),
        cloud(CloudWhy::Language(english()), LocalUse::Dimmed),
    ];
    let shown = format!("{notes:?}");
    let parts = [
        "Allowance",
        "renews_in: Some(3600s)",
        "Ready",
        "Key",
        "Download { bytes: 740000000 }",
        "iso: \"en\"",
        "Dimmed",
        "Again",
    ];
    holds(&shown, &parts);
}

#[test]
fn every_reason_and_local_state_is_distinct() {
    let whys = [
        CloudWhy::Net,
        CloudWhy::Key,
        CloudWhy::Credit,
        CloudWhy::Allowance { renews_in: None },
        CloudWhy::Rate,
        CloudWhy::TooBig,
        CloudWhy::Language(english()),
        CloudWhy::Empty,
        CloudWhy::Strange,
        CloudWhy::Terms,
        CloudWhy::Off,
        CloudWhy::NoLocal,
        CloudWhy::Fault,
    ];
    let names: HashSet<String> = whys.iter().map(|w| format!("{w:?}")).collect();
    assert_eq!(names.len(), whys.len());
    let locals = [
        LocalUse::Ready,
        LocalUse::Download { bytes: 1 },
        LocalUse::Dimmed,
    ];
    let names: HashSet<String> = locals.iter().map(|l| format!("{l:?}")).collect();
    assert_eq!(names.len(), locals.len());
}

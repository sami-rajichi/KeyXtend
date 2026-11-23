//! The cloud parts of part 1's pages (ADR-0047): the Models page's Local | Cloud sides with the Cloud side's band, a
//! cloud model's row with its service, mark and why it is unavailable, and Compare's cloud models with the send
//! warning.

use super::hidden;
use super::panel::{day, holds, service, shown};
use crate::ports::voice::*;

#[test]
fn the_models_page_has_two_sides_and_the_cloud_side_says_what_it_lists() {
    assert_eq!(ModelSide::ALL, [ModelSide::Local, ModelSide::Cloud]);

    let bands = [CloudBand::Off, CloudBand::NoneOn, CloudBand::Listed];
    assert_eq!(CloudBand::ALL, bands);
    assert_eq!(Unavailable::ALL, [Unavailable::KeepNothing]);
    let clicks = [
        VoiceClick::Side(ModelSide::Cloud),
        VoiceClick::Open(VoiceTab::Cloud),
    ];
    holds(&format!("{clicks:?}"), &["Side(Cloud)", "Open(Cloud)"]);
}

#[test]
fn a_cloud_models_row_names_its_service_mark_and_why_it_is_unavailable() {
    let cloud = CloudRow {
        service: service(),
        mark: ServiceMark::Ready,
        unavailable: Some(Unavailable::KeepNothing),
    };
    let row = ModelRow {
        name: hidden(2),
        bytes: 0,
        default: false,
        state: ModelState::Ready,
        used: false,
        tick: true,
        fetch: None,
        cloud: Some(cloud),
    };
    let sheet = ModelsSheet {
        rows: vec![row],
        best: None,
        untested: false,
        slow: false,
        folder: None,
        offer: None,
        side: ModelSide::Cloud,
        band: Some(CloudBand::Listed),
    };
    let shown = shown(VoiceSheet::Models(sheet));
    let parts = [
        "Named(\"Service A\")",
        "mark: Ready",
        "Some(KeepNothing)",
        "side: Cloud",
        "band: Some(Listed)",
    ];
    holds(&shown, &parts);
}

#[test]
fn compare_lists_its_cloud_models_and_names_each_ticked_service_before_it_sends() {
    let pick = |tick| CompareCloud {
        name: hidden(2),
        service: service(),
        tick,
    };
    let ready = CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Cpu,
        mic: CompareMic::Closed,
        cloud: vec![pick(true), pick(false)],
        send: vec![service()],
    };
    let first = shown(VoiceSheet::Compare(ready));
    holds(
        &first,
        &["tick: true", "tick: false", "send: [Named(\"Service A\")]"],
    );
    let row = CompareRow {
        name: hidden(2),
        wrong: Some(1),
        words: 12,
        wait_ms: 900,
        mb: 0,
        text: None,
        tick: false,
        cloud: Some(service()),
    };
    let results = CompareSheet::Results {
        day: day(),
        device: DeviceLabel::Cpu,
        rows: vec![row],
    };
    let second = shown(VoiceSheet::Compare(results));
    holds(&second, &["mb: 0", "cloud: Some(Named(\"Service A\"))"]);
}

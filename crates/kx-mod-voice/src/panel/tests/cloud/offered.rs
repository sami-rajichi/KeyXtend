//! The rows not offered (ADR-0050): with the entry and key a hand edit or an older settings file left, none shows on
//! the Cloud page or counts at Home, and none of their models lists on the Models page's Cloud side or on Compare, so
//! none can be ticked, picked or sent to.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    CompareSheet, ModelSide, ServiceName, VoiceClick, VoiceSheet, VoiceTab,
};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::super::{models, on_models};
use super::rows::every_row;
use super::{home, on_cloud, put};
use crate::compare::Act;
use crate::services::table;
use crate::services::tests::shipped::{entry, first};
use crate::services::tests::{NOT_OFFERED, OFFERED};

/// Cloud on, with the first offered row and each row not offered switched on and keyed.
fn hand_edited(rig: &mut Rig) {
    let hidden = NOT_OFFERED.map(|id| table().unwrap().get(id).unwrap());
    let rows: Vec<_> = [first()].into_iter().chain(hidden).collect();
    let on: Vec<String> = rows.iter().map(|row| entry(row, "")).collect();
    rig.settings.cloud = true;
    put(rig, "services", &on.join("\n"));
    rig.keyed.extend(rows.iter().map(|row| row.id.clone()));
}

/// The shown name of each row `ids` names.
fn shown(rig: &Rig, ids: &[&str]) -> Vec<ServiceName> {
    let table = rig.services.as_ref().unwrap();
    ids.iter()
        .map(|id| table.get(id).unwrap().shown())
        .collect()
}

#[test]
fn the_cloud_page_lists_only_the_offered_rows_and_home_counts_only_them() {
    let mut rig = on_cloud("");
    hand_edited(&mut rig);
    let names: Vec<ServiceName> = every_row(&mut rig).into_iter().map(|r| r.name).collect();
    assert_eq!(names, shown(&rig, &OFFERED));
    assert_eq!(
        home(&mut rig).cloud,
        Some(1),
        "the first row alone is ready"
    );
}

#[test]
fn the_models_cloud_side_lists_no_model_of_a_row_not_offered() {
    let mut rig = on_models(&[]);
    hand_edited(&mut rig);
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    let view = rig.view().unwrap();
    let services: Vec<ServiceName> = models(&view)
        .rows
        .iter()
        .filter_map(|r| r.cloud.as_ref().map(|c| c.service.clone()))
        .collect();
    let firsts = vec![first().shown(); first().models.len()];
    assert_eq!(services, firsts, "the first row's models alone");
}

#[test]
fn compare_lists_ticks_and_sends_to_no_model_of_a_row_not_offered() {
    let mut rig = Rig::new();
    hand_edited(&mut rig);
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    let n = first().models.len();
    for at in 0..u8::try_from(n).unwrap() {
        rig.click(&VoiceClick::Row(at));
    }
    let VoiceSheet::Compare(CompareSheet::Ready { cloud, send, .. }) = rig.view().unwrap().sheet
    else {
        panic!("Compare is not ready");
    };
    let ticked: Vec<(ServiceName, bool)> = cloud.into_iter().map(|c| (c.service, c.tick)).collect();
    assert_eq!(ticked, vec![(first().shown(), true); n]);
    assert_eq!(send, [first().shown()], "the warning names that row alone");
    let out = rig.click(&VoiceClick::Compare(Redacted::new("hello world".into())));
    assert!(
        matches!(&out[..], [Do::Compare(Act::Open { .. })]),
        "{out:?}"
    );
    let plan = rig.panel.compare.plan().unwrap();
    let to: Vec<&str> = plan.clouds.iter().map(|r| r.service.as_str()).collect();
    assert_eq!(to, vec![first().id.as_str(); n]);
}

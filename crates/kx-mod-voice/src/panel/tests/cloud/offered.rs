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
use super::rows::{every_row, named};
use super::{home, on_cloud, put};
use crate::compare::Act;
use crate::services::tests::{NOT_OFFERED, OFFERED};

/// Cloud on, with Groq and each row not offered switched on and keyed.
fn hand_edited(rig: &mut Rig) {
    let ids = || ["groq"].into_iter().chain(NOT_OFFERED);
    let on: Vec<String> = ids()
        .map(|id| format!("{id} = {{ on = 2026-11-01, terms = 1 }}"))
        .collect();
    rig.settings.cloud = true;
    put(rig, "services", &on.join("\n"));
    rig.keyed.extend(ids().map(String::from));
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
    assert_eq!(home(&mut rig).cloud, Some(1), "Groq alone is ready");
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
    assert_eq!(services, [named("Groq"), named("Groq")], "Groq's two alone");
}

#[test]
fn compare_lists_ticks_and_sends_to_no_model_of_a_row_not_offered() {
    let mut rig = Rig::new();
    hand_edited(&mut rig);
    rig.click(&VoiceClick::Open(VoiceTab::Compare));
    for at in 0..2 {
        rig.click(&VoiceClick::Row(at));
    }
    let VoiceSheet::Compare(CompareSheet::Ready { cloud, send, .. }) = rig.view().unwrap().sheet
    else {
        panic!("Compare is not ready");
    };
    let ticked: Vec<(ServiceName, bool)> = cloud.into_iter().map(|c| (c.service, c.tick)).collect();
    assert_eq!(ticked, [(named("Groq"), true), (named("Groq"), true)]);
    assert_eq!(send, [named("Groq")], "the warning names Groq alone");
    let out = rig.click(&VoiceClick::Compare(Redacted::new("hello world".into())));
    assert!(
        matches!(&out[..], [Do::Compare(Act::Open { .. })]),
        "{out:?}"
    );
    let plan = rig.panel.compare.plan().unwrap();
    let to: Vec<&str> = plan.clouds.iter().map(|r| r.service.as_str()).collect();
    assert_eq!(to, ["groq", "groq"]);
}

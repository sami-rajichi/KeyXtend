//! The Models page opened from Home shows the Cloud side while the language in front has a cloud pick, in use or
//! waiting for its service Off, else the Local side; Download and Back from the folders still open the Local side.

use kx_module_api::ports::{ModelSide, PanelButton, Unavailable, VoiceClick, VoiceTab};

use super::super::super::rig::Rig;
use super::super::models;
use super::{apply, home, put};
use crate::services::tests::shipped::{entry, first, pick};
use crate::services::tests::{ROUTER, with_rows};

/// Cloud on, with the first offered row on and keyed, at Home.
fn groq_on() -> Rig {
    let mut rig = Rig::new();
    rig.settings.cloud = true;
    put(&mut rig, "services", &entry(first(), ""));
    rig.keyed.insert(first().id.clone());
    rig
}

/// As `groq_on`, with that row's first model picked for English.
fn picked() -> Rig {
    let mut rig = groq_on();
    let picks = format!(r#"en = "{}""#, pick(first(), 0));
    put(&mut rig, "picks", &picks);
    rig
}

/// The Models page's side as shown.
fn side(rig: &mut Rig) -> ModelSide {
    models(&rig.view().unwrap()).side
}

/// Models opened from Home: its side, and whether its first row is in use.
fn opened(rig: &mut Rig) -> (ModelSide, bool) {
    home(rig);
    rig.click(&VoiceClick::Open(VoiceTab::Models));
    let view = rig.view().unwrap();
    let sheet = models(&view);
    (sheet.side, sheet.rows.first().is_some_and(|r| r.used))
}

#[test]
fn models_from_home_opens_on_the_cloud_side_while_the_language_has_a_cloud_pick() {
    assert_eq!(opened(&mut groq_on()).0, ModelSide::Local, "no pick");
    let mut rig = picked();
    let used = (ModelSide::Cloud, true);
    assert_eq!(opened(&mut rig), used, "the pick in use");
    rig.settings.services.clear();
    assert_eq!(opened(&mut rig).0, ModelSide::Cloud, "waiting, Off");
    rig.settings.cloud = false;
    assert_eq!(opened(&mut rig).0, ModelSide::Local, "cloud off");
    let mut hidden = picked();
    hidden.hidden = true;
    assert_eq!(opened(&mut hidden).0, ModelSide::Local, "a hidden run");
}

#[test]
fn after_use_on_the_cloud_side_models_opens_there_again() {
    let mut rig = groq_on();
    assert_eq!(opened(&mut rig).0, ModelSide::Local);
    rig.click(&VoiceClick::Side(ModelSide::Cloud));
    rig.click(&VoiceClick::Row(0));
    let out = rig.click(&VoiceClick::Button(PanelButton::Use));
    apply(&mut rig, &out);
    rig.panel.close();
    rig.panel.open();
    assert_eq!(opened(&mut rig), (ModelSide::Cloud, true));
}

#[test]
fn a_pick_the_keep_nothing_switch_rules_out_opens_on_the_cloud_side_with_its_reason() {
    let mut rig = Rig::new();
    rig.services = Some(with_rows(ROUTER));
    rig.settings.cloud = true;
    let router = r#""router-test" = { on = 2026-11-01, terms = 1, keep = true }"#;
    put(&mut rig, "services", router);
    put(&mut rig, "picks", r#"en = "router-test:kept/model""#);
    rig.keyed.insert("router-test".into());
    assert_eq!(opened(&mut rig).0, ModelSide::Cloud);
    let view = rig.view().unwrap();
    let rows = &models(&view).rows;
    let kept = rows.iter().find(|r| r.name.expose() == "Kept");
    let cloud = kept.and_then(|r| r.cloud.as_ref());
    let why = cloud.and_then(|c| c.unavailable);
    assert_eq!(why, Some(Unavailable::KeepNothing));
}

#[test]
fn download_and_back_from_the_folders_open_the_local_side_with_a_cloud_pick() {
    let mut rig = picked();
    rig.with(|p, f| p.download_for("en", f));
    assert_eq!(side(&mut rig), ModelSide::Local, "the bar's Download");
    rig.click(&VoiceClick::Button(PanelButton::Change));
    rig.click(&VoiceClick::Button(PanelButton::Back));
    assert_eq!(side(&mut rig), ModelSide::Local, "Back from the folders");
}

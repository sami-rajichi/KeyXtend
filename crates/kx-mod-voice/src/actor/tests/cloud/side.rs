//! The Models page's side through the actor: after Use on a cloud model, the panel opened again shows Models on the
//! Cloud side, where the language's pick is.

use kx_module_api::ports::{ModelSide, PanelButton, VoiceSheet, VoiceTab};

use super::*;
use crate::actor::Msg;

/// The Models page's side as last drawn; `None` on another page.
fn side(rig: &Rig) -> Option<ModelSide> {
    match rig.views().last()?.panel.clone()?.sheet {
        VoiceSheet::Models(sheet) => Some(sheet.side),
        _ => None,
    }
}

/// Opens the panel on Models, from Home.
fn open_models(rig: &mut Rig) {
    rig.send(Msg::Open);
    rig.act(VoiceClick::Open(VoiceTab::Models));
}

#[test]
fn after_use_on_a_cloud_model_models_opens_on_the_cloud_side_again() {
    let mut rig = Rig::build(|fake, host| {
        groq(fake, host);
        host.put("picks", table(""));
    });
    open_models(&mut rig);
    assert_eq!(side(&rig), Some(ModelSide::Local), "no cloud pick yet");
    rig.act(VoiceClick::Side(ModelSide::Cloud));
    rig.act(VoiceClick::Row(0));
    rig.act(VoiceClick::Button(PanelButton::Use));
    rig.send(Msg::Close);
    open_models(&mut rig);
    assert_eq!(side(&rig), Some(ModelSide::Cloud));
}

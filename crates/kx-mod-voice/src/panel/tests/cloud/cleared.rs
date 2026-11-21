//! An emptied address or Other model line, saved, clears what it kept and sends the picks it served to local ones.

use kx_module_api::ports::{CloudLine, PanelKey, VoiceClick};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::typing::{key, on_custom_model};
use super::{apply, lines_sheet, put};
use crate::settings::{PICKS, SERVICES};

/// Backspaces the line taking the letters from its caret to its start, one for each character before it.
fn empty(rig: &mut Rig) {
    for _ in 0..*lines_sheet(rig).caret.expose() {
        key(rig, &PanelKey::Erase);
    }
}

#[test]
fn saving_an_emptied_address_or_model_line_clears_it_and_the_picks_it_served() {
    let kept = r#", address = "https://a.example.com/v1", model = "my-model""#;
    let mut rig = on_custom_model(kept);
    put(&mut rig, "picks", r#"en = "custom:my-model""#);
    key(&mut rig, &PanelKey::Down);
    empty(&mut rig);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(sheet.model.expose().as_str(), "");
    assert!(
        sheet.saves.contains(&CloudLine::Model),
        "Save stays live to clear it"
    );
    let out = rig.click(&VoiceClick::Save(CloudLine::Model));
    let set = |key| out.iter().any(|d| matches!(d, Do::Set(k, _) if *k == key));
    assert!(set(SERVICES) && set(PICKS), "{out:?}");
    apply(&mut rig, &out);
    assert_eq!(rig.settings.services["custom"].model, "");
    assert!(
        !rig.settings.picks.contains_key("en"),
        "its pick goes local"
    );
    key(&mut rig, &PanelKey::Up);
    empty(&mut rig);
    let out = key(&mut rig, &PanelKey::Enter);
    assert!(matches!(&out[..], [Do::Set(SERVICES, _)]), "{out:?}");
    apply(&mut rig, &out);
    assert_eq!(rig.settings.services["custom"].address, "");
    let sheet = lines_sheet(&mut rig);
    let shown = sheet.address.map(|a| a.expose().clone());
    assert_eq!((shown.as_deref(), sheet.address_fault), (Some(""), None));
}

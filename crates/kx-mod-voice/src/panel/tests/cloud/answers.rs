//! The key page says why, in its own words: a Paste the clipboard could not answer shows Busy on the key's line, not
//! the general fault, and an Other model id the frames refuse stays on its line with the refusal until the page changes.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipText, CloudButton, KeyCheck, PanelButton, VoiceClick};

use super::super::super::Do;
use super::lines::on_groq_lines;
use super::{apply, cloud_sheet, lines_sheet};

#[test]
fn a_busy_clipboard_on_paste_shows_busy_on_the_key_line_and_no_fault_band() {
    let mut rig = on_groq_lines();
    let out = rig.click(&VoiceClick::Service(CloudButton::Paste));
    let [Do::Take(round)] = out[..] else {
        panic!("no take: {out:?}");
    };
    assert!(rig.with(|p, _| p.pasted(round, ClipText::Busy)).is_empty());
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Busy);
    assert_eq!(rig.view().unwrap().note, None, "no general fault band");
}

#[test]
fn a_refused_other_model_stays_on_its_line_with_why_until_the_page_changes() {
    let mut rig = on_groq_lines();
    let typed = |rig: &mut super::super::super::rig::Rig, text: &str| {
        rig.click(&VoiceClick::SetModel(Redacted::new(text.to_owned())))
    };
    let bad = "two words";
    assert!(typed(&mut rig, bad).is_empty(), "the frames refuse it");
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        (sheet.model.expose().as_str(), sheet.model_refused),
        (bad, true)
    );
    let out = typed(&mut rig, "whisper-large-v3");
    apply(&mut rig, &out);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        (sheet.model.expose().as_str(), sheet.model_refused),
        ("whisper-large-v3", false)
    );
    assert!(typed(&mut rig, bad).is_empty());
    rig.click(&VoiceClick::Button(PanelButton::Back));
    if !cloud_sheet(&mut rig).rows[0].tick {
        rig.click(&VoiceClick::Row(0));
    }
    rig.click(&VoiceClick::Service(CloudButton::Lines));
    assert!(
        !lines_sheet(&mut rig).model_refused,
        "a new page forgets it"
    );
}

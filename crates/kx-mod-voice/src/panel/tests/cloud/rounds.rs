//! The key page's rounds (T24 review 2): a Paste on the address or model line replaces only the Paste waited for, so a
//! key test runs on, and Test, Remove or a key Save never drop that Paste; the key line's Paste still starts afresh.

use kx_module_api::ports::{ClipText, CloudButton, CloudLine, KeyCheck, VoiceClick};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::lines::on_groq_lines;
use super::{act, lines_sheet, named, on_cloud, put};
use crate::compare::tests::secret;
use crate::services::tests::DOMAIN;

/// The Other model id pasted in these tests.
const MODEL: &str = "whisper-1";

/// Clicks Test on the shown key page; gives the test's ticket.
fn test(rig: &mut Rig) -> u32 {
    match rig.click(&VoiceClick::Service(CloudButton::Test))[..] {
        [Do::Test { ticket, .. }] => ticket,
        ref out => panic!("no test: {out:?}"),
    }
}

/// Clicks Paste on `line`; gives the take's round.
fn take(rig: &mut Rig, line: CloudLine) -> u32 {
    match rig.click(&VoiceClick::Paste(line))[..] {
        [Do::Take { round, .. }] => round,
        ref out => panic!("no take: {out:?}"),
    }
}

/// The clipboard's answer to the take of `round`: the model id.
fn answer(rig: &mut Rig, round: u32) {
    let text = ClipText::Text(secret(MODEL));
    rig.with(|p, f| p.pasted(round, text, f));
}

#[test]
fn a_model_paste_never_cancels_the_key_test_in_flight() {
    let mut rig = on_groq_lines();
    let ticket = test(&mut rig);
    let round = take(&mut rig, CloudLine::Model);
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Testing, "it runs on");
    answer(&mut rig, round);
    rig.with(|p, _| p.tested(ticket, Ok(()), None));
    let sheet = lines_sheet(&mut rig);
    assert_eq!(sheet.model.expose(), MODEL);
    assert_eq!(sheet.check, KeyCheck::Works, "its answer counts");
}

#[test]
fn test_remove_or_a_key_save_never_drop_a_model_paste_whose_copy_was_taken() {
    let acts: [fn(&mut Rig); 3] = [
        |rig| {
            test(rig);
        },
        |rig| {
            rig.click(&VoiceClick::Service(CloudButton::Remove));
        },
        |rig| {
            rig.click(&VoiceClick::SetKey(secret("kx-new-key")));
        },
    ];
    for (at, act) in acts.into_iter().enumerate() {
        let mut rig = on_groq_lines();
        let round = take(&mut rig, CloudLine::Model);
        act(&mut rig);
        answer(&mut rig, round);
        assert_eq!(lines_sheet(&mut rig).model.expose(), MODEL, "act {at}");
    }
}

#[test]
fn a_key_test_never_drops_an_address_paste_whose_copy_was_taken() {
    let mut rig = on_cloud(DOMAIN);
    let old = "https://kx-old.speech.example.com/stt";
    let entry = format!(r#"domain-test = {{ on = 2026-11-01, terms = 1, address = "{old}" }}"#);
    put(&mut rig, "services", &entry);
    rig.keyed.insert("domain-test".into());
    act(&mut rig, &named("Domain Test"), CloudButton::Lines);
    let round = take(&mut rig, CloudLine::Address);
    test(&mut rig);
    let address = "https://kx-new.speech.example.com/stt";
    rig.with(|p, f| p.pasted(round, ClipText::Text(secret(address)), f));
    let sheet = lines_sheet(&mut rig);
    let shown = sheet.address.as_ref().map(|a| a.expose().as_str());
    assert_eq!(shown, Some(address), "the pasted address reaches its line");
    assert_eq!(sheet.check, KeyCheck::Testing, "and the test runs on");
}

#[test]
fn the_key_lines_paste_still_drops_the_test_in_flight() {
    let mut rig = on_groq_lines();
    let ticket = test(&mut rig);
    take(&mut rig, CloudLine::Key);
    rig.with(|p, _| p.tested(ticket, Ok(()), None));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Untested);
}

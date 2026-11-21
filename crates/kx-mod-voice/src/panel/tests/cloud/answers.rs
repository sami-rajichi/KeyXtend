//! The key page says why, in its own words: a test's refusal names its reason, with a used allowance's renewal, a
//! Paste the clipboard could not answer shows Busy on the key's line, not the general fault, and only until the page
//! changes; an Other model id the frames refuse stays on its line likewise.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipText, CloudButton, CloudFault, KeyCheck, PanelButton, ServiceSheet, VoiceClick,
};
use std::time::Duration;

use super::super::super::Do;
use super::super::super::rig::Rig;
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
    let typed = |rig: &mut Rig, text: &str| {
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
    reopen(&mut rig);
    assert!(
        !lines_sheet(&mut rig).model_refused,
        "a new page forgets it"
    );
}

/// Five hours, a renewal a used-up allowance may tell.
const RENEWS: Option<Duration> = Some(Duration::from_hours(5));

#[test]
fn a_tests_refusal_names_its_reason_and_only_no_answer_points_at_the_network() {
    use KeyCheck::{Account, Allowance, Credit, Failed, Rate, Refused, Region, Strange};
    let cases = [
        (CloudFault::Key, Refused),
        (CloudFault::Credit, Credit),
        (CloudFault::Account, Account),
        (CloudFault::Region, Region),
        (CloudFault::Allowance { renews_in: None }, Allowance),
        (CloudFault::Rate, Rate),
        (CloudFault::Strange, Strange),
        (CloudFault::Network, Failed),
        (CloudFault::Timeout, Failed),
        (CloudFault::Gone, Failed),
        (CloudFault::Range("model"), Failed),
        // A key test sends no sound, so these cannot come; they read as no answer.
        (CloudFault::TooBig, Failed),
        (CloudFault::Language, Failed),
        (CloudFault::Empty, Failed),
    ];
    for (fault, said) in cases {
        let sheet = tested(Err(fault));
        assert_eq!((sheet.check, sheet.renews_at), (said, None), "{fault:?}");
    }
}

#[test]
fn a_used_allowance_on_test_carries_its_renewal_only_while_its_answer_shows() {
    let mut rig = on_groq_lines();
    let ticket = test(&mut rig);
    rig.with(|p, f| {
        p.tested(
            ticket,
            Err(CloudFault::Allowance { renews_in: RENEWS }),
            f.now,
        );
    });
    let sheet = lines_sheet(&mut rig);
    let at = sheet.renews_at.map(|t| (t.hour, t.minute));
    assert_eq!(
        (sheet.check, at),
        (KeyCheck::Allowance, Some((17, 0))),
        "12:00 and five hours"
    );
    let ticket = test(&mut rig);
    assert_eq!(lines_sheet(&mut rig).renews_at, None, "testing again");
    rig.with(|p, f| p.tested(ticket, Ok(()), f.now));
    assert_eq!(lines_sheet(&mut rig).renews_at, None, "the key works");
}

/// Groq's key page after one test of its key answered `answer`.
fn tested(answer: Result<(), CloudFault>) -> ServiceSheet {
    let mut rig = on_groq_lines();
    let ticket = test(&mut rig);
    rig.with(|p, f| p.tested(ticket, answer, f.now));
    lines_sheet(&mut rig)
}

/// Clicks Test on the shown key page; gives the test's ticket.
fn test(rig: &mut Rig) -> u32 {
    let out = rig.click(&VoiceClick::Service(CloudButton::Test));
    let [Do::Test { ticket, .. }] = out[..] else {
        panic!("no test: {out:?}");
    };
    ticket
}

#[test]
fn a_paste_that_kept_nothing_says_so_once_and_leaves_the_tests_answer_standing() {
    for (clip, said) in [
        (ClipText::Busy, KeyCheck::Busy),
        (ClipText::Empty, KeyCheck::Unfit),
    ] {
        let mut rig = on_groq_lines();
        let out = rig.click(&VoiceClick::Service(CloudButton::Test));
        let [Do::Test { ticket, .. }] = out[..] else {
            panic!("no test: {out:?}");
        };
        rig.with(|p, f| p.tested(ticket, Ok(()), f.now));
        let out = rig.click(&VoiceClick::Service(CloudButton::Paste));
        let [Do::Take(round)] = out[..] else {
            panic!("no take: {out:?}");
        };
        rig.with(|p, _| p.pasted(round, clip));
        assert_eq!(lines_sheet(&mut rig).check, said);
        reopen(&mut rig);
        assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Works, "{said:?}");
    }
}

/// Leaves Groq's key page for the Cloud page and opens it again.
fn reopen(rig: &mut Rig) {
    rig.click(&VoiceClick::Button(PanelButton::Back));
    if !cloud_sheet(rig).rows[0].tick {
        rig.click(&VoiceClick::Row(0));
    }
    rig.click(&VoiceClick::Service(CloudButton::Lines));
}

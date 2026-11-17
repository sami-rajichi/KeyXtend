//! A service's key page (design "Keys"): Paste puts the clipboard's text on the key line as dots (edges 23 and carry 9),
//! a test's answer counts only while its page and key stand (carry 3), the link opens only the row's own HTTPS address
//! (edge 23), and a typed address is kept or refused with why (edges 13 and 22, carries 6, 10 and 11).

use kx_module_api::ports::{
    AddressFault, ClipText, CloudButton, CloudFault, KeyCheck, PanelButton, ServiceMark, VoiceClick,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::super::super::rig::Rig;
use super::super::super::{Do, Page};
use super::{act, apply, lines_sheet, on_cloud, put};
use crate::settings::{PICKS, SERVICES};

/// A key-like text, built as the test runs so no key sits in the source.
fn made_key() -> String {
    ["kx", "made", &"7".repeat(24)].join("-")
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// Groq on and keyed, on its key page.
fn on_groq_lines() -> Rig {
    let mut rig = on_cloud("");
    put(
        &mut rig,
        "services",
        "groq = { on = 2026-11-01, terms = 1 }",
    );
    rig.keyed.insert("groq".into());
    act(&mut rig, 0, CloudButton::Lines);
    rig
}

/// Custom on and keyed with `extra` in its entry, on its key page.
fn on_custom_lines(extra: &str) -> Rig {
    let mut rig = on_cloud("");
    put(
        &mut rig,
        "services",
        &format!("custom = {{ on = 2026-11-01, terms = 1{extra} }}"),
    );
    rig.keyed.insert("custom".into());
    act(&mut rig, 1, CloudButton::Lines);
    rig
}

#[test]
fn paste_sets_the_clipboards_text_as_the_key_which_shows_only_as_dots() {
    let mut rig = on_groq_lines();
    let out = rig.click(&VoiceClick::Service(CloudButton::Paste));
    let [Do::Take(round)] = out[..] else {
        panic!("no take: {out:?}");
    };
    let key = made_key();
    let copied = secret(&format!(" {key}\n"));
    let out = rig.with(|p, _| p.pasted(round, ClipText::Text(copied)));
    let [Do::SetKey { service, key: set }] = &out[..] else {
        panic!("no key set: {out:?}");
    };
    assert_eq!(
        (service.as_str(), set.expose().as_str()),
        ("groq", key.as_str())
    );
    assert!(!format!("{out:?}").contains(&key), "the key never prints");
    let dots = u32::try_from(key.len()).unwrap();
    rig.with(|p, _| p.key_set("groq", Ok(dots)));
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        (sheet.key, sheet.check),
        (Redacted::new(dots), KeyCheck::Untested)
    );
    assert!(!format!("{sheet:?}").contains(&key));
    // A Paste's answer after another Paste, or once the page changed, is dropped.
    let out = rig.click(&VoiceClick::Service(CloudButton::Paste));
    let [Do::Take(again)] = out[..] else {
        panic!("no take: {out:?}");
    };
    assert!(
        rig.with(|p, _| p.pasted(round, ClipText::Text(secret(&key))))
            .is_empty()
    );
    let out = rig.with(|p, _| p.pasted(again, ClipText::Empty));
    assert!(out.is_empty());
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Unfit);
}

#[test]
fn a_tests_answer_counts_only_while_its_page_and_key_stand() {
    let mut rig = on_groq_lines();
    let test = |rig: &mut Rig| match rig.click(&VoiceClick::Service(CloudButton::Test))[..] {
        [Do::Test { ticket, .. }] => ticket,
        ref out => panic!("no test: {out:?}"),
    };
    let first = test(&mut rig);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(sheet.check, KeyCheck::Testing);
    assert!(
        !sheet.acts.contains(&CloudButton::Test),
        "one test at a time"
    );
    rig.with(|p, _| p.tested(first, Ok(())));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Works);
    let second = test(&mut rig);
    rig.click(&VoiceClick::SetKey(secret(&made_key())));
    rig.with(|p, _| p.tested(second, Ok(())));
    assert_eq!(
        lines_sheet(&mut rig).check,
        KeyCheck::Untested,
        "the key changed"
    );
    let third = test(&mut rig);
    rig.click(&VoiceClick::Button(PanelButton::Back));
    rig.with(|p, _| p.tested(third, Err(CloudFault::Key)));
    act(&mut rig, 0, CloudButton::Lines);
    assert_eq!(
        lines_sheet(&mut rig).check,
        KeyCheck::Untested,
        "the user left"
    );
    let fourth = test(&mut rig);
    rig.with(|p, _| p.tested(fourth, Err(CloudFault::Key)));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Refused);
    let fifth = test(&mut rig);
    rig.with(|p, _| p.tested(fifth, Err(CloudFault::Gone)));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Failed);
}

#[test]
fn remove_drops_the_key_and_its_dots() {
    let mut rig = on_groq_lines();
    rig.with(|p, _| p.dots("groq", 12));
    assert_eq!(lines_sheet(&mut rig).key, Redacted::new(12));
    let out = rig.click(&VoiceClick::Service(CloudButton::Remove));
    assert_eq!(out, [Do::RemoveKey("groq".into())]);
    assert_eq!(lines_sheet(&mut rig).key, Redacted::new(0));
}

#[test]
fn the_link_opens_only_the_rows_own_https_address() {
    let mut rig = on_groq_lines();
    let out = rig.click(&VoiceClick::Service(CloudButton::Link));
    assert!(matches!(&out[..], [Do::Link(l)] if l.expose() == "https://console.groq.com/keys"));
    rig.click(&VoiceClick::Service(CloudButton::Facts));
    assert_eq!(rig.panel.page, Some(Page::Card));
    let out = rig.click(&VoiceClick::Service(CloudButton::Link));
    assert!(matches!(&out[..], [Do::Link(l)] if l.expose().starts_with("https://")));
    // Custom has no address of its own: neither its card nor its key page offers a link.
    let mut rig = on_custom_lines("");
    assert!(!lines_sheet(&mut rig).acts.contains(&CloudButton::Link));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::NoTest);
}

#[test]
fn a_typed_address_is_kept_when_https_on_a_host_name_and_refused_with_why() {
    let mut rig = on_custom_lines(r#", model = "whisper-1""#);
    assert_eq!(lines_sheet(&mut rig).mark, ServiceMark::NeedsAddress);
    let typed =
        |rig: &mut Rig, text: &str| rig.click(&VoiceClick::SetAddress(Redacted::new(text.into())));
    for (text, fault) in [
        ("http://speech.example.com/v1", AddressFault::Https),
        ("https://10.0.0.1/v1", AddressFault::Host),
    ] {
        assert!(typed(&mut rig, text).is_empty(), "{text} is refused");
        let sheet = lines_sheet(&mut rig);
        assert_eq!(
            (
                sheet.address.unwrap().expose().as_str(),
                sheet.address_fault
            ),
            (text, Some(fault))
        );
    }
    let out = typed(&mut rig, "https://speech.example.com/v1/audio");
    assert!(matches!(&out[..], [Do::Set(SERVICES, _)]));
    apply(&mut rig, &out);
    let sheet = lines_sheet(&mut rig);
    assert_eq!(
        (sheet.mark, sheet.address_fault),
        (ServiceMark::Ready, None)
    );
    put(&mut rig, "picks", r#"en = "custom:whisper-1""#);
    let out = typed(&mut rig, "https://other.example.com/v1/audio");
    assert!(
        matches!(&out[..], [Do::Set(SERVICES, _)]),
        "a changed address keeps the picks"
    );
    apply(&mut rig, &out);
    let out = typed(&mut rig, "");
    assert!(out.iter().any(|d| matches!(d, Do::Set(PICKS, _))));
    apply(&mut rig, &out);
    assert!(
        !rig.settings.picks.contains_key("en"),
        "a cleared address sends the picks to the local default"
    );
}

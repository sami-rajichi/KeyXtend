//! A typed address on a key page is kept or refused with why: no https, no host name, a bad port, a bad path or query,
//! outside the row's domain or too long (edges 13 and 22, carries 6, 10 and 11).

use kx_module_api::Redacted;
use kx_module_api::ports::{AddressFault, CloudButton, HttpsUrl, ServiceMark, VoiceClick};

use super::super::super::Do;
use super::super::super::rig::Rig;
use super::lines::on_custom_lines;
use super::{act, apply, lines_sheet, named, on_cloud, put};
use crate::services::tests::DOMAIN;
use crate::settings::{PICKS, SERVICES};

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

#[test]
fn a_long_address_says_long_and_a_host_in_capitals_is_kept_in_lower_case() {
    let mut rig = on_custom_lines(r#", model = "whisper-1""#);
    let long = format!(
        "https://speech.example.com/{}",
        "a".repeat(HttpsUrl::MAX_BYTES)
    );
    assert!(
        rig.click(&VoiceClick::SetAddress(Redacted::new(long)))
            .is_empty()
    );
    assert_eq!(
        lines_sheet(&mut rig).address_fault,
        Some(AddressFault::Long)
    );
    let typed = Redacted::new("HTTPS://Speech.Example.COM/V1/Audio".into());
    let out = rig.click(&VoiceClick::SetAddress(typed));
    assert!(matches!(&out[..], [Do::Set(SERVICES, _)]), "{out:?}");
    apply(&mut rig, &out);
    let kept = &rig.settings.services["custom"].address;
    assert_eq!(
        kept, "https://speech.example.com/V1/Audio",
        "the path keeps its case"
    );
    assert_eq!(lines_sheet(&mut rig).address_fault, None);
}

#[test]
fn an_address_outside_the_rows_domain_says_domain() {
    let mut rig = on_cloud(DOMAIN);
    put(
        &mut rig,
        "services",
        "domain-test = { on = 2026-11-01, terms = 1 }",
    );
    rig.keyed.insert("domain-test".into());
    act(&mut rig, &named("Domain Test"), CloudButton::Lines);
    let outside = "https://stt.example.org/stt";
    assert!(
        rig.click(&VoiceClick::SetAddress(Redacted::new(outside.into())))
            .is_empty()
    );
    assert_eq!(
        lines_sheet(&mut rig).address_fault,
        Some(AddressFault::Domain)
    );
    let inside = "https://kx-test.speech.example.com/stt";
    let out = rig.click(&VoiceClick::SetAddress(Redacted::new(inside.into())));
    assert!(matches!(&out[..], [Do::Set(SERVICES, _)]), "{out:?}");
}

#[test]
fn a_bad_character_after_a_good_host_says_path_and_a_bad_host_still_says_host() {
    let mut rig = on_custom_lines(r#", model = "whisper-1""#);
    for (text, fault) in [
        ("https://speech.example.com/v1 audio", AddressFault::Path),
        ("https://speech.example.com/v1/a@b", AddressFault::Path),
        ("https://speech.example.com/v1/صوت", AddressFault::Path),
        ("https://user@speech.example.com/v1", AddressFault::Host),
    ] {
        assert!(
            rig.click(&VoiceClick::SetAddress(Redacted::new(text.into())))
                .is_empty()
        );
        assert_eq!(lines_sheet(&mut rig).address_fault, Some(fault), "{text}");
    }
}

#[test]
fn a_bad_port_on_a_good_host_says_port_and_never_host() {
    let mut rig = on_custom_lines(r#", model = "whisper-1""#);
    for text in [
        "https://speech.example.com:0/v1",
        "https://speech.example.com:x/v1",
    ] {
        let out = rig.click(&VoiceClick::SetAddress(Redacted::new(text.into())));
        assert!(out.is_empty(), "{text} is refused");
        let fault = lines_sheet(&mut rig).address_fault;
        assert_eq!(fault, Some(AddressFault::Port), "{text}");
    }
}

//! A service's key page (design "Keys"): Paste puts the clipboard's text on the key line as dots (edges 23 and carry
//! 9), whose caret waits after them until a new key is typed, a test's answer counts only while its page and key stand
//! (carry 3), and the link opens only the row's own HTTPS address (edge 23); a typed address's checks are in `address`.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, ClipText, CloudButton, CloudFault, CloudLine, HttpsUrl, KeyCheck, MAX_KEY_BYTES,
    PanelButton, PanelKey, PasteFault, ServiceName, TAG_LEN, VoiceClick,
};

use super::super::super::rig::Rig;
use super::super::super::{Do, Page};
use super::typing::key;
use super::{act, apply, lines_sheet, on_cloud, put, shown};
use crate::compare::tests::secret;
use crate::services::table;
use crate::services::tests::shipped::{entry, first};

/// An address the Custom row takes.
const ADDRESS: &str = "https://speech.example.com/v1/audio";

/// A key-like text, built as the test runs so no key sits in the source.
fn made_key() -> String {
    ["kx", "made", &"7".repeat(24)].join("-")
}

/// Groq, the first offered row, on and keyed, on its key page.
pub(super) fn on_groq_lines() -> Rig {
    let mut rig = on_cloud("");
    put(&mut rig, "services", &entry(first(), ""));
    rig.keyed.insert(first().id.clone());
    act(&mut rig, &first().shown(), CloudButton::Lines);
    rig
}

/// Custom on and keyed with `extra` in its entry, on its key page.
pub(super) fn on_custom_lines(extra: &str) -> Rig {
    let mut rig = on_cloud("");
    put(
        &mut rig,
        "services",
        &format!("custom = {{ on = 2026-11-01, terms = 1{extra} }}"),
    );
    rig.keyed.insert("custom".into());
    act(&mut rig, &ServiceName::Custom, CloudButton::Lines);
    rig
}

/// Paste on `line`, answered with `clip`; the answer's own acts.
fn paste(rig: &mut Rig, line: CloudLine, clip: ClipText) -> Vec<Do> {
    let out = rig.click(&VoiceClick::Paste(line));
    let [Do::Take { round, .. }] = out[..] else {
        panic!("no take: {out:?}");
    };
    rig.with(|p, f| p.pasted(round, clip, f))
}

#[test]
fn paste_sets_the_clipboards_text_as_the_key_which_shows_only_as_dots() {
    let mut rig = on_groq_lines();
    let out = rig.click(&VoiceClick::Paste(CloudLine::Key));
    let [Do::Take { round, max }] = out[..] else {
        panic!("no take: {out:?}");
    };
    assert_eq!(max, MAX_KEY_BYTES, "a key's most");
    let key = made_key();
    let copied = secret(&format!(" {key}\n"));
    let out = rig.with(|p, f| p.pasted(round, ClipText::Text(copied), f));
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
    let out = rig.click(&VoiceClick::Paste(CloudLine::Key));
    let [Do::Take { round: again, .. }] = out[..] else {
        panic!("no take: {out:?}");
    };
    assert!(
        rig.with(|p, f| p.pasted(round, ClipText::Text(secret(&key)), f))
            .is_empty()
    );
    let out = rig.with(|p, f| p.pasted(again, ClipText::Empty, f));
    assert!(out.is_empty());
    let sheet = lines_sheet(&mut rig);
    let empty = Some((CloudLine::Key, PasteFault::Empty));
    assert_eq!(sheet.pasted, empty, "its note");
    assert_eq!(sheet.check, KeyCheck::Untested, "in the answer's place");
}

#[test]
fn paste_on_the_address_line_puts_the_clipboards_text_there_ready_for_save() {
    let mut rig = on_custom_lines("");
    let out = rig.click(&VoiceClick::Paste(CloudLine::Address));
    let [Do::Take { round, max }] = out[..] else {
        panic!("no take: {out:?}");
    };
    assert_eq!(max, HttpsUrl::MAX_BYTES, "an address's most");
    let copied = secret(&format!(" {ADDRESS}\n"));
    let out = rig.with(|p, f| p.pasted(round, ClipText::Text(copied), f));
    assert!(out.is_empty(), "nothing kept before Save");
    let sheet = lines_sheet(&mut rig);
    let address = sheet.address.as_ref().map(|a| a.expose().as_str());
    assert_eq!(address, Some(ADDRESS), "the text, trimmed");
    let ready = (sheet.typing, sheet.saves.contains(&CloudLine::Address));
    assert_eq!(ready, (CloudLine::Address, true), "it takes the letters");
    let out = rig.click(&VoiceClick::Save(CloudLine::Address));
    assert!(!out.is_empty(), "Save keeps it");
    let model = ClipText::Text(secret("whisper-1"));
    assert!(paste(&mut rig, CloudLine::Model, model).is_empty());
    assert_eq!(lines_sheet(&mut rig).model.expose(), "whisper-1");
}

#[test]
fn an_empty_or_busy_paste_says_so_under_its_line_and_a_too_long_one_keeps_its_refusal() {
    let mut rig = on_custom_lines(&format!(r#", address = "{ADDRESS}""#));
    for line in [CloudLine::Address, CloudLine::Model, CloudLine::Key] {
        paste(&mut rig, line, ClipText::Empty);
        let sheet = lines_sheet(&mut rig);
        assert_eq!(sheet.pasted, Some((line, PasteFault::Empty)), "{line:?}");
        assert_eq!(sheet.address_fault, None, "{line:?}: no address refusal");
        let address = sheet.address.as_ref().map(|a| a.expose().as_str());
        assert_eq!(address, Some(ADDRESS), "the line keeps its text");
    }
    let blank = paste(&mut rig, CloudLine::Key, ClipText::Text(secret(" \n")));
    assert!(blank.is_empty(), "spaces set no key");
    let empty = Some((CloudLine::Key, PasteFault::Empty));
    assert_eq!(lines_sheet(&mut rig).pasted, empty, "spaces are no text");
    paste(&mut rig, CloudLine::Model, ClipText::Busy);
    let busy = Some((CloudLine::Model, PasteFault::Busy));
    assert_eq!(lines_sheet(&mut rig).pasted, busy, "busy on the model line");
    paste(&mut rig, CloudLine::Address, ClipText::TooLong);
    let sheet = lines_sheet(&mut rig);
    let long = (sheet.address_fault, sheet.pasted);
    assert_eq!(long, (Some(AddressFault::Long), None));
    let long = "m".repeat(*TAG_LEN.end() + 1);
    paste(&mut rig, CloudLine::Model, ClipText::Text(secret(&long)));
    assert!(lines_sheet(&mut rig).model_refused, "past the line's most");
    paste(&mut rig, CloudLine::Key, ClipText::TooLong);
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
    rig.with(|p, _| p.tested(first, Ok(()), None));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Works);
    let second = test(&mut rig);
    rig.click(&VoiceClick::SetKey(secret(&made_key())));
    rig.with(|p, _| p.tested(second, Ok(()), None));
    assert_eq!(
        lines_sheet(&mut rig).check,
        KeyCheck::Untested,
        "the key changed"
    );
    let third = test(&mut rig);
    rig.click(&VoiceClick::Button(PanelButton::Back));
    rig.with(|p, _| p.tested(third, Err(CloudFault::Key), None));
    act(&mut rig, &shown("groq"), CloudButton::Lines);
    assert_eq!(
        lines_sheet(&mut rig).check,
        KeyCheck::Untested,
        "the user left"
    );
    let fourth = test(&mut rig);
    rig.with(|p, _| p.tested(fourth, Err(CloudFault::Key), None));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Refused);
    let fifth = test(&mut rig);
    rig.with(|p, _| p.tested(fifth, Err(CloudFault::Gone), None));
    assert_eq!(lines_sheet(&mut rig).check, KeyCheck::Failed);
}

#[test]
fn with_a_sealed_key_and_nothing_typed_the_caret_waits_after_its_dots() {
    let mut rig = on_groq_lines();
    rig.with(|p, _| p.dots("groq", 12));
    let shown = |rig: &mut Rig| {
        let s = lines_sheet(rig);
        (
            *s.key.expose(),
            *s.typed.expose(),
            s.typing,
            *s.caret.expose(),
        )
    };
    let sealed = (12, 0, CloudLine::Key, 12);
    assert_eq!(shown(&mut rig), sealed);
    key(&mut rig, &PanelKey::Erase);
    rig.click(&VoiceClick::Caret {
        line: CloudLine::Key,
        at: Redacted::new(12),
    });
    assert_eq!(shown(&mut rig), sealed, "Backspace and a click leave it");
    key(&mut rig, &PanelKey::Text(Redacted::new("a".into())));
    let new = (12, 1, CloudLine::Key, 1);
    assert_eq!(shown(&mut rig), new, "a new key of one letter");
    key(&mut rig, &PanelKey::Erase);
    assert_eq!(shown(&mut rig), sealed);
}

#[test]
fn a_row_with_no_test_call_shows_no_test_and_its_note_while_keyed() {
    let mut rig = on_cloud("");
    let lemon = table().unwrap().get("lemonfox").unwrap();
    put(&mut rig, "services", &entry(lemon, ""));
    rig.keyed.insert(lemon.id.clone());
    act(&mut rig, &lemon.shown(), CloudButton::Lines);
    let sheet = lines_sheet(&mut rig);
    let test = (sheet.tests, sheet.acts.contains(&CloudButton::Test));
    assert_eq!(test, (false, false), "no Test at all");
    assert_eq!(sheet.check, KeyCheck::NoTest, "its note");
    assert!(lines_sheet(&mut on_groq_lines()).tests, "Groq has a test");
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
    let keys = first().keys.as_ref().unwrap().expose();
    assert!(matches!(&out[..], [Do::Link(l)] if l.expose() == keys));
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
fn edge_17_the_other_model_line_says_why_a_kept_models_id_cannot_go_while_keep_nothing_is_on() {
    let mut rig = on_cloud("");
    let router = table().unwrap().get("openrouter").unwrap();
    let kept = router.models.iter().find(|m| m.keeps).unwrap();
    let typed = format!(r#", keep = true, model = "{}""#, kept.id);
    put(&mut rig, "services", &entry(router, &typed));
    act(&mut rig, &router.shown(), CloudButton::Lines);
    let sheet = lines_sheet(&mut rig);
    assert_eq!((sheet.keep, sheet.model_na), (Some(true), true));
    let out = rig.click(&VoiceClick::KeepNothing(false));
    apply(&mut rig, &out);
    let sheet = lines_sheet(&mut rig);
    assert_eq!((sheet.keep, sheet.model_na), (Some(false), false));
}

#[test]
fn back_or_no_on_a_card_opened_from_the_key_page_goes_back_to_that_page() {
    let mut rig = on_groq_lines();
    rig.click(&VoiceClick::Service(CloudButton::Facts));
    assert_eq!(rig.panel.page, Some(Page::Card));
    rig.click(&VoiceClick::Button(PanelButton::Back));
    assert_eq!(rig.panel.page, Some(Page::Lines), "Back");
    rig.click(&VoiceClick::Service(CloudButton::Facts));
    rig.settings.services.clear();
    rig.click(&VoiceClick::Confirm(false));
    assert_eq!(
        rig.panel.page,
        Some(Page::Lines),
        "No on a service still off"
    );
}

//! The Qt Quick tests' key pages: Groq's with its key tested, `OpenRouter`'s, the Custom row's, a typed key,
//! `AssemblyAI`'s, the answers that draw a warning band, and each part with every line and the longest answer in each
//! language.

use super::cloud::named;
use super::{Shown, paged, shown};
use crate::lang;
use crate::qml::LANGS;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CloudButton, CloudLine, KeyCheck, LinesPart, LocalTime, Renewal, ServiceMark,
    ServiceName, ServiceSheet, VoiceSheet,
};

/// A renewal at 23:59 UTC, as a used allowance's answer shows it.
const RENEWS_MS: i64 = 1_794_700_740_000;
/// The sealed key's length, after whose dots the caret waits while nothing is typed.
const DOTS: u32 = 56;
/// The answers a test draws as a warning band, by the page's name.
const BANDS: [(&str, KeyCheck); 3] = [
    ("serviceAllowance", KeyCheck::Allowance),
    ("serviceRate", KeyCheck::Rate),
    ("serviceStrange", KeyCheck::Strange),
];

/// Groq's key page with its key tested, the keep-nothing row's with a refused model, or the Custom row's with an
/// address that is not HTTPS taking the letters.
pub(super) fn lines(which: &str) -> VoiceSheet {
    let text = |s: &str| Redacted::new(s.to_owned());
    let mut s = ServiceSheet {
        service: named("Groq"),
        mark: ServiceMark::Ready,
        key: Redacted::new(DOTS),
        typed: Redacted::new(0),
        check: KeyCheck::Works,
        renews_at: None,
        address: None,
        address_fault: None,
        model: text(""),
        model_na: false,
        model_refused: false,
        keep: None,
        site: Some("console.groq.com".to_owned()),
        tests: true,
        acts: CloudButton::ALL.to_vec(),
        part: None,
        typing: CloudLine::Key,
        caret: Redacted::new(DOTS),
        saves: Vec::new(),
    };
    if which == "router" {
        (s.service, s.mark, s.check) = (named("OpenRouter"), ServiceMark::Ready, KeyCheck::NoTest);
        (s.model, s.model_na, s.keep) = (text("mistralai/voxtral-small"), true, Some(true));
        s.saves.push(CloudLine::Model);
    } else if which == "custom" {
        (s.service, s.mark) = (ServiceName::Custom, ServiceMark::NeedsKey);
        (s.key, s.check) = (Redacted::new(0), KeyCheck::Untested);
        (s.address, s.address_fault) =
            (Some(text("http://lan.example")), Some(AddressFault::Https));
        (s.model, s.model_refused, s.site, s.tests) = (text("no such/model"), true, None, false);
        s.acts = vec![CloudButton::Paste, CloudButton::Facts, CloudButton::Off];
        (s.typing, s.caret) = (CloudLine::Address, Redacted::new(7));
        s.saves = vec![CloudLine::Address, CloudLine::Model];
    }
    VoiceSheet::Service(s)
}

/// Groq's key page as `edit` changes it.
fn groq(edit: impl FnOnce(&mut ServiceSheet)) -> VoiceSheet {
    let VoiceSheet::Service(mut s) = lines("groq") else {
        unreachable!("a key page")
    };
    edit(&mut s);
    VoiceSheet::Service(s)
}

/// The renewal a used allowance's answer shows, on a later day, so its line takes its longest form.
fn renews() -> Option<Renewal> {
    LocalTime::at(RENEWS_MS, 0).map(|at| Renewal { at, today: false })
}

/// The key test's answer with the longest line in language `tag`, with the renewal it shows.
fn longest_check(tag: &str) -> (KeyCheck, Option<Renewal>) {
    let t = lang(tag);
    let len = |c: &KeyCheck| {
        t.voice_key_check(*c, renews())
            .map_or(0, |a| a.chars().count())
    };
    let check = KeyCheck::ALL
        .into_iter()
        .max_by_key(len)
        .unwrap_or(KeyCheck::Failed);
    (check, (check == KeyCheck::Allowance).then(renews).flatten())
}

/// Part `part` of the Custom row's key page in language `tag` with every line, each with its longest answer.
fn service_long(part: LinesPart, tag: &str) -> VoiceSheet {
    let VoiceSheet::Service(mut s) = lines("custom") else {
        unreachable!("a key page")
    };
    (s.key, (s.check, s.renews_at)) = (Redacted::new(DOTS), longest_check(tag));
    (s.address_fault, s.keep) = (Some(AddressFault::Path), Some(true));
    (s.site, s.part) = (Some("console.groq.com".to_owned()), Some(part));
    VoiceSheet::Service(s)
}

/// The key pages in language `tag`, by name; a long one shows its first part on page 1 of 2, its second on page 2.
pub(super) fn panels(tag: &str) -> Vec<(String, Shown)> {
    let two = |sheet, at| paged(shown(sheet, &[]), at, 2, 0);
    let typed = groq(|s| {
        (s.typed, s.caret, s.saves) = (Redacted::new(20), Redacted::new(12), vec![CloudLine::Key]);
    });
    let assembly = groq(|s| {
        (s.service, s.check, s.tests) = (named("AssemblyAI"), KeyCheck::NoTest, false);
        s.site = Some("www.assemblyai.com".to_owned());
    });
    let mut out = vec![
        ("service", shown(lines("groq"), &[])),
        ("serviceRouter", shown(lines("router"), &[])),
        ("serviceAssembly", shown(assembly, &[])),
        ("serviceCustom", shown(lines("custom"), &[])),
        ("serviceTyped", shown(typed, &[])),
        ("serviceLong", two(service_long(LinesPart::Key, tag), 0)),
        (
            "serviceLongModel",
            two(service_long(LinesPart::Model, tag), 1),
        ),
    ];
    for (name, check) in BANDS {
        let renews_at = (check == KeyCheck::Allowance).then(renews).flatten();
        let sheet = groq(|s| (s.check, s.renews_at) = (check, renews_at));
        out.push((name, shown(sheet, &[])));
    }
    out.into_iter().map(|(n, s)| (n.to_owned(), s)).collect()
}

/// The long key page shows the key test's longest answer in each language, so the fit test measures it.
#[test]
fn the_long_key_page_shows_the_longest_key_answer_in_each_language() {
    for tag in LANGS {
        let t = lang(tag);
        let VoiceSheet::Service(s) = service_long(LinesPart::Key, tag) else {
            unreachable!("a key page")
        };
        let len = |c, r| {
            t.voice_key_check(c, r)
                .map_or(0, |a: String| a.chars().count())
        };
        let most = KeyCheck::ALL.into_iter().map(|c| len(c, renews())).max();
        assert_eq!(Some(len(s.check, s.renews_at)), most, "{tag}");
    }
}

//! Cloud services on the bar and the panel (ADR-0047): the cloud page after Models, the marks and buttons, the acts
//! that carry a key, an address or a model id and never print them, a card's facts as ids, numbers and dates, the key
//! page's dots and the bar's cloud mark.

use super::panel::{day, holds, service, shown};
use super::{hidden, keeps_secrets, page, secret};
use crate::ports::Hundredths;
use crate::ports::voice::*;
use crate::{REDACTED_MARKER, Redacted};

/// A fact of id `id` with `args`.
fn fact(id: &str, args: Vec<FactArg>) -> Fact {
    Fact {
        id: id.into(),
        args,
        warn: false,
    }
}

/// A named value.
fn arg(name: &str, value: FactValue) -> FactArg {
    FactArg {
        name: name.into(),
        value,
    }
}

/// Dollars in hundredths.
fn usd(cents: u32) -> FactValue {
    FactValue::Money {
        amount: Hundredths(cents),
        currency: "USD".into(),
    }
}

/// A made-up free allowance: 8 hours a day, 20 requests a minute.
fn free() -> Fact {
    let counts = vec![
        arg("hours", FactValue::Count(8)),
        arg("requests", FactValue::Count(20)),
    ];
    fact("day", counts)
}

#[test]
fn the_cloud_page_follows_models_in_the_home_list() {
    use VoiceTab::{Cloud, Compare, Device, History, Models, Options};
    let want = [Models, Cloud, Device, Compare, History, Options];
    assert_eq!(VoiceTab::ALL, want, "the mock-up's second row");
}

#[test]
fn a_key_tests_answers_are_the_designs_with_a_refusals_own_reasons() {
    use KeyCheck::{
        Account, Allowance, Busy, Credit, Failed, NoTest, Rate, Refused, Region, Strange, Testing,
        Unfit, Untested, Works,
    };
    let checks = [
        Untested, Testing, Works, Refused, Failed, Credit, Account, Region, Allowance, Rate,
        Strange, Unfit, NoTest, Busy,
    ];
    assert_eq!(KeyCheck::ALL, checks);
}

#[test]
fn the_marks_and_buttons_are_the_designs() {
    use CloudButton::{Facts, Lines, Link, Paste, Remove, Test};
    use ServiceMark::{NeedsAddress, NeedsKey, Off, Ready};
    assert_eq!(
        ServiceMark::ALL,
        [Off, NeedsKey, NeedsAddress, Ready],
        "a model's Unavailable is its row's"
    );
    let want = [
        CloudButton::On,
        CloudButton::Off,
        Facts,
        Lines,
        Paste,
        Test,
        Remove,
        Link,
    ];
    assert_eq!(CloudButton::ALL, want);

    let faults = [
        AddressFault::Https,
        AddressFault::Host,
        AddressFault::Port,
        AddressFault::Domain,
        AddressFault::Long,
        AddressFault::Path,
    ];
    assert_eq!(AddressFault::ALL, faults);
}

#[test]
fn an_act_that_carries_a_key_an_address_or_a_model_never_prints_it() {
    let clicks = [
        VoiceClick::SetKey(secret(0)),
        VoiceClick::SetAddress(hidden(1)),
        VoiceClick::SetModel(hidden(2)),
    ];
    let acts = clicks.map(|click| VoiceAct { round: 4, click });
    let shown = format!("{acts:?}{acts:#?}");
    assert!(keeps_secrets(&shown), "{shown}");
    for part in [
        "round: 4",
        "SetKey",
        "SetAddress",
        "SetModel",
        REDACTED_MARKER,
    ] {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn the_switches_buttons_and_the_local_model_are_plain_clicks() {
    let clicks = [
        VoiceClick::Cloud(true),
        VoiceClick::KeepNothing(false),
        VoiceClick::Service(CloudButton::Test),
        VoiceClick::Local,
        VoiceClick::Confirm(true),
    ];
    let shown = format!("{clicks:?}");
    let parts = [
        "Cloud(true)",
        "KeepNothing(false)",
        "Service(Test)",
        "Local",
        "Confirm(true)",
    ];
    holds(&shown, &parts);
}

#[test]
fn the_cloud_page_lists_services_with_their_marks_free_allowance_and_warning() {
    let row = |name, mark, warn: bool| ServiceRow {
        name,
        mark,
        untested: warn,
        free: Some(free()),
        warn: warn.then(|| "trains".to_owned()),
        tick: false,
    };
    let rows = vec![
        row(service(), ServiceMark::Ready, false),
        row(
            ServiceName::Named("Service B".into()),
            ServiceMark::Off,
            true,
        ),
        ServiceRow {
            free: None,
            ..row(ServiceName::Custom, ServiceMark::NeedsKey, false)
        },
    ];
    let acts = vec![CloudButton::Facts, CloudButton::On];
    let shown = shown(VoiceSheet::Cloud(CloudSheet {
        on: true,
        rows,
        acts,
    }));
    let parts = [
        "Service A",
        "Ready",
        "untested: true",
        "warn: Some(\"trains\")",
        "Custom",
        "Count(8)",
        "free: None",
        "On]",
    ];
    holds(&shown, &parts);
}

/// A made-up card, its price part on show, with the training warning and old facts.
fn card() -> CloudCard {
    let price = |hours, cents| UsePrice {
        hours,
        price: fact("one", vec![arg("amount", usd(cents))]),
    };
    let switch = FactValue::Text("Made-up switch".into());
    CloudCard {
        service: ServiceName::Named("Service B".into()),
        place: fact("us", Vec::new()),
        trains: Fact {
            warn: true,
            ..fact("by-default", Vec::new())
        },
        keeps: fact("unsaid", Vec::new()),
        free: free(),
        light: price(15, 330),
        heavy: price(60, 1_320),
        bills: Some(fact("least", vec![arg("seconds", FactValue::Count(10))])),
        rule: Some(fact("non-commercial", Vec::new())),
        read: day(),
        old: true,
        site: Some("example.com".into()),
        warn: Some(fact("trains", vec![arg("sw", switch)])),
        untested: true,
        on: None,
        part: CardPart::Price,
    }
}

#[test]
fn a_card_holds_its_facts_as_ids_numbers_and_dates_with_its_least_bill_warning_and_age() {
    let shown = shown(VoiceSheet::Card(Box::new(card())));
    let parts = [
        "by-default",
        "warn: true",
        "hours: 15",
        "Hundredths(1320)",
        "\"USD\"",
        "day: 12",
        "old: true",
        "example.com",
        "Text(\"Made-up switch\")",
        "Count(10)",
        "non-commercial",
        "untested: true",
        "on: None",
        "part: Price",
    ];
    holds(&shown, &parts);
}

#[test]
fn the_key_page_shows_the_key_as_dots_and_never_the_typed_lines() {
    let sheet = ServiceSheet {
        service: ServiceName::Custom,
        mark: ServiceMark::Ready,
        key: Redacted::new(56),
        typed: Redacted::new(0),
        check: KeyCheck::Untested,
        renews_at: None,
        address: Some(hidden(1)),
        address_fault: Some(AddressFault::Host),
        model: hidden(2),
        model_na: true,
        model_refused: true,
        keep: Some(true),
        site: None,
        tests: true,
        acts: vec![CloudButton::Paste, CloudButton::Test],
        part: Some(LinesPart::Model),
        typing: CloudLine::Address,
        caret: Redacted::new(0),
        saves: vec![CloudLine::Address],
    };
    let shown = shown(VoiceSheet::Service(sheet));
    assert!(!shown.contains("56"), "a key's length is a secret: {shown}");
    holds(
        &shown,
        &[
            "Custom",
            "Ready",
            "Untested",
            "address_fault: Some(Host)",
            "model_na: true",
            "model_refused: true",
            "keep: Some(true)",
            "part: Some(Model)",
            "Paste",
        ],
    );
}

#[test]
fn the_bar_names_the_service_its_voice_goes_to() {
    let page_a = VoicePage {
        cloud: Some(service()),
        ..page(VoiceBar::Transcribing)
    };
    let shown = format!("{:?}", VoiceView::new(page_a));
    holds(
        &shown,
        &["cloud: Some(Named(\"Service A\"))", "Transcribing"],
    );
    let local = format!("{:?}", page(VoiceBar::Hidden));
    assert!(local.contains("cloud: None"), "{local}");
}

//! The cloud panels with their longest content, which the fit test measures: the Cloud page and the Models page's Cloud
//! side with the services table's rows, each part of Groq's card with every line on, each part of a key page with every
//! line and the longest answers, and Compare with every cloud model of the services in the table.

use super::cloud::{card, lines, named};
use super::{Shown, paged, shown};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CardPart, CloudBand, CloudButton, CloudRow, CloudSheet, CompareCloud,
    CompareSheet, DeviceLabel, DeviceName, KeyCheck, LinesPart, ModelRow, ModelSide, ModelState,
    ModelsSheet, PanelButton, ServiceMark, ServiceName, ServiceRow, Unavailable, VoiceSheet,
};

/// The Cloud page with the table's rows, both untested, Groq ticked and Custom waiting for its address.
fn cloud_long() -> VoiceSheet {
    let row = |name, mark, free, tick| ServiceRow {
        name,
        mark,
        untested: true,
        free,
        warn: false,
        tick,
    };
    let VoiceSheet::Card(groq) = card(false) else {
        unreachable!("a card")
    };
    let rows = vec![
        row(named("Groq"), ServiceMark::Ready, Some(groq.free), true),
        row(ServiceName::Custom, ServiceMark::NeedsAddress, None, false),
    ];
    let acts = vec![CloudButton::Facts, CloudButton::Lines, CloudButton::Off];
    VoiceSheet::Cloud(CloudSheet {
        on: true,
        rows,
        acts,
    })
}

/// A cloud model `name` of `service` at `mark`, maybe one that keeps nothing it cannot promise.
fn model(name: &str, service: ServiceName, mark: ServiceMark, na: bool) -> ModelRow {
    ModelRow {
        name: Redacted::new(name.to_owned()),
        bytes: 0,
        default: false,
        state: ModelState::Ready,
        used: false,
        tick: false,
        fetch: None,
        cloud: Some(CloudRow {
            service,
            mark,
            unavailable: na.then_some(Unavailable::KeepNothing),
        }),
    }
}

/// The Models page's Cloud side with Groq's models, a typed Custom model and one no keep-nothing host serves.
fn models_long() -> VoiceSheet {
    let mut used = model(
        "Whisper Large v3 Turbo",
        named("Groq"),
        ServiceMark::Ready,
        false,
    );
    (used.used, used.tick) = (true, true);
    let rows = vec![
        used,
        model("Whisper Large v3", named("Groq"), ServiceMark::Ready, false),
        model(
            "whisper-large-v3-turbo",
            ServiceName::Custom,
            ServiceMark::NeedsAddress,
            false,
        ),
        model(
            "voxtral-small",
            named("OpenRouter"),
            ServiceMark::Ready,
            true,
        ),
    ];
    VoiceSheet::Models(ModelsSheet {
        rows,
        best: None,
        untested: false,
        slow: false,
        folder: None,
        offer: None,
        side: ModelSide::Cloud,
        band: Some(CloudBand::Listed),
    })
}

/// Part `part` of Groq's card asking again, old, untested and with the training warning.
fn card_long(part: CardPart) -> VoiceSheet {
    let VoiceSheet::Card(mut c) = card(true) else {
        unreachable!("a card")
    };
    (c.untested, c.on, c.part) = (true, None, part);
    VoiceSheet::Card(c)
}

/// Part `part` of the Custom row's key page with every line, each with its longest answer.
fn service_long(part: LinesPart) -> VoiceSheet {
    let VoiceSheet::Service(mut s) = lines("custom") else {
        unreachable!("a key page")
    };
    (s.key, s.check) = (Redacted::new(56), KeyCheck::Failed);
    (s.address_fault, s.keep) = (Some(AddressFault::Path), Some(true));
    (s.site, s.part) = (Some("console.groq.com".to_owned()), Some(part));
    VoiceSheet::Service(s)
}

/// Compare's set text with every cloud model of Groq and Custom ticked.
fn compare_long() -> VoiceSheet {
    let pick = |name: &str, service| CompareCloud {
        name: Redacted::new(name.to_owned()),
        service,
        tick: true,
    };
    VoiceSheet::Compare(CompareSheet::Ready {
        models: 2,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        recording: false,
        preparing: false,
        cloud: vec![
            pick("Whisper Large v3 Turbo", named("Groq")),
            pick("Whisper Large v3", named("Groq")),
            pick("whisper-large-v3-turbo", ServiceName::Custom),
        ],
        send: vec![named("Groq"), ServiceName::Custom],
    })
}

/// The longest cloud panels, by name; a card and a key page show their first part on page 1 of 2, and their second
/// on page 2.
pub fn panels() -> Vec<(&'static str, Shown)> {
    let two = |sheet, at| paged(shown(sheet, &[]), at, 2, 0);
    vec![
        ("cloudLong", paged(shown(cloud_long(), &[]), 0, 1, 1)),
        (
            "modelsLong",
            paged(shown(models_long(), &[PanelButton::Use]), 0, 1, 1),
        ),
        ("cardLong", two(card_long(CardPart::Voice), 0)),
        ("cardLongPrice", two(card_long(CardPart::Price), 1)),
        ("serviceLong", two(service_long(LinesPart::Key), 0)),
        ("serviceLongModel", two(service_long(LinesPart::Model), 1)),
        ("compareLong", shown(compare_long(), &[])),
    ]
}

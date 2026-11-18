//! The cloud panels with their longest content, which the fit test measures: each Cloud page of the services table's
//! rows, page by page, the Models page's Cloud side, each part of each table service's card with every line on, each
//! part of a key page with every line and the longest answers, and Compare with each table service's cloud models.

use super::cloud::{TRAINS, card, lines, named};
use super::{Shown, paged, shown};
use kx_mod_voice::VoiceSettings;
use kx_mod_voice::cloud::Cloud;
use kx_mod_voice::services::{Address, Service, Services, table};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    AddressFault, CardPart, CloudBand, CloudButton, CloudRow, CloudSheet, CompareCloud,
    CompareSheet, DeviceLabel, DeviceName, Fact, KeyCheck, LinesPart, ModelRow, ModelSide,
    ModelState, ModelsSheet, PanelButton, ServiceMark, ServiceName, ServiceRow, Unavailable,
    VoiceSheet,
};
use kx_ui::UiSettings;

/// The start of each long Cloud page's name, then its page number from 1.
const CLOUD_LONG: &str = "cloudLong-";
/// The start of a service's long card's name, then its id: its first part, and its second.
const CARD_LONG: &str = "cardLong-";
const PRICE_LONG: &str = "cardLongPrice-";

/// The voice module's services table, which the long panels show whole.
fn services() -> &'static Services {
    table().expect("the services table reads")
}

/// The warning a row of the table shows: its card's own, as the module draws it, Custom's too; else a named row's
/// training warning, as its long card does.
fn warn_id(s: &Service) -> Option<String> {
    let own = s.facts.warn.as_ref().map(|f| f.id.clone());
    own.or_else(|| s.name.is_some().then(|| TRAINS.to_owned()))
}

/// The rows a panel's page holds.
fn per() -> usize {
    UiSettings::from_defaults()
        .expect("the [ui] defaults")
        .page_rows
}

/// Row `s` of the table on the Cloud page, untested, a named one with its warning mark, with the mark its address asks
/// for; `tick`ed or not.
fn long_row(s: &Service, tick: bool) -> ServiceRow {
    ServiceRow {
        name: s.shown(),
        mark: if matches!(s.address, Address::Typed { .. }) {
            ServiceMark::NeedsAddress
        } else {
            ServiceMark::NeedsKey
        },
        untested: true,
        free: s.name.is_some().then(|| s.facts.free.clone()),
        warn: warn_id(s),
        tick,
    }
}

/// The Cloud pages with every row of the table, a panel's rows to a page, each page's first row ticked.
fn cloud_long() -> Vec<Shown> {
    let rows = services().rows();
    let count = u16::try_from(rows.len().div_ceil(per())).expect("a few pages");
    let acts = vec![CloudButton::Facts, CloudButton::Lines, CloudButton::Off];
    let page = |(at, part): (u16, &[Service])| {
        let rows = part.iter().enumerate().map(|(i, s)| long_row(s, i == 0));
        let sheet = VoiceSheet::Cloud(CloudSheet {
            on: true,
            rows: rows.collect(),
            acts: acts.clone(),
        });
        paged(shown(sheet, &[]), at, count, 1)
    };
    (0..count).zip(rows.chunks(per())).map(page).collect()
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

/// Part `part` of service `s`'s card as the module builds it from the table, asking again, old and untested, with the
/// training warning when the table gives it none.
fn card_long(s: &Service, part: CardPart) -> VoiceSheet {
    let settings = VoiceSettings::from_defaults().expect("the [voice] defaults");
    let mut c = Cloud::new(services(), false).card(&settings, s, s.facts.read);
    let VoiceSheet::Card(sample) = card(true) else {
        unreachable!("a card")
    };
    c.warn = c.warn.or_else(|| sample.warn.clone());
    (c.old, c.untested, c.on, c.part) = (true, true, None, part);
    VoiceSheet::Card(Box::new(c))
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
/// on page 2. Each Cloud page is named by its number, and each service of the table has its card's two parts.
pub fn panels() -> Vec<(String, Shown)> {
    let two = |sheet, at| paged(shown(sheet, &[]), at, 2, 0);
    let mut out = vec![
        (
            "modelsLong",
            paged(shown(models_long(), &[PanelButton::Use]), 0, 1, 1),
        ),
        ("serviceLong", two(service_long(LinesPart::Key), 0)),
        ("serviceLongModel", two(service_long(LinesPart::Model), 1)),
        ("compareLong", shown(compare_long(), &[])),
    ]
    .into_iter()
    .map(|(name, s)| (name.to_owned(), s))
    .collect::<Vec<_>>();
    let pages = cloud_long().into_iter().enumerate();
    out.extend(pages.map(|(i, s)| (format!("{CLOUD_LONG}{}", i + 1), s)));
    for s in services().rows() {
        out.push((
            format!("{CARD_LONG}{}", s.id),
            two(card_long(s, CardPart::Voice), 0),
        ));
        out.push((
            format!("{PRICE_LONG}{}", s.id),
            two(card_long(s, CardPart::Price), 1),
        ));
    }
    out
}

/// Every row of the table shows on the long Cloud pages, a panel's rows to a page, so the fit test measures each one.
#[test]
fn the_long_cloud_pages_hold_every_row_of_the_table_page_by_page() {
    let per = per();
    let all: Vec<ServiceName> = services().rows().iter().map(Service::shown).collect();
    let named = panels()
        .into_iter()
        .filter(|(n, _)| n.starts_with(CLOUD_LONG));
    let pages: Vec<Shown> = named.map(|(_, s)| s).collect();
    let count = u16::try_from(all.len().div_ceil(per)).expect("a few pages");
    assert_eq!(pages.len(), usize::from(count), "one panel a page");
    let mut shown = Vec::new();
    for (at, p) in (0..count).zip(&pages) {
        assert_eq!(p.pages, (at, count), "page {at}");
        let VoiceSheet::Cloud(c) = &p.sheet else {
            panic!("a Cloud page")
        };
        assert!(c.rows.len() <= per, "page {at} keeps a page of rows");
        shown.extend(c.rows.iter().map(|r| r.name.clone()));
    }
    assert_eq!(shown, all);
}

/// A Custom row shows its own warning as the module does, and none without one, so the fit test measures it too.
#[test]
fn a_custom_row_shows_its_own_warning_and_none_without_one() {
    let rows = services().rows();
    let mut custom = rows
        .iter()
        .find(|s| s.name.is_none())
        .expect("Custom")
        .clone();
    assert_eq!(long_row(&custom, false).warn, None, "no warning of its own");
    custom.facts.warn = Some(Fact {
        id: "keeps-logs".to_owned(),
        args: Vec::new(),
        warn: true,
    });
    let warn = long_row(&custom, false).warn;
    assert_eq!(warn.as_deref(), Some("keeps-logs"));
}

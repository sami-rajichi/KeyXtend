//! The cloud pages as QML reads them (ADR-0047): the Cloud page with its switch and services, a service's card with
//! its facts, the day they were read and its link, and the parts the key page, Models, Compare and the bar share.

use super::acts::{CLOUD, CONFIRM, OPEN, REMOVE, SERVICE};
use super::panel::{Cx, Page, Sheet};
use super::parts::{Button, Hint, Tone, word};
use super::sheets::{mark, mark_key, tab_id};
use crate::I18n;
use crate::i18n::keys::vars::{DATE, FREE, HOURS, NAME, NAMES, SITE};
use crate::i18n::keys::{
    CARD_BILLS, CARD_LABELS, CARD_RULE, FACT_BILLS, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_RULE,
    FACT_TRAINS, FACT_WARN, FACT_WHERE, VOICE_CARD, VOICE_CARD_ASK, VOICE_CARD_CONFIRM,
    VOICE_CARD_CUSTOM, VOICE_CARD_HOURS, VOICE_CARD_LINK, VOICE_CARD_OLD, VOICE_CARD_ON,
    VOICE_CARD_READ_ON, VOICE_CARD_SUB, VOICE_CARD_UNTESTED, VOICE_CLOUD_FREE, VOICE_CLOUD_MODEL,
    VOICE_CLOUD_OFF, VOICE_CLOUD_SUB, VOICE_CLOUD_SWITCH, VOICE_CUSTOM, VOICE_CUSTOM_LINE,
    VOICE_JOIN, VOICE_OFF, VOICE_ON, VOICE_ON_BAR_CLOUD, VOICE_SEND_WARNING, VoiceMark,
    cloud_button_id, cloud_button_key, panel_button_id, service_mark, warn_mark_key,
};
use crate::i18n::{Said, clean};
use kx_module_api::ports::{
    CardPart, CloudButton, CloudCard, CloudRow, CloudSheet, CloudWhy, CompareCloud, Fact,
    PanelButton, ServiceMark, ServiceName, ServiceRow, UsePrice, VoiceTab,
};
use serde::Serialize;

/// The card's page id.
pub(super) const CARD: &str = "card";
/// The id of the mark naming a cloud model's service, which takes the cloud icon.
const MARK_CLOUD: &str = "cloud";
/// The id of a row's warning mark, which takes the warning tone; its word is its warning fact's own.
const MARK_WARN: &str = "warn";

/// A switch's row: its label, its value's word and its state; a click sends `kind` with the other state.
#[derive(Serialize)]
pub(super) struct Switch {
    kind: &'static str,
    label: String,
    value: String,
    on: bool,
}

impl Switch {
    /// The switch `label`, `on` or off, whose click sends `kind`.
    pub(super) fn new(t: &I18n, kind: &'static str, label: String, on: bool) -> Self {
        let value = t.text(if on { VOICE_ON } else { VOICE_OFF });
        Self {
            kind,
            label,
            value,
            on,
        }
    }
}

/// A line with the button it leads to, such as a band's to the Cloud page.
#[derive(Serialize)]
pub(super) struct Lead {
    text: String,
    button: Button,
}

impl Lead {
    /// The line `text` with `button`.
    pub(super) const fn new(text: String, button: Button) -> Self {
        Self { text, button }
    }
}

/// A service's row on the Cloud page: its name, its line, the Untested tag, its marks and a sealed key's Remove key.
#[derive(Serialize)]
pub(super) struct ServiceView {
    name: String,
    line: String,
    tag: Option<String>,
    marks: Vec<Hint>,
    remove: Option<Button>,
    tick: bool,
}

/// A card's fact: its label's id and words, its sentence in parts, and whether it takes the warning tone.
#[derive(Serialize)]
pub(super) struct FactView {
    id: &'static str,
    label: String,
    value: Said,
    warn: bool,
}

/// The day a card's facts were read, the words after it once they are old, and the link to the service's page.
#[derive(Serialize)]
pub(super) struct Read {
    text: String,
    old: Option<String>,
    link: Option<Button>,
}

/// A cloud model Compare may send the recording to, with its service, ticked or not.
#[derive(Serialize)]
pub(super) struct PickView {
    name: String,
    service: String,
    tick: bool,
}

/// The bar's cloud mark: the service the voice goes to at Stop, and the line readers hear.
#[derive(Serialize)]
pub(super) struct CloudMark {
    name: String,
    said: String,
}

/// Service `name` as the pages name it; the Custom row in the user's language.
pub(super) fn service(t: &I18n, name: &ServiceName) -> String {
    match name {
        ServiceName::Named(n) => clean(n),
        ServiceName::Custom => t.text(VOICE_CUSTOM),
    }
}

/// Cloud button `b` with its word, acting while `acts` holds it.
pub(super) fn button(t: &I18n, b: CloudButton, acts: &[CloudButton]) -> Button {
    let label = t.text(cloud_button_key(b));
    Button::new(SERVICE, cloud_button_id(b), 0, label).on(acts.contains(&b))
}

/// The button that opens the Cloud page.
pub(super) fn to_cloud(cx: &Cx<'_>) -> Button {
    let tab = VoiceTab::Cloud;
    Button::new(OPEN, tab_id(tab), 0, cx.tab(tab))
}

/// The Cloud page: the `cloud` switch, the line while off, the services, then Facts, Key and model, Switch off and
/// Switch on.
pub(super) fn page(cx: &Cx<'_>, c: &CloudSheet) -> Page {
    let t = cx.t;
    let sheet = Sheet::Cloud {
        switch: Switch::new(t, CLOUD, t.text(VOICE_CLOUD_SWITCH), c.on),
        off: (!c.on).then(|| t.text(VOICE_CLOUD_OFF)),
        rows: c
            .rows
            .iter()
            .zip(0..)
            .map(|(r, at)| row(t, r, at))
            .collect(),
    };
    let tab = VoiceTab::Cloud;
    let mut page = Page::new(tab_id(tab), cx.tab(tab), t.text(VOICE_CLOUD_SUB), sheet);
    let b = |x| button(t, x, &c.acts);
    page.foot = vec![
        b(CloudButton::Facts),
        b(CloudButton::Lines),
        b(CloudButton::Off),
        b(CloudButton::On).tone(Tone::Primary),
    ];
    page
}

/// Service row `r` at place `at`: its free allowance, or what the Custom row asks for, its warning fact's own mark
/// before its state, or its own Remove key in the Off mark's place.
fn row(t: &I18n, r: &ServiceRow, at: i32) -> ServiceView {
    let line = r.free.as_ref().map_or_else(
        || t.text(VOICE_CUSTOM_LINE),
        |f| {
            let free = t.voice_fact(FACT_FREE, f).text;
            t.voice_with(VOICE_CLOUD_FREE, &[(FREE, free)])
        },
    );
    let warn = r
        .warn
        .as_ref()
        .map(|id| Hint::new(MARK_WARN, t.text(&warn_mark_key(id))));
    let state = (!r.remove).then(|| mark(t, service_mark(r.mark)));
    ServiceView {
        name: service(t, &r.name),
        line,
        tag: r.untested.then(|| t.text(&mark_key(VoiceMark::Untested))),
        marks: warn.into_iter().chain(state).collect(),
        remove: r.remove.then(|| {
            let b = CloudButton::Remove;
            Button::new(REMOVE, cloud_button_id(b), at, t.text(cloud_button_key(b)))
        }),
        tick: r.tick,
    }
}

/// A service's card: its facts, the day read with the link, Untested and the warning; Cancel and Confirm while it
/// asks, else the day it was switched on.
pub(super) fn card(cx: &Cx<'_>, c: &CloudCard) -> Page {
    let t = cx.t;
    let read = t.voice_with(VOICE_CARD_READ_ON, &[(DATE, t.voice_date(c.read))]);
    let link = c.site.as_ref().map(|s| {
        let label = t.voice_with(VOICE_CARD_LINK, &[(SITE, clean(s))]);
        Button::new(SERVICE, cloud_button_id(CloudButton::Link), 0, label)
    });
    let sheet = Sheet::Card {
        facts: facts(t, c),
        read: Read {
            text: read,
            old: c.old.then(|| t.text(VOICE_CARD_OLD)),
            link,
        },
        untested: c.untested.then(|| t.text(VOICE_CARD_UNTESTED)),
        warn: c.warn.as_ref().map(|w| t.voice_fact(FACT_WARN, w)),
    };
    let sub = c.on.map_or_else(
        || t.text(VOICE_CARD_SUB),
        |d| t.voice_with(VOICE_CARD_ON, &[(DATE, t.voice_date(d))]),
    );
    let mut page = Page::new(CARD, service(t, &c.service), sub, sheet);
    if c.on.is_none() {
        let who = match &c.service {
            ServiceName::Named(n) => clean(n),
            ServiceName::Custom => t.text(VOICE_CARD_CUSTOM),
        };
        page.line = Some(t.voice_with(VOICE_CARD_ASK, &[(NAME, who)]));
        let cancel = word(panel_button_id(PanelButton::Cancel));
        let yes = Button::new(CONFIRM, "", 1, t.text(VOICE_CARD_CONFIRM));
        page.foot = vec![
            Button::new(CONFIRM, "", 0, t.text(&cancel)),
            yes.tone(Tone::Primary),
        ];
    }
    page
}

/// The facts of the card's shown part in their fixed order, then the least billed and the service's rule if any.
fn facts(t: &I18n, c: &CloudCard) -> Vec<FactView> {
    let label = |id: &str| t.text(&format!("{VOICE_CARD}{id}"));
    let fact = |id, label, group, f: &Fact| FactView {
        id,
        label,
        value: t.voice_fact(group, f),
        warn: f.warn,
    };
    let used = |id, u: &UsePrice| {
        let hours = t.counted(VOICE_CARD_HOURS, usize::from(u.hours));
        let label = t.voice_with(&format!("{VOICE_CARD}{id}"), &[(HOURS, hours)]);
        fact(id, label, FACT_PRICE, &u.price)
    };
    let [place, trains, keeps, free, light, heavy] = CARD_LABELS;
    match c.part {
        CardPart::Voice => vec![
            fact(place, label(place), FACT_WHERE, &c.place),
            fact(trains, label(trains), FACT_TRAINS, &c.trains),
            fact(keeps, label(keeps), FACT_KEEPS, &c.keeps),
        ],
        CardPart::Price => {
            let mut out = vec![
                fact(free, label(free), FACT_FREE, &c.free),
                used(light, &c.light),
                used(heavy, &c.heavy),
            ];
            let bills = c.bills.as_ref();
            out.extend(bills.map(|b| fact(CARD_BILLS, label(CARD_BILLS), FACT_BILLS, b)));
            let rule = c.rule.as_ref();
            out.extend(rule.map(|r| fact(CARD_RULE, label(CARD_RULE), FACT_RULE, r)));
            out
        }
    }
}

/// A cloud model's line and marks on Models: in the cloud, or why it cannot be used now; its service, then where it
/// stands, In use while it is the pick.
pub(super) fn model(t: &I18n, used: bool, c: &CloudRow) -> (String, Vec<Hint>) {
    let line = c.unavailable.map_or_else(
        || t.text(VOICE_CLOUD_MODEL),
        |u| t.voice_cloud(&CloudWhy::Unavailable(u)),
    );
    let state = match (c.unavailable, c.mark) {
        (Some(_), _) => VoiceMark::Unavailable,
        (None, ServiceMark::Ready) if used => VoiceMark::Used,
        (None, ServiceMark::Ready) => VoiceMark::Ready,
        (None, m) => service_mark(m),
    };
    let name = Hint::new(MARK_CLOUD, service(t, &c.service));
    (line, vec![name, mark(t, state)])
}

/// Compare's cloud model `c`.
pub(super) fn pick(t: &I18n, c: &CompareCloud) -> PickView {
    PickView {
        name: clean(c.name.expose()),
        service: service(t, &c.service),
        tick: c.tick,
    }
}

/// Compare's warning before Start, naming the services the recording goes to; `None` for none.
pub(super) fn send(t: &I18n, send: &[ServiceName]) -> Option<String> {
    let names: Vec<String> = send.iter().map(|s| service(t, s)).collect();
    let names = names.join(&t.text(VOICE_JOIN));
    (!send.is_empty()).then(|| t.voice_with(VOICE_SEND_WARNING, &[(NAMES, names)]))
}

/// The bar's cloud mark for `name`.
pub(super) fn cloud_mark(t: &I18n, name: &ServiceName) -> CloudMark {
    let name = service(t, name);
    let said = t.voice_with(VOICE_ON_BAR_CLOUD, &[(NAME, name.clone())]);
    CloudMark { name, said }
}

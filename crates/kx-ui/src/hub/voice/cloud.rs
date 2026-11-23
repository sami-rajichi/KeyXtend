//! The cloud pages as QML reads them (ADR-0047): the Cloud page with its switch and services, and the parts the card,
//! the key page, Models, Compare and the bar's chip share.

use super::acts::{CLOUD, OPEN, SERVICE};
use super::panel::{Cx, Page, Sheet};
use super::parts::{Button, Hint, Tone, heard};
use super::sheets::{mark, mark_key, tab_id};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::{FREE, NAMES};
use crate::i18n::keys::{
    FACT_FREE, VOICE_CARD_CUSTOM, VOICE_CLOUD_FREE, VOICE_CLOUD_MODEL, VOICE_CLOUD_OFF,
    VOICE_CLOUD_SUB, VOICE_CLOUD_SWITCH, VOICE_CUSTOM, VOICE_CUSTOM_LINE, VOICE_OFF, VOICE_ON,
    VOICE_SEND_WARNING, VoiceMark, cloud_button_id, cloud_button_key, service_mark, warn_mark_key,
};
use kx_module_api::ports::{
    CloudButton, CloudRow, CloudSheet, CloudWhy, CompareCloud, ServiceMark, ServiceName,
    ServiceRow, VoiceTab,
};
use serde::Serialize;

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

/// A service's row on the Cloud page: its name, its line, the Untested tag and its marks.
#[derive(Serialize)]
pub(super) struct ServiceView {
    name: String,
    line: String,
    tag: Option<String>,
    marks: Vec<Hint>,
    tick: bool,
}

/// A cloud model Compare may send the recording to, with its service, the line readers hear, ticked or not.
#[derive(Serialize)]
pub(super) struct PickView {
    name: String,
    service: String,
    said: String,
    tick: bool,
}

/// Service `name` as the pages name it; the Custom row in the user's language.
pub(super) fn service(t: &I18n, name: &ServiceName) -> String {
    match name {
        ServiceName::Named(n) => clean(n),
        ServiceName::Custom => t.text(VOICE_CUSTOM),
    }
}

/// Service `name` inside a sentence; the Custom row as the address the user types.
pub(super) fn in_sentence(t: &I18n, name: &ServiceName) -> String {
    match name {
        ServiceName::Named(n) => clean(n),
        ServiceName::Custom => t.text(VOICE_CARD_CUSTOM),
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

/// The Cloud page: the `cloud` switch, the line while off, the services, then Facts, Key and model, Remove key, Switch
/// off and Switch on.
pub(super) fn page(cx: &Cx<'_>, c: &CloudSheet) -> Page {
    let t = cx.t;
    let sheet = Sheet::Cloud {
        switch: Switch::new(t, CLOUD, t.text(VOICE_CLOUD_SWITCH), c.on),
        off: (!c.on).then(|| t.text(VOICE_CLOUD_OFF)),
        rows: c.rows.iter().map(|r| row(t, r)).collect(),
    };
    let tab = VoiceTab::Cloud;
    let mut page = Page::new(tab_id(tab), cx.tab(tab), t.text(VOICE_CLOUD_SUB), sheet);
    let b = |x| button(t, x, &c.acts);
    page.foot = vec![
        b(CloudButton::Facts),
        b(CloudButton::Lines),
        b(CloudButton::Remove).tone(Tone::Danger),
        b(CloudButton::Off),
        b(CloudButton::On).tone(Tone::Primary),
    ];
    page
}

/// Service row `r`: its free allowance, or what the Custom row asks for, its warning fact's own mark before its state.
fn row(t: &I18n, r: &ServiceRow) -> ServiceView {
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
    ServiceView {
        name: service(t, &r.name),
        line,
        tag: r.untested.then(|| t.text(&mark_key(VoiceMark::Untested))),
        marks: warn
            .into_iter()
            .chain([mark(t, service_mark(r.mark))])
            .collect(),
        tick: r.tick,
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
    let (name, service) = (clean(c.name.expose()), service(t, &c.service));
    PickView {
        said: heard(t, &name, &service),
        name,
        service,
        tick: c.tick,
    }
}

/// Compare's warning before Start, naming the services the recording goes to in a list that ends with "and"; `None`
/// for none.
pub(super) fn send(t: &I18n, send: &[ServiceName]) -> Option<String> {
    let names: Vec<String> = send.iter().map(|s| in_sentence(t, s)).collect();
    (!send.is_empty()).then(|| t.voice_with(VOICE_SEND_WARNING, &[(NAMES, t.and_list(&names))]))
}

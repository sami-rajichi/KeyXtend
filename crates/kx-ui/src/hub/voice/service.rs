//! A service's key page as QML reads it (ADR-0047): the key line as dots with Paste and Test and the test's answer,
//! the address and Other model lines with why one was refused, the keep-nothing switch, and where keys are made.
//! A typed line shows its text only on the panel's own line (ADR-0040); the key never leaves as text.

use super::acts::KEEP;
use super::cloud::{self, Lead, Switch};
use super::panel::{Cx, Page, Sheet};
use super::parts::{Button, Hint, Tone, word};
use super::sheets::mark_key;
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::SITE;
use crate::i18n::keys::{
    VOICE_ADDRESS, VOICE_ADDRESS_HINT, VOICE_ADDRESS_HINT_OWN, VOICE_ADDRESS_LINE,
    VOICE_KEEP_NOTHING, VOICE_KEY, VOICE_KEY_HINT, VOICE_KEY_LINE, VOICE_KEY_LINK,
    VOICE_KEY_SEALED, VOICE_MASK, VOICE_MODEL_HINT, VOICE_MODEL_LINE, VOICE_MODEL_REFUSED,
    address_fault_id, key_check_id, panel_button_id, service_mark,
};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CloudButton, CloudWhy, PanelButton, ServiceName, ServiceSheet, Unavailable,
};
use serde::Serialize;

/// The key page's id.
pub(super) const SERVICE_PAGE: &str = "service";
/// The ids of its lines.
const KEY_LINE: &str = "key";
const ADDRESS_LINE: &str = "address";
const MODEL_LINE: &str = "model";

/// A typed line: its caption, its hint while empty, its text, or for the key its length and the mask `PanelLine` draws
/// its dots from, why it was refused, and its buttons.
#[derive(Serialize)]
pub(super) struct LineView {
    id: &'static str,
    caption: String,
    hint: String,
    text: String,
    dots: u32,
    mask: Option<String>,
    fault: Option<String>,
    buttons: Vec<Button>,
}

/// The key page's body: the key line, the test's answer, the address and model lines, keep-nothing and where keys are
/// made.
#[derive(Serialize)]
pub(super) struct KeyPage {
    key: LineView,
    check: Option<Hint>,
    address: Option<LineView>,
    model: LineView,
    keep: Option<Switch>,
    link: Option<Lead>,
}

/// The key page: its lines and answers, Facts, Remove key and Switch off, with the line that the key is sealed.
pub(super) fn page(cx: &Cx<'_>, s: &ServiceSheet) -> Page {
    let t = cx.t;
    let b = |x| cloud::button(t, x, &s.acts);
    let key = LineView {
        id: KEY_LINE,
        caption: t.text(VOICE_KEY_LINE),
        hint: t.text(VOICE_KEY_HINT),
        text: String::new(),
        dots: *s.key.expose(),
        mask: Some(t.text(VOICE_MASK)),
        fault: None,
        buttons: vec![b(CloudButton::Paste), b(CloudButton::Test)],
    };
    let check = key_check_id(s.check).map(|id| Hint::new(id, t.text(&format!("{VOICE_KEY}{id}"))));
    let link = s.site.as_ref().map(|site| {
        let text = t.voice_with(VOICE_KEY_LINK, &[(SITE, clean(site))]);
        let open = t.text(&word(panel_button_id(PanelButton::Open)));
        Lead::new(text, b(CloudButton::Link).labelled(open))
    });
    let sheet = Sheet::Service(Box::new(KeyPage {
        key,
        check,
        address: s.address.as_ref().map(|a| address(t, s, a)),
        model: model(t, s),
        keep: s
            .keep
            .map(|on| Switch::new(t, KEEP, t.text(VOICE_KEEP_NOTHING), on)),
        link,
    }));
    let sub = t.text(&mark_key(service_mark(s.mark)));
    let mut page = Page::new(SERVICE_PAGE, cloud::service(t, &s.service), sub, sheet);
    page.line = Some(t.text(VOICE_KEY_SEALED));
    page.foot = vec![
        b(CloudButton::Facts),
        b(CloudButton::Remove).tone(Tone::Danger),
        b(CloudButton::Off),
    ];
    page
}

/// The address line with `a`: the Custom row's hint names an HTTPS host, a service's own its resource.
fn address(t: &I18n, s: &ServiceSheet, a: &Redacted<String>) -> LineView {
    let hint = if s.service == ServiceName::Custom {
        VOICE_ADDRESS_HINT
    } else {
        VOICE_ADDRESS_HINT_OWN
    };
    let fault = s
        .address_fault
        .map(|f| t.voice_https(&format!("{VOICE_ADDRESS}{}", address_fault_id(f))));
    LineView {
        id: ADDRESS_LINE,
        caption: t.text(VOICE_ADDRESS_LINE),
        hint: t.voice_https(hint),
        text: clean(a.expose()),
        dots: 0,
        mask: None,
        fault,
        buttons: Vec::new(),
    }
}

/// The Other model line: why its id was refused, or why no host of it keeps nothing (edge 17).
fn model(t: &I18n, s: &ServiceSheet) -> LineView {
    let fault = if s.model_refused {
        Some(t.text(VOICE_MODEL_REFUSED))
    } else {
        let why = CloudWhy::Unavailable(Unavailable::KeepNothing);
        s.model_na.then(|| t.voice_cloud(&why))
    };
    LineView {
        id: MODEL_LINE,
        caption: t.text(VOICE_MODEL_LINE),
        hint: t.text(VOICE_MODEL_HINT),
        text: clean(s.model.expose()),
        dots: 0,
        mask: None,
        fault,
        buttons: Vec::new(),
    }
}

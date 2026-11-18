//! A service's key page as QML reads it (ADR-0047): the key line as dots with Paste and Test and the test's answer,
//! the address and Other model lines with why one was refused, the keep-nothing switch, and where keys are made.
//! A typed line shows its text only on the panel's own line (ADR-0040); the key never leaves as text.

use super::acts::{KEEP, SAVE};
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
    VOICE_KEY_SEALED, VOICE_LINE_SAVE, VOICE_LINE_SAVE_NAME, VOICE_MASK, VOICE_MODEL_HINT,
    VOICE_MODEL_LINE, VOICE_MODEL_REFUSED, address_fault_id, cloud_line_id, key_check_id,
    panel_button_id, service_mark,
};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CloudButton, CloudLine, CloudWhy, LinesPart, PanelButton, ServiceName, ServiceSheet,
    Unavailable,
};
use serde::Serialize;

/// The key page's id.
pub(super) const SERVICE_PAGE: &str = "service";
/// A typed line: its caption, its hint while empty, its text, or for the key its sealed and typed lengths and the mask
/// `PanelLine` draws their dots from, whether it takes the letters and its caret there (ADR-0049), why it was refused,
/// and its buttons, Save among them.
#[derive(Serialize)]
pub(super) struct LineView {
    id: &'static str,
    caption: String,
    hint: String,
    text: String,
    dots: u32,
    typed: u32,
    mask: Option<String>,
    typing: bool,
    at: u32,
    fault: Option<String>,
    buttons: Vec<Button>,
}

impl LineView {
    /// Line `line` of `s` with `words`, its caption's key and its hint, and `text`; its caret while it takes the letters,
    /// and its Save, which dims while it is empty and names its line for readers.
    fn new(
        t: &I18n,
        s: &ServiceSheet,
        line: CloudLine,
        words: (&str, String),
        text: String,
    ) -> Self {
        let typing = s.typing == line;
        let id = cloud_line_id(line);
        let save = Button::new(SAVE, id, 0, t.text(VOICE_LINE_SAVE))
            .named(t.text(&format!("{VOICE_LINE_SAVE_NAME}{id}")));
        Self {
            id,
            caption: t.text(words.0),
            hint: words.1,
            text,
            dots: 0,
            typed: 0,
            mask: None,
            typing,
            at: if typing { *s.caret.expose() } else { 0 },
            fault: None,
            buttons: vec![save.on(s.saves.contains(&line))],
        }
    }
}

/// The key page's body: the key line, the test's answer, the address and model lines, keep-nothing and where keys are
/// made; a part of two shows only its own.
#[derive(Serialize)]
pub(super) struct KeyPage {
    key: Option<LineView>,
    check: Option<Hint>,
    address: Option<LineView>,
    model: Option<LineView>,
    keep: Option<Switch>,
    link: Option<Lead>,
}

/// The key page: its lines and answers, Facts, Remove key and Switch off, with the line that the key is sealed.
pub(super) fn page(cx: &Cx<'_>, s: &ServiceSheet) -> Page {
    let t = cx.t;
    let b = |x| cloud::button(t, x, &s.acts);
    let (first, rest) = (
        s.part != Some(LinesPart::Model),
        s.part != Some(LinesPart::Key),
    );
    let check = key_check_id(s.check).map(|id| Hint::new(id, t.text(&format!("{VOICE_KEY}{id}"))));
    let link = s.site.as_ref().map(|site| {
        let text = t.voice_with(VOICE_KEY_LINK, &[(SITE, clean(site))]);
        let open = t.text(&word(panel_button_id(PanelButton::Open)));
        Lead::new(text, b(CloudButton::Link).labelled(open))
    });
    let keep = |on| Switch::new(t, KEEP, t.text(VOICE_KEEP_NOTHING), on);
    let sheet = Sheet::Service(Box::new(KeyPage {
        key: first.then(|| key(t, s)),
        check: check.filter(|_| first),
        address: s
            .address
            .as_ref()
            .filter(|_| rest)
            .map(|a| address(t, s, a)),
        model: rest.then(|| model(t, s)),
        keep: s.keep.filter(|_| rest).map(keep),
        link: link.filter(|_| first),
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

/// The key line: the sealed key's dots and the typed key's, never its text, the mask `PanelLine` draws them from, and
/// Paste, Save and Test.
fn key(t: &I18n, s: &ServiceSheet) -> LineView {
    let b = |x| cloud::button(t, x, &s.acts);
    let words = (VOICE_KEY_LINE, t.text(VOICE_KEY_HINT));
    let mut line = LineView::new(t, s, CloudLine::Key, words, String::new());
    (line.dots, line.typed) = (*s.key.expose(), *s.typed.expose());
    line.mask = Some(t.text(VOICE_MASK));
    line.buttons.insert(0, b(CloudButton::Paste));
    line.buttons.push(b(CloudButton::Test));
    line
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
    let words = (VOICE_ADDRESS_LINE, t.voice_https(hint));
    let mut line = LineView::new(t, s, CloudLine::Address, words, clean(a.expose()));
    line.fault = fault;
    line
}

/// The Other model line: why its id was refused, or why no host of it keeps nothing (edge 17).
fn model(t: &I18n, s: &ServiceSheet) -> LineView {
    let fault = if s.model_refused {
        Some(t.text(VOICE_MODEL_REFUSED))
    } else {
        let why = CloudWhy::Unavailable(Unavailable::KeepNothing);
        s.model_na.then(|| t.voice_cloud(&why))
    };
    let words = (VOICE_MODEL_LINE, t.text(VOICE_MODEL_HINT));
    let mut line = LineView::new(t, s, CloudLine::Model, words, clean(s.model.expose()));
    line.fault = fault;
    line
}

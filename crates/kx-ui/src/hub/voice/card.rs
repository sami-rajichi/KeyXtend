//! A service's card as QML reads it (ADR-0047): its facts, the day they were read with the words after it once old and
//! its link, Untested and the warning, each with the line readers hear; Cancel and Confirm while it asks.

use super::acts::{CONFIRM, SERVICE};
use super::cloud::{in_sentence, service};
use super::panel::{Cx, Page, Sheet};
use super::parts::{Button, Tone, heard, word};
use crate::I18n;
use crate::i18n::keys::vars::{DATE, HOURS, NAME, SITE};
use crate::i18n::keys::{
    CARD_BILLS, CARD_LABELS, CARD_RULE, FACT_BILLS, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_RULE,
    FACT_TRAINS, FACT_WARN, FACT_WHERE, VOICE_CARD, VOICE_CARD_ASK, VOICE_CARD_CONFIRM,
    VOICE_CARD_HOURS, VOICE_CARD_LINK, VOICE_CARD_OLD, VOICE_CARD_ON, VOICE_CARD_READ_OLD,
    VOICE_CARD_READ_ON, VOICE_CARD_SUB, VOICE_CARD_UNTESTED, cloud_button_id, panel_button_id,
};
use crate::i18n::{Said, clean};
use kx_module_api::ports::{CardPart, CloudButton, CloudCard, Fact, PanelButton, UsePrice};
use serde::Serialize;

/// The card's page id.
pub(super) const CARD: &str = "card";

/// A card's fact: its label's id and words, its sentence in parts, whether it takes the warning tone, and the line
/// readers hear.
#[derive(Serialize)]
pub(super) struct FactView {
    id: &'static str,
    label: String,
    value: Said,
    warn: bool,
    said: String,
}

/// The day a card's facts were read, the words after it once they are old, the line readers hear of both, and the link
/// to the service's page.
#[derive(Serialize)]
pub(super) struct Read {
    text: String,
    old: Option<String>,
    said: String,
    link: Option<Button>,
}

/// A service's card: its facts, the day read with the link, Untested and the warning; Cancel and Confirm while it
/// asks, else the day it was switched on.
pub(super) fn card(cx: &Cx<'_>, c: &CloudCard) -> Page {
    let t = cx.t;
    let sheet = Sheet::Card {
        facts: facts(t, c),
        read: read(t, c),
        untested: c.untested.then(|| t.text(VOICE_CARD_UNTESTED)),
        warn: c.warn.as_ref().map(|w| t.voice_fact(FACT_WARN, w)),
    };
    let sub = c.on.map_or_else(
        || t.text(VOICE_CARD_SUB),
        |d| t.voice_with(VOICE_CARD_ON, &[(DATE, t.voice_date(d))]),
    );
    let mut page = Page::new(CARD, service(t, &c.service), sub, sheet);
    if c.on.is_none() {
        let who = in_sentence(t, &c.service);
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

/// The day card `c` was read, the words after it once old, and the link to the service's page.
fn read(t: &I18n, c: &CloudCard) -> Read {
    let date = [(DATE, t.voice_date(c.read))];
    let link = c.site.as_ref().map(|s| {
        let label = t.voice_with(VOICE_CARD_LINK, &[(SITE, clean(s))]);
        Button::new(SERVICE, cloud_button_id(CloudButton::Link), 0, label)
    });
    Read {
        text: t.voice_with(VOICE_CARD_READ_ON, &date),
        old: c.old.then(|| t.text(VOICE_CARD_OLD)),
        said: t.voice_with(
            if c.old {
                VOICE_CARD_READ_OLD
            } else {
                VOICE_CARD_READ_ON
            },
            &date,
        ),
        link,
    }
}

/// The facts of the card's shown part in their fixed order, then the least billed and the service's rule if any.
fn facts(t: &I18n, c: &CloudCard) -> Vec<FactView> {
    let label = |id: &str| t.text(&format!("{VOICE_CARD}{id}"));
    let fact = |id, label: String, group, f: &Fact| {
        let value = t.voice_fact(group, f);
        let said = heard(t, &label, &value.text);
        FactView {
            id,
            label,
            value,
            warn: f.warn,
            said,
        }
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

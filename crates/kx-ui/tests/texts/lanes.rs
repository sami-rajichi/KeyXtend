//! The chip lanes the Qt Quick tests draw, as the hub writes them for `Ui.words`, and the ask kinds QML sends to
//! `Ui.wordsAsk`, by the name QML holds each under (WordNames.qml).

use crate::suggest::{chip, lane};
use kx_module_api::ports::{ChipKind, Lane, ListPage, Suggested};
use kx_module_api::{Bus, Redacted};
use kx_test_support::host::FakeCore;
use kx_ui::{Hub, asks};
use serde_json::{Value, json};
use std::sync::Arc;

/// The emails on the open list's first page, and how many pages it has.
const EMAILS: [&str; 5] = [
    "ali@example.org",
    "mona@example.net",
    "sam@example.com",
    "lea@example.fr",
    "omar@example.org",
];
const PAGES: u16 = 2;

/// Each ask kind by the name QML holds it under.
pub fn asks() -> Value {
    json!({
        "forget": asks::FORGET,
        "cancel": asks::CANCEL,
        "pause": asks::PAUSE,
        "power": asks::POWER,
        "undo": asks::UNDO,
        "more": asks::MORE,
        "page": asks::PAGE,
        "pick": asks::PICK,
        "again": asks::AGAIN,
    })
}

/// The open list's first page.
fn page() -> ListPage {
    ListPage {
        items: EMAILS.map(|e| Redacted::new(e.to_owned())).to_vec(),
        page: 0,
        pages: PAGES,
    }
}

/// The lanes by name as `hub` shows them: the start's, chips of every kind with an Arabic word, forget mode's word and
/// known-email chips, the lock, off, empty, stopped, and an open email list on its first page.
pub fn lanes(hub: &Arc<Hub>) -> Value {
    let start: Value = serde_json::from_str(&hub.words_json()).expect("lane JSON");
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    let shown = |lane: Suggested| -> Value {
        bus.publish(&lane).expect("the lane is sent");
        serde_json::from_str(&hub.words_json()).expect("lane JSON")
    };
    let chips = vec![
        chip("hello", ChipKind::Word),
        chip("أن", ChipKind::Word),
        chip("gmail.com", ChipKind::Domain),
        chip(EMAILS[0], ChipKind::Email),
        chip("", ChipKind::Undo),
        chip("", ChipKind::More),
    ];
    let list = Suggested {
        list: Some(page()),
        ..lane(11, Lane::Chips(chips.clone()))
    };
    let forget = Suggested {
        forgetting: true,
        ..lane(
            12,
            Lane::Chips([0, 1, 3].map(|i| chips[i].clone()).to_vec()),
        )
    };
    let off = Suggested {
        on: false,
        ..lane(9, Lane::Off)
    };
    json!({
        "start": start,
        "chips": shown(lane(7, Lane::Chips(chips))),
        "forget": shown(forget),
        "locked": shown(lane(8, Lane::Locked)),
        "off": shown(off),
        "empty": shown(lane(10, Lane::Empty)),
        "stopped": shown(lane(13, Lane::Stopped)),
        "list": shown(list),
    })
}

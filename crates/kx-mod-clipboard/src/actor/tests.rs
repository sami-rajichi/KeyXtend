//! The actor over the fakes: copies and their limits (`copies`), the panel (`panel`), inserting (`paste`, `joins`,
//! `stops`, `props`), saves (`saves`), the days (`days`) and pictures (`pictures`); `rig` drives it without a thread.
//! Here: a damaged list, and messages that print no content.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod copies;
mod days;
mod ears;
mod fakes;
mod flood;
mod images;
mod joins;
mod logs;
mod panel;
mod paste;
mod pictures;
mod props;
mod rig;
mod room;
mod saves;
mod stops;

use kx_module_api::ports::{BlobName, ClipNote, PanelKey};
use kx_module_api::{Redacted, keys};
use rig::{LIST, Rig};

use super::Msg;
use crate::fixture::{app, text};

#[test]
fn a_damaged_list_is_told_once_and_copies_go_on() {
    let mut rig = Rig::build(|f, _| {
        let day = BlobName::new("2026-10-30").unwrap();
        f.shelf.plant(&day, &BlobName::new(LIST).unwrap(), vec![1]);
    });
    assert_eq!(rig.notices(), [keys::CLIP_DAMAGED]);
    rig.text("after");
    rig.open();
    assert_eq!(rig.titles(), ["after"]);
    assert_eq!(rig.notices(), [keys::CLIP_DAMAGED], "told once");
}

#[test]
fn a_message_prints_no_copied_text_typed_search_or_app() {
    let secret = "zebra-quartz";
    let msgs = [
        Msg::Note(ClipNote::Copied {
            parts: vec![text(secret)],
            app: Some(app(secret)),
        }),
        Msg::Key(PanelKey::Text(Redacted::new(secret.to_owned()))),
    ];
    for msg in msgs {
        let shown = format!("{msg:?}");
        assert!(!shown.contains(secret), "{shown}");
    }
}

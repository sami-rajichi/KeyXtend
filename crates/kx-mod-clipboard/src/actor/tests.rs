//! The actor over the fakes: copies and their limits (`copies`), the panel (`panel`), inserting (`paste`, `joins`,
//! `stops`), saves (`saves`), the days (`days`) and pictures (`pictures`); `rig` drives it without a thread.

#![allow(
    clippy::unwrap_used,
    reason = "these tests and their rig fail the test on a failed setup"
)]

mod copies;
mod days;
mod ears;
mod fakes;
mod joins;
mod logs;
mod panel;
mod paste;
mod pictures;
mod rig;
mod saves;
mod stops;

use kx_module_api::keys;
use kx_module_api::ports::BlobName;
use rig::{LIST, Rig};

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

//! A copy brings one image format, PNG first, and counts only its bytes; an entry with PNG and a DIB still keeps both,
//! both count toward the cap, a copy of the same pair moves its entry up, and Insert writes both as they are.

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, ClipKind, ClipPart, ImageCodec, PanelNote};
use toml::Value;

use super::super::Msg;
use super::rig::Rig;
use crate::fixture::MB;

/// An image coded as `codec`, its bytes `bytes`.
fn image(codec: ImageCodec, bytes: Vec<u8>) -> ClipPart {
    ClipPart::Image {
        codec,
        bytes: Redacted::new(bytes),
    }
}

/// A copy holding a 3-byte PNG and a 5-byte DIB, both of bytes `n`.
fn pair(n: u8) -> Vec<ClipPart> {
    vec![
        image(ImageCodec::Png, vec![n; 3]),
        image(ImageCodec::Dib, vec![n; 5]),
    ]
}

#[test]
fn both_formats_are_kept_and_counted_and_a_twin_moves_up() {
    let mut rig = Rig::new();
    rig.copy(pair(1), None);
    rig.text("between");
    rig.copy(pair(1), None);
    rig.open();
    let rows = rig.page().rows;
    assert_eq!(rows.len(), 2, "one entry for the pair");
    assert_eq!((rows[0].kind, rows[0].bytes), (ClipKind::Image, Some(8)));
}

#[test]
fn both_formats_count_toward_the_entry_cap() {
    let mut rig = Rig::build(|_, host| host.put("max_entry_mb", Value::Integer(1)));
    let big = |codec| image(codec, vec![0; usize::try_from(MB * 3 / 5).unwrap()]);
    rig.copy(vec![big(ImageCodec::Png), big(ImageCodec::Dib)], None);
    rig.open();
    assert_eq!(rig.page().note, Some(PanelNote::TooBig));
    assert!(rig.page().empty);
}

#[test]
fn insert_writes_every_format_the_copy_had() {
    let mut rig = Rig::new();
    rig.copy(pair(2), None);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Insert));
    assert_eq!(rig.fakes.writer.writes(), [pair(2)]);
}

#[test]
fn a_png_alone_counts_only_its_own_bytes_toward_the_cap() {
    let mut rig = Rig::build(|_, host| host.put("max_entry_mb", Value::Integer(1)));
    let size = MB * 3 / 5;
    let png = image(ImageCodec::Png, vec![0; usize::try_from(size).unwrap()]);
    rig.copy(vec![png], None);
    rig.open();
    assert_eq!(rig.page().note, None);
    assert_eq!(rig.page().rows[0].bytes, Some(size));
}

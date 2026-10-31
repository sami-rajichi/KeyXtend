//! An inserted PNG goes with the DIB the media worker makes of it, so apps that read only `CF_DIB` get the picture;
//! the insert waits for it up to `media_ms`, and a worker down, late or refusing, or a DIB past the entry cap, leaves
//! the PNG alone with one line logged. An entry with a DIB goes as it is.

use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipAct, ClipPart, DibAsk, ImageCodec, PictureError};
use toml::Value;

use super::super::Msg;
use super::super::paste::Paste;
use super::logs::lines_of;
use super::rig::Rig;
use crate::fixture::{MB, image, text};

/// The DIB the fake worker makes.
fn made() -> Vec<u8> {
    vec![40; 44]
}

/// The DIB part an insert adds.
fn dib_part(bytes: Vec<u8>) -> ClipPart {
    ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(bytes),
    }
}

/// A rig holding `parts` copied, the panel open, and the row ticked and inserted.
fn inserted(rig: &mut Rig, parts: Vec<ClipPart>) {
    rig.copy(parts, None);
    rig.open();
    rig.tick_row(0);
    rig.send(Msg::Act(ClipAct::Insert));
}

#[test]
fn an_inserted_png_goes_with_the_workers_dib_and_the_store_keeps_the_png() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue_dib(Ok(made()));
    inserted(&mut rig, vec![image()]);
    assert_eq!(rig.fakes.writer.writes(), [vec![image(), dib_part(made())]]);
    let ClipPart::Image { bytes, .. } = image() else {
        unreachable!()
    };
    let max_bytes = u64::from(rig.settings.max_entry_mb) * MB;
    assert_eq!(
        rig.fakes.pics.last_dib(),
        Some(DibAsk {
            png: bytes,
            max_bytes
        })
    );
    assert_eq!(rig.pastes().len(), 1, "one paste for both formats");
    assert_eq!(rig.page().rows[0].bytes, Some(3), "only the PNG is stored");
}

#[test]
fn an_entry_with_a_dib_goes_as_it_is_and_asks_the_worker_nothing() {
    let pair = vec![image(), dib_part(vec![7; 5])];
    for parts in [pair, vec![dib_part(vec![8; 5])]] {
        let mut rig = Rig::new();
        inserted(&mut rig, parts.clone());
        assert_eq!(rig.fakes.writer.writes(), [parts]);
        assert_eq!(rig.fakes.pics.dib_asks(), 0);
    }
}

#[test]
fn a_worker_down_refusing_or_a_dib_past_the_cap_leaves_the_png_alone() {
    let past = vec![0; usize::try_from(MB).unwrap() + 1];
    let answers = [
        Err(PictureError::Down {
            retry_in: Duration::from_secs(1),
        }),
        Err(PictureError::Refused),
        Err(PictureError::TooBig),
        Ok(past),
    ];
    for answer in answers {
        let mut rig = Rig::build(|_, host| host.put("max_entry_mb", Value::Integer(1)));
        rig.fakes.pics.queue_dib(answer);
        let lines = lines_of(|| inserted(&mut rig, vec![image()]));
        assert_eq!(rig.fakes.writer.writes(), [vec![image()]]);
        assert_eq!(rig.pastes().len(), 1, "the insert goes on");
        let alone: Vec<_> = lines.iter().filter(|l| l.contains("PNG alone")).collect();
        assert_eq!(alone.len(), 1, "{lines:?}");
    }
}

#[test]
fn the_insert_waits_for_the_dib_up_to_media_ms_then_goes_as_png_alone() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue_dib(Ok(made()));
    rig.hold_draws(true);
    inserted(&mut rig, vec![image()]);
    assert!(matches!(rig.pasting(), Paste::Making { .. }));
    assert!(
        rig.fakes.writer.writes().is_empty(),
        "nothing before the DIB"
    );
    let media = Duration::from_millis(u64::from(rig.settings.media_ms));
    assert!(rig.wait() <= media, "the loop wakes for the deadline");
    rig.advance(media.saturating_sub(Duration::from_micros(1)));
    assert!(
        rig.fakes.writer.writes().is_empty(),
        "still within media_ms"
    );
    rig.advance(Duration::from_micros(1));
    assert_eq!(rig.fakes.writer.writes(), [vec![image()]]);
    rig.hold_draws(false);
    assert_eq!(rig.fakes.writer.writes().len(), 1, "a late DIB is dropped");
    assert_eq!(rig.pastes().len(), 1);
}

#[test]
fn a_held_dib_keeps_the_tick_order_and_the_gap() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue_dib(Ok(made()));
    rig.copy(vec![image()], None);
    rig.text("after");
    rig.open();
    rig.tick_row(1);
    rig.tick_row(0);
    rig.hold_draws(true);
    rig.send(Msg::Act(ClipAct::Insert));
    assert!(matches!(rig.pasting(), Paste::Making { .. }));
    rig.hold_draws(false);
    rig.answer(true);
    rig.advance(rig.settings.paste_gap());
    let writes = rig.fakes.writer.writes();
    assert_eq!(
        writes,
        [vec![image(), dib_part(made())], vec![text("after")]]
    );
}

#[test]
fn a_close_while_the_dib_is_made_stops_the_insert_and_drops_it() {
    let mut rig = Rig::new();
    rig.hold_draws(true);
    inserted(&mut rig, vec![image()]);
    rig.send(Msg::Act(ClipAct::Close));
    assert!(matches!(rig.pasting(), Paste::Idle));
    rig.hold_draws(false);
    assert!(rig.fakes.writer.writes().is_empty());
    assert!(rig.pastes().is_empty());
}

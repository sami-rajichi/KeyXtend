//! Pictures: a new image's thumbnail queues to `Pictures` at `thumb_px` and shows on its row, one thumbnail draws at
//! a time, a preview is drawn at `preview_px`, and a worker that fails leaves the type icon, asked once per run.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipAct, ClipKind, ClipNote, ClipPart, ImageCodec, PanelKey, PictureError, Pixels, Preview,
};

use super::super::Msg;
use super::rig::Rig;

/// A 2 by 1 picture.
fn pixels() -> Pixels {
    Pixels::new(2, 1, vec![7; 8], 2).unwrap()
}

/// A copied image.
fn image() -> Vec<ClipPart> {
    vec![ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
    }]
}

#[test]
fn a_new_images_thumbnail_queues_to_pictures_and_shows_on_its_row() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue(Ok(pixels()));
    rig.copy(image(), None);
    assert_eq!(rig.fakes.pics.asks(), 1);
    let ask = rig.fakes.pics.last_ask().unwrap();
    assert_eq!(
        (ask.side, ask.codec),
        (rig.settings.thumb_px, ImageCodec::Png)
    );
    rig.open();
    assert_eq!(rig.page().rows[0].thumb, Some(pixels()));
}

#[test]
fn a_preview_draws_the_image_large() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue(Ok(pixels()));
    rig.copy(image(), None);
    rig.open();
    rig.send(Msg::Act(ClipAct::Preview));
    assert_eq!(
        rig.fakes.pics.last_ask().map(|a| a.side),
        Some(rig.settings.preview_px)
    );
    assert_eq!(rig.page().preview, Some(Preview::Image(Some(pixels()))));
}

#[test]
fn a_worker_that_fails_leaves_the_type_icon_and_is_asked_once() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue(Err(PictureError::Gone));
    rig.copy(image(), None);
    rig.open();
    rig.send(Msg::Open);
    rig.open();
    let row = rig.page().rows[0].clone();
    assert_eq!((row.kind, row.thumb), (ClipKind::Image, None));
    assert_eq!(rig.fakes.pics.asks(), 1);
}

#[test]
fn thumbnails_draw_one_at_a_time_and_every_row_gets_its_own() {
    let mut rig = Rig::new();
    for n in 1..=3 {
        rig.fakes.pics.queue(Ok(pixels()));
        let parts = vec![ClipPart::Image {
            codec: ImageCodec::Png,
            bytes: Redacted::new(vec![n]),
        }];
        rig.fakes.clip.push(ClipNote::Copied { parts, app: None });
    }
    rig.open();
    assert_eq!(
        rig.fakes.pics.asks(),
        3,
        "the copies after the first wait for the view"
    );
    assert!(rig.page().rows.iter().all(|r| r.thumb.is_some()));
}

#[test]
fn a_preview_asked_while_another_draws_is_drawn_after_it_and_later_ones_too() {
    let mut rig = Rig::new();
    rig.fakes.pics.queue(Ok(pixels()));
    for n in 1..=3 {
        let parts = vec![ClipPart::Image {
            codec: ImageCodec::Png,
            bytes: Redacted::new(vec![n]),
        }];
        rig.copy(parts, None);
    }
    rig.open();
    let asks = rig.fakes.pics.asks();
    let large = Some(Preview::Image(Some(pixels())));
    rig.hold_draws(true);
    rig.send(Msg::Act(ClipAct::Preview));
    rig.send(Msg::Act(ClipAct::Back));
    rig.send(Msg::Key(PanelKey::Down));
    rig.send(Msg::Act(ClipAct::Preview));
    rig.hold_draws(false);
    assert_eq!(
        rig.fakes.pics.asks(),
        asks + 2,
        "the waiting preview went after the first"
    );
    assert_eq!(rig.page().preview, large);
    rig.send(Msg::Act(ClipAct::Back));
    rig.send(Msg::Key(PanelKey::Down));
    rig.send(Msg::Act(ClipAct::Preview));
    assert_eq!(
        rig.fakes.pics.asks(),
        asks + 3,
        "a later preview is drawn too"
    );
    assert_eq!(rig.page().preview, large);
}

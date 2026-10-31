//! Pictures: a new image's thumbnail queues to `Pictures` at `thumb_px` and shows on its row, a preview is drawn at
//! `preview_px`, and a worker that fails leaves the type icon, asked once per run.

use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipAct, ClipKind, ClipPart, ImageCodec, PictureError, Pixels, Preview,
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

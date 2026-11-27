//! Snip's picture provider serves each frozen screen over the shared pixels themselves, never a copy, with the unused
//! byte made opaque only as the texture loads; an old URL serves one clear pixel, the pixels go once the snip ends, and
//! the editor's grab is copied once into the freeze's byte order.

use cxx_qt_lib::{QColor, QImage, QImageFormat};
use kx_module_api::Bus;
use kx_module_api::ports::{
    BGRX_BYTES, BarKey, FrozenScreen, Keycap, Picking, Rect, Shot, SnipPage, SnipStage, SnipView,
};
use kx_test_support::host::FakeCore;
use serde_json::Value;
use std::sync::Arc;

/// Two pixels, blue 1 green 2 red 3, then blue 4 green 5 red 6, each with the freeze's unused 0.
const BGRX: [u8; 8] = [1, 2, 3, 0, 4, 5, 6, 0];
/// The same two pixels as the texture loads them: red, green, blue, opaque.
const LOADED: [u8; 8] = [3, 2, 1, 255, 6, 5, 4, 255];
/// One clear pixel, which QML draws as nothing.
const CLEAR: (u32, [u8; 4]) = (1, [0; 4]);
/// The provider's URLs start so.
const URL: &str = "image://snip/";

/// Round `round` picking over `frozen`, or nothing shown when it is empty.
fn page(round: u32, frozen: Vec<FrozenScreen>) -> SnipPage {
    let stage = frozen.first().map_or(SnipStage::Idle, |f| {
        SnipStage::Pick(Picking {
            screen: f.rect(),
            keys: vec![Keycap {
                key: BarKey::Region,
                on: true,
                dim: None,
            }],
            corner: None,
            within: None,
            windows: Vec::new(),
        })
    });
    SnipPage {
        round,
        frozen,
        stage,
        ask: None,
        note: None,
        panel: None,
        dialog: false,
    }
}

/// What the provider serves for the URL's id: its width, its bytes as loaded and whether it shares the pixels.
fn served(src: &str) -> Result<(u32, Vec<u8>, bool), String> {
    let id = src
        .strip_prefix(URL)
        .ok_or_else(|| format!("{src} is not the provider's"))?;
    Ok(kx_ui::snip_pixels(id))
}

/// The first frozen screen's URL in the hub's view now.
fn first_src(hub: &kx_ui::Hub) -> Result<String, String> {
    let v: Value = serde_json::from_str(&hub.snip_json()).map_err(|e| e.to_string())?;
    let src = v["frozen"][0]["src"]
        .as_str()
        .ok_or("no frozen screen in the view")?;
    Ok(src.to_owned())
}

/// The frozen screen is served over its own shared pixels; an old URL and an ended snip serve a clear pixel and let
/// the pixels go.
pub fn serves_the_frozen_screens_without_a_copy() -> Result<(), String> {
    let hub = crate::hub()?;
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).map_err(|e| e.to_string())?;
    let shot = Shot::new(2, 1, BGRX.to_vec()).ok_or("bad test pixels")?;
    let rect = Rect {
        x: 0,
        y: 0,
        w: 2,
        h: 1,
    };
    let screen = FrozenScreen::new(rect, 96, shot.clone()).ok_or("bad test screen")?;
    bus.publish(&SnipView::new(page(1, vec![screen])))
        .map_err(|e| e.to_string())?;
    let src = first_src(&hub)?;
    let got = served(&src)?;
    if got != (2, LOADED.to_vec(), true) {
        return Err(format!(
            "{src} served {got:?}, not the shared pixels made opaque"
        ));
    }
    let fresh = Shot::new(2, 1, BGRX.to_vec()).ok_or("bad test pixels")?;
    let again = FrozenScreen::new(rect, 96, fresh.clone()).ok_or("bad test screen")?;
    bus.publish(&SnipView::new(page(2, vec![again])))
        .map_err(|e| e.to_string())?;
    if first_src(&hub)? == src || served(&src)? != (CLEAR.0, CLEAR.1.to_vec(), false) {
        return Err(format!("{src} still serves after a new freeze"));
    }
    bus.publish(&SnipView::new(page(3, Vec::new())))
        .map_err(|e| e.to_string())?;
    let held = [&shot, &fresh].map(|s| Arc::strong_count(s.bgrx().expose()));
    if held != [1, 1] {
        return Err(format!(
            "the freezes' pixels are held {held:?} times after the snip"
        ));
    }
    Ok(())
}

/// Two opaque pixels as the editor's grab may hold them, red green blue: 3 2 1, then 6 5 4.
const PAINTED: [[i32; 3]; 2] = [[3, 2, 1], [6, 5, 4]];
/// The same pixels as Copy and Save take them: blue, green, red, then Qt's opaque unused byte.
const COPIED: [u8; 8] = [1, 2, 3, 255, 4, 5, 6, 255];

/// The editor's grab in each format Qt may hand over is copied once in the freeze's byte order, and a buffer of any
/// other size is refused.
pub fn copies_the_editors_picture_in_each_format() -> Result<(), String> {
    let formats = [
        (QImageFormat::Format_RGB32, "RGB32"),
        (
            QImageFormat::Format_ARGB32_Premultiplied,
            "ARGB32_Premultiplied",
        ),
        (QImageFormat::Format_RGBA8888, "RGBA8888"),
    ];
    for (format, name) in formats {
        let mut picture = QImage::from_width_height_and_format(2, 1, format);
        for (x, [r, g, b]) in (0..).zip(PAINTED) {
            picture.set_pixel_color(x, 0, &QColor::from_rgb(r, g, b));
        }
        let shot = kx_ui::snip_copied(&picture).ok_or("the grab gave no shot")?;
        let got = (shot.width(), shot.height(), shot.bgrx().expose().to_vec());
        if got != (2, 1, COPIED.to_vec()) {
            return Err(format!("{name} was copied as {got:?}"));
        }
        let lens = [COPIED.len() - 1, COPIED.len(), COPIED.len() + BGRX_BYTES];
        let fits = lens.map(|len| kx_ui::snip_fits(&picture, len));
        if fits != [false, true, false] {
            return Err(format!("{name} filled {lens:?} bytes as {fits:?}"));
        }
    }
    Ok(())
}

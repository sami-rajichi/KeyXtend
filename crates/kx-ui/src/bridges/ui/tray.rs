//! `Ui`'s tray icon: the picture QML grabs becomes the icon, at the tray's size.

use super::qobject;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{AspectRatioMode, QImage, TransformationMode};
use kx_platform::ui::TrayImage;

impl qobject::Ui {
    pub(super) fn tray_px(&self) -> i32 {
        let px = self.with(0, crate::hub::Hub::tray_px);
        i32::try_from(px).unwrap_or(i32::MAX)
    }

    pub(super) fn tray_show(&self, picture: &QImage) -> bool {
        let Some(hub) = self.rust().hub.clone() else {
            return false;
        };
        let Some(image) = rgba(picture, hub.tray_px()) else {
            tracing::warn!("QML sent an empty tray picture");
            return false;
        };
        hub.tray_show(&image)
    }

    pub(super) fn tray_hide(&self) {
        self.with((), crate::hub::Hub::tray_hide);
    }
}

/// `picture` scaled to `px` pixels square, as straight RGBA; `None` for an empty picture or size.
fn rgba(picture: &QImage, px: u32) -> Option<TrayImage> {
    let side = i32::try_from(px).ok().filter(|s| *s > 0)?;
    if picture.is_null() {
        return None;
    }
    let (fit, smooth) = (
        AspectRatioMode::IgnoreAspectRatio,
        TransformationMode::SmoothTransformation,
    );
    let scaled = picture.scaled(side, side, fit, smooth);
    let byte = |v: i32| u8::try_from(v).unwrap_or_default();
    let mut pixels = Vec::new();
    for y in 0..side {
        for x in 0..side {
            let c = scaled.pixel_color(x, y);
            pixels.extend([c.red(), c.green(), c.blue(), c.alpha()].map(byte));
        }
    }
    TrayImage::new(px, px, pixels).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cxx_qt_lib::{QColor, QImageFormat};

    /// A `side` pixels square picture filled with `rgba`, stored premultiplied as a grab is.
    fn filled(side: i32, rgba: [i32; 4]) -> QImage {
        let mut image = QImage::from_width_height_and_format(
            side,
            side,
            QImageFormat::Format_ARGB32_Premultiplied,
        );
        image.fill(&QColor::from_rgba(rgba[0], rgba[1], rgba[2], rgba[3]));
        image
    }

    #[test]
    fn a_picture_turns_into_straight_rgba_at_the_tray_size() {
        let image = rgba(&filled(20, [10, 20, 30, 255]), 16).unwrap();
        assert_eq!((image.width(), image.height()), (16, 16));
        assert!(
            image.rgba().chunks(4).all(|p| p == [10, 20, 30, 255]),
            "red, green, blue, then alpha"
        );
    }

    #[test]
    fn see_through_pixels_keep_their_colour_unpremultiplied() {
        let image = rgba(&filled(4, [200, 100, 50, 128]), 4).unwrap();
        let first = &image.rgba()[..4];
        let near = |a: u8, b: u8| a.abs_diff(b) <= 2;
        assert!(
            near(first[0], 200) && near(first[1], 100) && near(first[2], 50) && near(first[3], 128),
            "{first:?}"
        );
    }

    #[test]
    fn an_empty_picture_or_size_gives_none() {
        assert!(rgba(&QImage::default(), 16).is_none());
        assert!(rgba(&filled(4, [0, 0, 0, 255]), 0).is_none());
    }
}

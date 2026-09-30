//! The tray port: the app's icon in the taskbar's notification area, which opens our own menu when clicked.

use super::UiError;
use kx_module_api::ports::Point;
use std::fmt;
use thiserror::Error;

/// Bytes per RGBA pixel.
const RGBA: usize = 4;

/// A picture whose pixels do not fill its size.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("the picture is not {width} by {height} RGBA pixels")]
pub struct BadImage {
    /// The width it claimed.
    pub width: u32,
    /// The height it claimed.
    pub height: u32,
}

/// The icon's picture: straight (not premultiplied) RGBA pixels, row by row from the top left.
#[derive(Clone, PartialEq, Eq)]
pub struct TrayImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl TrayImage {
    /// A `width` by `height` picture from `rgba`.
    ///
    /// # Errors
    /// `BadImage` when a side is 0 or `rgba` is not exactly four bytes per pixel.
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, BadImage> {
        let bad = BadImage { width, height };
        let (w, h) = (usize::try_from(width), usize::try_from(height));
        let (Ok(w), Ok(h)) = (w, h) else {
            return Err(bad);
        };
        let want = w.checked_mul(h).and_then(|n| n.checked_mul(RGBA));
        if w == 0 || h == 0 || want != Some(rgba.len()) {
            return Err(bad);
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The pixels, four bytes each.
    #[must_use]
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
}

/// Shows the size only, never the pixels.
impl fmt::Debug for TrayImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrayImage")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// What the icon heard.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TrayEvent {
    /// The user clicked the icon or picked it with the keyboard; the menu opens from this point, in physical pixels.
    Click(Point),
    /// Another program's window came to the front: a click elsewhere, Alt+Tab, or a program raising itself.
    Away,
}

/// Called with each event, on the adapter's own thread.
pub type OnTray = Box<dyn Fn(TrayEvent) + Send>;

/// The icon while it shows; dropping it removes the icon and ends its events.
pub trait TrayIcon: Send {
    /// Changes the picture and the tooltip; a picture the taskbar refused is kept for when it restarts.
    ///
    /// # Errors
    /// `Refused` when the OS will not make the picture, or the taskbar refuses it and the adapter waits for its answer.
    fn set(&self, image: &TrayImage, tip: &str) -> Result<(), UiError>;
}

/// The app's icon in the taskbar's notification area.
pub trait Tray: Send + Sync {
    /// The picture's side in pixels, as the taskbar draws it at the system scale.
    fn icon_px(&self) -> u32;

    /// Shows the icon with `image` and tooltip `tip`, and calls `on` with each event until the icon drops.
    /// `on` runs on another thread, so it hands work to the UI thread and never blocks on the one that drops the icon.
    ///
    /// # Errors
    /// `Refused` when the OS will not make the picture or the icon's window; a taskbar not there yet gets the icon
    /// when it starts.
    fn show(&self, image: &TrayImage, tip: &str, on: OnTray) -> Result<Box<dyn TrayIcon>, UiError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_takes_four_bytes_per_pixel() {
        let image = TrayImage::new(2, 3, vec![7; 24]).unwrap();
        assert_eq!((image.width(), image.height()), (2, 3));
        assert_eq!(image.rgba().len(), 24);
    }

    #[test]
    fn a_picture_that_does_not_fill_its_size_is_refused() {
        let bad = |w, h, len| TrayImage::new(w, h, vec![0; len]);
        assert_eq!(
            bad(2, 3, 23),
            Err(BadImage {
                width: 2,
                height: 3
            })
        );
        assert!(bad(2, 3, 25).is_err(), "too many bytes");
        assert!(
            bad(0, 3, 0).is_err() && bad(3, 0, 0).is_err(),
            "an empty side"
        );
        assert!(bad(u32::MAX, u32::MAX, 4).is_err(), "a size that overflows");
    }

    #[test]
    fn a_picture_never_prints_its_pixels() {
        let image = TrayImage::new(1, 1, vec![0xAB; 4]).unwrap();
        let shown = format!("{image:?}");
        assert!(shown.contains("width") && !shown.contains("171"), "{shown}");
    }

    #[test]
    fn errors_name_the_size() {
        let e = BadImage {
            width: 16,
            height: 20,
        };
        assert!(e.to_string().contains("16 by 20"), "{e}");
    }
}

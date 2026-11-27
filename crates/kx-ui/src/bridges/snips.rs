//! Snip's picture provider on the app's engine (`cpp/snips.h`): `image://snip/<generation>/<place>` draws a frozen
//! screen or the editor's picture over the hub's own shared pixels, never a copy. It also makes a window before its
//! shield or first show, after its owner, hands the hub each window's show before the screen shows it, and copies the
//! editor's grab once, into a new shot, with probes for tests in debug builds (`cpp/snipprobes.h`).

use super::installed;
use crate::hub::SHOTS;
use cxx::UniquePtr;
use cxx_qt_lib::{QImage, QQmlApplicationEngine};
use kx_module_api::ports::Shot;
use std::pin::Pin;

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `snips.h` declares exactly these items with these types; Qt calls the provider on its GUI or image
    // thread, the engine owns the provider it is given, `fetch` and `shown` are plain Rust functions, safe on any
    // thread, the watch calls `shown` on the GUI thread that owns the windows, `make_titled` and `shown_titled` read
    // Qt's window list and so run on the GUI thread only, as only the Snip object's invokables call them, and the C++
    // side takes each `SnipPixels` it hands over back into a `rust::Box`, which Rust drops once Qt lets the image go.
    unsafe extern "C++" {
        include!("kx-ui/cpp/snips.h");
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        include!("cxx-qt-lib/qimage.h");
        /// Qt's QML engine for an app.
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
        /// Qt's picture in memory.
        type QImage = cxx_qt_lib::QImage;

        /// Registers the provider on `engine` as `name`; `fetch` hands over a picture's shared pixels.
        #[namespace = "kxui"]
        fn serve_snips(
            engine: Pin<&mut QQmlApplicationEngine>,
            name: &str,
            fetch: fn(&str) -> *mut SnipPixels,
        );

        /// Makes each of our windows titled `title` that Qt has not made yet; how many it made.
        #[namespace = "kxui"]
        fn make_titled(title: &str) -> usize;

        /// How many of our windows titled `title` Qt shows now, freed ones not yet deleted included.
        #[namespace = "kxui"]
        fn shown_titled(title: &str) -> usize;

        /// Hands each window's title on as Qt is about to show it, while it lives.
        #[namespace = "kxui"]
        type ShowWatch;

        /// A watch that calls `shown` with each window's title as Qt is about to show it; null before Qt's app object.
        #[namespace = "kxui"]
        fn show_watch_new(shown: fn(title: &str)) -> UniquePtr<ShowWatch>;

        /// Shows a new clear window titled `title` under a watch over `shown`, then lets it go; true when the watch
        /// told `shown` while its native window was made but hidden, still hidden once `shown` returned, and it showed
        /// after.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn show_watched(title: &str, shown: fn(title: &str)) -> bool;

        /// Makes the windows titled `title` beside a new hidden one; true when only that one was made, and it has its
        /// native window and stays hidden.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn titled_made(title: &str) -> bool;

        /// Makes a shadow, its window and that window's tip, all titled `title`; true when each was made after its
        /// owner, all stay hidden, and on Windows each lies in front of its owner.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn owners_made_first(title: &str) -> bool;

        /// True when each of the app's windows titled `title` and its owner are made and hidden; false for none.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn titled_owner_made(title: &str) -> bool;

        /// Shows a shadow titled `shade` and its window as Glow does, made ahead by `title` when `ahead`, through a
        /// dialog, a maximise, a restore and a tip; 0 when the window stays over its shadow and owned by it, `u8::MAX`
        /// when a make ahead missed the pair, else the first step that broke.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn glow_pair_broken(title: &str, shade: &str, ahead: bool) -> u8;

        /// Copies `picture` once into `bgrx`, row by row; false when it is not that size.
        #[namespace = "kxui"]
        fn snip_fill(picture: &QImage, bgrx: &mut [u8]) -> bool;

        /// What a provider over `fetch` serves for `id`, as RGBA8 bytes as its texture loads them, with its width, and
        /// whether it draws the shared pixels themselves.
        #[cfg(debug_assertions)]
        #[namespace = "kxui"]
        fn snip_served(
            id: &str,
            fetch: fn(&str) -> *mut SnipPixels,
            width: &mut u32,
            shared: &mut bool,
        ) -> Vec<u8>;
    }

    #[namespace = "kxui"]
    extern "Rust" {
        /// A picture's shared pixels, which C++ holds until Qt lets its image go.
        type SnipPixels;
        /// Pixels in a row, 0 for none.
        fn width(self: &SnipPixels) -> u32;
        /// Rows, 0 for none.
        fn height(self: &SnipPixels) -> u32;
        /// The bytes, `BGRX_BYTES` a pixel, row by row from the top.
        fn bgrx(self: &SnipPixels) -> &[u8];
    }

    // The C++ side drops what `fetch` hands over as a `rust::Box`.
    impl Box<SnipPixels> {}
}

/// A picture's shared pixels, or none.
pub struct SnipPixels(Option<Shot>);

impl SnipPixels {
    fn width(&self) -> u32 {
        self.0.as_ref().map_or(0, Shot::width)
    }

    fn height(&self) -> u32 {
        self.0.as_ref().map_or(0, Shot::height)
    }

    fn bgrx(&self) -> &[u8] {
        self.0.as_ref().map_or(&[], |s| s.bgrx().expose())
    }
}

/// Shares picture `id` of the installed hub's kept view, none when there is no such picture, and hands it to C++,
/// which holds it as a `rust::Box` until Qt lets the image go.
fn fetch(id: &str) -> *mut SnipPixels {
    Box::into_raw(Box::new(SnipPixels(
        installed().and_then(|hub| hub.snip_picture(id)),
    )))
}

/// Registers Snip's picture provider on `engine`; call it on the Qt GUI thread before the engine loads QML.
pub fn serve_snips(engine: Pin<&mut QQmlApplicationEngine>) {
    ffi::serve_snips(engine, SHOTS, fetch);
}

/// Makes each of our windows titled `title` that Qt has not made yet, so a shield can come before it shows.
pub(super) fn made(title: &str) {
    let _ = ffi::make_titled(title);
}

/// How many of our windows titled `title` Qt shows now, freed ones not yet deleted included.
pub(super) fn shown(title: &str) -> usize {
    ffi::shown_titled(title)
}

/// While it lives, each of our windows Qt shows during a capture's aside is kept out of captures before it shows.
pub(crate) struct ShowWatch {
    _watch: UniquePtr<ffi::ShowWatch>,
}

impl ShowWatch {
    /// Starts the watch, which warns when Qt's app object does not exist yet; call it on the Qt GUI thread.
    pub(crate) fn new() -> Self {
        let watch = ffi::show_watch_new(showing);
        if watch.is_null() {
            tracing::warn!(
                "Snip's show watch has no Qt app, so windows shown during an aside go unshielded"
            );
        }
        Self { _watch: watch }
    }
}

/// Qt is about to show the window titled `title`: the installed hub shields it while the keyboard is aside, and lays
/// the frozen screens under it and our other windows while they show.
fn showing(title: &str) {
    if let Some(hub) = installed() {
        hub.aside_window(title);
        hub.snip_lay();
    }
}

/// Shows a new clear window titled `title` under the show watch, then lets it go; true when the hub heard of it while its
/// native window was made but hidden, still hidden once the hub returned, and it showed after. Debug builds only, for
/// tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn show_watched(title: &str) -> bool {
    ffi::show_watched(title, showing)
}

/// Whether the window titled `title` Qt had not made is made, hidden, by the maker shields come after. Debug builds
/// only.
#[cfg(debug_assertions)]
#[must_use]
pub fn titled_made(title: &str) -> bool {
    ffi::titled_made(title)
}

/// Whether the maker makes a shadow, its window and that window's tip of one title `title` each after its owner, all
/// hidden, so each starts in front of its owner. Debug builds only.
#[cfg(debug_assertions)]
#[must_use]
pub fn owners_made_first(title: &str) -> bool {
    ffi::owners_made_first(title)
}

/// Whether each of the app's windows titled `title` is made and hidden with its owner, as a window's `make` leaves them
/// before it ever shows. Debug builds only.
#[cfg(debug_assertions)]
#[must_use]
pub fn titled_owner_made(title: &str) -> bool {
    ffi::titled_owner_made(title)
}

/// The first step at which a shadowed window titled `title`, its shadow titled `shade`, made ahead by `title` when
/// `ahead` and shown as Glow shows it, fell under its shadow or lost it as owner, through a dialog, a maximise, a
/// restore and a tip; 0 when none, `u8::MAX` when a make ahead missed the pair. Debug builds only.
#[cfg(debug_assertions)]
#[must_use]
pub fn glow_pair_broken(title: &str, shade: &str, ahead: bool) -> u8 {
    ffi::glow_pair_broken(title, shade, ahead)
}

/// The editor's flattened `picture` as a shot, copied once into its shared pixels; `None` for an empty picture.
pub(super) fn shot_of(picture: &QImage) -> Option<Shot> {
    let width = u32::try_from(picture.width()).ok()?;
    let height = u32::try_from(picture.height()).ok()?;
    Shot::filled(width, height, |bgrx| ffi::snip_fill(picture, bgrx))
}

/// The shot Copy and Save take from the editor's flattened `picture`. Debug builds only, for tests.
#[cfg(debug_assertions)]
#[must_use]
pub fn snip_copied(picture: &QImage) -> Option<Shot> {
    shot_of(picture)
}

/// Whether the editor's copy fills `len` bytes from `picture`, so tests see a wrong size refused. Debug builds only.
#[cfg(debug_assertions)]
#[must_use]
pub fn snip_fits(picture: &QImage, len: usize) -> bool {
    ffi::snip_fill(picture, &mut vec![0; len])
}

/// What the provider serves for `id`, `<generation>/<place>`, as RGBA8 bytes as its texture loads them, with its
/// width, and whether it draws the hub's shared pixels themselves; one clear pixel for none. Debug builds only.
#[cfg(debug_assertions)]
#[must_use]
pub fn snip_pixels(id: &str) -> (u32, Vec<u8>, bool) {
    let (mut width, mut shared) = (0, false);
    let rgba = ffi::snip_served(id, fetch, &mut width, &mut shared);
    (width, rgba, shared)
}

//! The UI: the texts in English, French and Arabic (Fluent, compiled in), the `[ui]` settings, the `ui` module and
//! the hub it shares with the app, and, with the `qt` feature, the QML module `KeyXtend.Ui` and its cxx-qt bridges.

// Only `bridges` may allow unsafe code, for the cxx-qt glue (ADR-0013, tidy F3).
#![deny(unsafe_code)]

#[cfg(feature = "qt")]
mod bridges;
#[cfg(all(feature = "qt", debug_assertions))]
mod churn;
pub mod device;
pub mod frames;
mod geom;
mod guard;
pub mod harden;
mod hub;
pub mod i18n;
mod plate;
mod settings;
#[cfg(feature = "qt")]
mod shell;
pub mod sounds;
mod ui_module;

#[cfg(feature = "qt")]
pub use bridges::clip::serve_clips;
#[cfg(feature = "qt")]
pub use bridges::device::Device;
#[cfg(feature = "qt")]
pub use bridges::frames::FrameLog;
#[cfg(feature = "qt")]
pub use bridges::harden::{
    ADAPTER_VAR, QML_DIR, drawing_pinned, harden_qt, pin_adapter, pin_drawing, pin_imports,
    pin_timing, qt_reads, qt_sees, swap_interval_set,
};
#[cfg(feature = "qt")]
pub use bridges::install;
#[cfg(feature = "qt")]
pub use bridges::shadows::serve_shadows;
#[cfg(feature = "qt")]
pub use bridges::tint::serve_icons;
#[cfg(all(feature = "qt", debug_assertions))]
pub use bridges::{
    clip::clip_pixels,
    device::{drawing, released, released_hidden, unreleased, wired},
    fonts::font_families,
    frames::{draw_window, mid_frame_draws},
    shadows::adopted,
    tint::{icon_mask, icon_pixels, icon_textures},
    wakes,
};
#[cfg(all(feature = "qt", debug_assertions))]
pub use churn::{caps_flipped, flip_caps};
pub use geom::Area;
pub use hub::{
    FontBook, Hub, HubError, Mode, Redraw, Request, Resize, Shown, UiPorts, Wake, Waker, asks,
    clip_acts,
};
pub use i18n::{AUTO, I18n, LANGS, Lang, find, os_lang, pick};
pub use settings::{Backdrop, PressFrame, SPEC, UiSettings};
#[cfg(feature = "qt")]
pub use shell::{After, Pick, Shell};
pub use ui_module::{ID, MANIFEST, UiModule};

/// The resource URL of the root QML file, which the app loads.
#[cfg(feature = "qt")]
pub const MAIN_QML: &str = env!("KX_UI_MAIN_QML");

/// The URI of the QML module compiled into the crate.
#[cfg(feature = "qt")]
pub const QML_URI: &str = env!("KX_UI_QML_URI");

/// The URL that serves the icons in their tint, the icon type and each compiled-in font's URL, as JSON
/// `{icons, ext, fonts}`, which QML reads.
#[cfg(feature = "qt")]
pub const ASSETS: &str = env!("KX_UI_ASSETS");

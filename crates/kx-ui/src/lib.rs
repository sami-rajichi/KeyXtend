//! The UI: the texts in English, French and Arabic (Fluent, compiled in), the `[ui]` settings, the `ui` module and
//! the hub it shares with the app, and, with the `qt` feature, the QML module `KeyXtend.Ui` and its cxx-qt bridges.

// Only `bridges` may allow unsafe code, for the cxx-qt glue (ADR-0013, tidy F3).
#![deny(unsafe_code)]

#[cfg(feature = "qt")]
mod bridges;
mod geom;
mod guard;
pub mod harden;
mod hub;
pub mod i18n;
mod plate;
mod settings;
mod ui_module;

#[cfg(feature = "qt")]
pub use bridges::harden::{QML_DIR, harden_qt, pin_imports};
#[cfg(feature = "qt")]
pub use bridges::install;
pub use geom::Area;
pub use hub::{Hub, HubError, Redraw, Request, Resize, Shown, UiPorts, Wake, Waker};
pub use i18n::{AUTO, I18n, LANGS, Lang, find, pick};
pub use settings::{Backdrop, SPEC, UiSettings};
pub use ui_module::{ID, MANIFEST, UiModule};

/// The resource URL of the root QML file, which the app loads.
#[cfg(feature = "qt")]
pub const MAIN_QML: &str = env!("KX_UI_MAIN_QML");

/// The compiled-in icon folder URL and type and each font's URL, as JSON `{icons, ext, fonts}`, which QML reads.
#[cfg(feature = "qt")]
pub const ASSETS: &str = env!("KX_UI_ASSETS");

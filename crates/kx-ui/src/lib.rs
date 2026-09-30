//! The UI. Today it holds the texts in English, French and Arabic (Fluent, compiled in); the QML views, cxx-qt
//! bridges and the `ui` module join it next.

#![forbid(unsafe_code)]

pub mod i18n;

pub use i18n::{AUTO, I18n, LANGS, Lang, find, pick};

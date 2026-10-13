//! The app without Qt: the command line, the one running copy, the quiet panic hook, the log, and the kernel with
//! every module over the hub the UI shares. `main.rs` adds the Windows adapter and Qt's loop.

#![forbid(unsafe_code)]

pub mod args;
pub mod boot;
pub mod copy;
pub mod drain;
pub mod log;
pub mod quiet;
pub mod relay;

/// True in the build whose manifest asks for uiAccess (`build.rs` sets `kx_ui_access`), which loads libraries only from
/// the app's folder and System32, and plugins and QML only from the app's folder; other builds find Qt through `PATH`.
pub const UI_ACCESS: bool = cfg!(kx_ui_access);

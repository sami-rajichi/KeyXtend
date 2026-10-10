//! Qt's start, hardened for the uiAccess process: no variable Qt reads survives but the graphics adapter and the drawing
//! variables the app picks, Qt Quick keeps no shader cache on disk, and when pinned only the app's own folders give
//! plugins and QML modules.

use crate::harden::{DRAW_VARS, qt_names};
use cxx_qt_lib::{QQmlApplicationEngine, QString, QStringList};
use std::path::{MAIN_SEPARATOR, Path};
use std::pin::Pin;

/// The resource root of the compiled-in QML modules, from the build.
const QML_ROOT: &str = env!("KX_UI_QML_ROOT");
/// The folder beside the app that holds Qt's own QML modules.
pub const QML_DIR: &str = "qml";
/// Where the engine looks for a module's plugin: the folder of its `qmldir`, as Qt does by default.
const MODULE_PLUGINS: &str = ".";
/// The variable Qt's Direct3D backend reads for the adapter it draws on, by its place in DXGI's adapter list.
pub const ADAPTER_VAR: &str = "QT_D3D_ADAPTER_INDEX";

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `harden.h` declares exactly these functions with these types; QStringList is cxx-qt-lib's binding. On
    // Windows the C runtime and Win32 lock their environments for each write. `narrow_names` reads without the lock,
    // but only in `harden_qt`, before Qt starts; the adapter pins of a device rebuild write later, on the GUI thread.
    unsafe extern "C++" {
        include!("kx-ui/cpp/harden.h");
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt string list.
        type QStringList = cxx_qt_lib::QStringList;

        /// Every name in the C runtime's narrow environment, which Qt's byte reads use, each ended by `NAME_END`.
        #[namespace = "kxui"]
        fn narrow_names() -> Vec<u8>;

        /// Removes variable `name`, in the narrow environment's bytes, from what Qt reads; false when it could not.
        #[namespace = "kxui"]
        fn unset_env(name: &[u8]) -> bool;

        /// Sets variable `name` to `value` where Qt reads it; false when it could not.
        #[namespace = "kxui"]
        fn set_env(name: &str, value: &str) -> bool;

        /// True when Qt reads variable `name` as set.
        #[namespace = "kxui"]
        fn qt_sees(name: &str) -> bool;

        /// True when Qt's text read, through the wide copy, finds variable `name`.
        #[namespace = "kxui"]
        fn qt_reads(name: &str) -> bool;

        /// The value Qt's byte read gives variable `name`, from the narrow copy Qt Quick reads; empty when it is not set.
        #[namespace = "kxui"]
        fn qt_value(name: &str) -> String;

        /// Turns off Qt Quick's shader pipeline cache on disk.
        #[namespace = "kxui"]
        fn no_shader_cache();

        /// Qt loads plugins only from `paths` and the app's folder.
        #[namespace = "kxui"]
        fn set_library_paths(paths: &QStringList);
    }
}

/// Clears every variable Qt may read, by the narrow names Qt reads, and turns off Qt Quick's shader cache on disk;
/// with `plugins`, Qt loads plugins only from that folder. Call it before Qt's app object exists, on Windows, where
/// both environments are locked and no thread of the app writes them; it says how many variables went.
pub fn harden_qt(plugins: Option<&Path>) -> usize {
    let names = ffi::narrow_names();
    let cleared = qt_names(&names)
        .into_iter()
        .filter(|n| ffi::unset_env(n))
        .count();
    let left = qt_names(&ffi::narrow_names()).len();
    if left > 0 {
        tracing::warn!(left, "some Qt variables could not be cleared");
    }
    ffi::no_shader_cache();
    if let Some(dir) = plugins {
        ffi::set_library_paths(&list([text(dir)]));
    }
    cleared
}

/// Has Qt draw on adapter `index`, its place in DXGI's adapter list; call it after `harden_qt`, which clears the
/// variable, and before Qt's app object exists. False when the variable could not be set.
#[must_use]
pub fn pin_adapter(index: u32) -> bool {
    ffi::set_env(ADAPTER_VAR, &index.to_string())
}

/// Sets every variable of `DRAW_VARS` where Qt reads it; call it after `harden_qt` and before Qt's app object exists.
/// False when one could not be set.
#[must_use]
pub fn pin_drawing() -> bool {
    let failed = DRAW_VARS
        .iter()
        .filter(|(name, value)| !ffi::set_env(name, value));
    failed.count() == 0
}

/// True when Qt's byte read gives every variable of `DRAW_VARS` the app's value, so every window draws on the GUI
/// thread.
#[must_use]
pub fn drawing_pinned() -> bool {
    DRAW_VARS
        .iter()
        .all(|(name, value)| ffi::qt_value(name) == *value)
}

/// Drops the adapter pin, so Qt draws on its default adapter; false when none was set or it could not go.
pub(crate) fn unpin_adapter() -> bool {
    ffi::qt_sees(ADAPTER_VAR) && ffi::unset_env(ADAPTER_VAR.as_bytes())
}

/// True when Qt reads variable `name` as set, through the C runtime's narrow copy it reads at start.
#[must_use]
pub fn qt_sees(name: &str) -> bool {
    ffi::qt_sees(name)
}

/// True when Qt's text read (`qEnvironmentVariable`) finds variable `name`, through the C runtime's wide copy.
#[must_use]
pub fn qt_reads(name: &str) -> bool {
    ffi::qt_reads(name)
}

/// Pins `engine`'s QML imports to the compiled-in modules and the `qml` folder in `dir`, and each module's plugin
/// to its own folder; call it before loading QML.
pub fn pin_imports(mut engine: Pin<&mut QQmlApplicationEngine>, dir: &Path) {
    let imports = list([QML_ROOT.to_owned(), text(&dir.join(QML_DIR))]);
    engine.as_mut().set_import_path_list(&imports);
    engine.set_plugin_path_list(&list([MODULE_PLUGINS.to_owned()]));
}

/// `paths` as a Qt list, with forward slashes, as Qt writes paths.
fn list(paths: impl IntoIterator<Item = String>) -> QStringList {
    let slashed = paths.into_iter().map(|p| p.replace(MAIN_SEPARATOR, "/"));
    slashed.map(|p| QString::from(p.as_str())).collect()
}

/// The text of `path`; a part that is not Unicode becomes U+FFFD, which matches no folder.
fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

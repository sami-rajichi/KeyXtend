//! Qt's start, hardened for the uiAccess process: no variable Qt reads survives, Qt Quick keeps no shader cache on
//! disk, and when pinned only the app's own folders give plugins and QML modules.

use crate::harden::qt_names;
use cxx_qt_lib::{QQmlApplicationEngine, QString, QStringList};
use std::path::{MAIN_SEPARATOR, Path};
use std::pin::Pin;

/// The resource root of the compiled-in QML modules, from the build.
const QML_ROOT: &str = env!("KX_UI_QML_ROOT");
/// The folder beside the app that holds Qt's own QML modules.
pub const QML_DIR: &str = "qml";
/// Where the engine looks for a module's plugin: the folder of its `qmldir`, as Qt does by default.
const MODULE_PLUGINS: &str = ".";

#[cxx_qt::bridge]
mod ffi {
    // SAFETY: `harden.h` declares exactly these functions with these types; QStringList is cxx-qt-lib's binding. On
    // Windows the C runtime and Win32 lock their environments, and no thread of the app writes them, so
    // `narrow_names` may read without the lock while other threads run.
    unsafe extern "C++" {
        include!("kx-ui/cpp/harden.h");
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt string list.
        type QStringList = cxx_qt_lib::QStringList;

        /// Every name in the C runtime's narrow environment, which Qt reads, each ended by `NAME_END`.
        #[namespace = "kxui"]
        fn narrow_names() -> Vec<u8>;

        /// Removes variable `name`, in the narrow environment's bytes, from what Qt reads; false when it could not.
        #[namespace = "kxui"]
        fn unset_env(name: &[u8]) -> bool;

        /// True when Qt reads variable `name` as set.
        #[namespace = "kxui"]
        fn qt_sees(name: &str) -> bool;

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

/// True when Qt reads variable `name` as set, through the C runtime's narrow copy it reads at start.
#[must_use]
pub fn qt_sees(name: &str) -> bool {
    ffi::qt_sees(name)
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

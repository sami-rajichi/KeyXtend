//! Qt's start in the uiAccess process: Qt variables another program planted reach neither Qt's plugin folders nor the
//! QML import paths, nor Qt's narrow or wide reads, pinned plugins and imports hold only the app's folders and the
//! compiled-in modules, and only the graphics adapter, the drawing variables and the frame timing the app picks reach
//! Qt; the frame log starts only once drawing is pinned. The variables must be set before the process starts, so the
//! test runs itself again, as a child, with them set.

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QStringList};
use kx_test_support::tempdir::TempDir;
use kx_ui::harden::{DRAW_VARS, LATENCY_VAR, is_qt, swap_interval, timing_vars};
use kx_ui::{FrameLog, UiSettings};
use std::env;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, ExitCode};

/// Marks the child run; its value is the folder the parent planted.
const CHILD: &str = "KX_HARDEN_CHILD";
/// The folder the child pins Qt's plugins to; it holds a copy of the platform plugin.
const PINNED: &str = "KX_HARDEN_PINNED";
/// The variables the child starts with, one in lower case, as Windows ignores case, and a planted adapter.
const PLANTED: [&str; 4] = [
    "QT_PLUGIN_PATH",
    "QML2_IMPORT_PATH",
    "qml_import_path",
    kx_ui::ADAPTER_VAR,
];
/// The adapter the child pins, as the app pins the lowest-power one.
const ADAPTER: u32 = 1;
/// A render loop another program chose, which the app's own value must replace (ADR-0021).
const OUTSIDE: (&str, &str) = ("QSG_RENDER_LOOP", "threaded");
/// A frame latency another program chose, which the `[ui]` setting must replace.
const LATE: (&str, &str) = (LATENCY_VAR, "0");
/// Names the C runtime narrows to a Qt variable with the ANSI code page's best fit: a fullwidth `ＱＴ`, and a
/// non-ASCII last letter. Each pairs the planted name with the name Qt then reads.
const ALIASES: [(&str, &str); 2] = [
    ("ＱＴ_KX_PROBE", "QT_KX_PROBE"),
    ("QT_QPA_PLATFORM_PLUGIN_PATĤ", "QT_QPA_PLATFORM_PLUGIN_PATH"),
];
/// The variable naming Qt's `qmake`, two folders above Qt's own plugins.
const QMAKE: &str = "QMAKE";
/// Qt's plugin folder, and the platform plugin Qt needs to start, inside it.
const PLUGINS: &str = "plugins";
const PLATFORM: [&str; 2] = ["platforms", "qwindows.dll"];

type Outcome = Result<(), String>;

fn main() -> ExitCode {
    let outcome = match (env::var_os(CHILD), env::var_os(PINNED)) {
        (Some(planted), Some(pinned)) => child(Path::new(&planted), Path::new(&pinned)),
        _ => parent(),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}

/// Qt's own plugin folder, found from `QMAKE`.
fn qt_plugins() -> Result<PathBuf, String> {
    let qmake = PathBuf::from(env::var_os(QMAKE).ok_or("QMAKE is not set")?);
    let qt = qmake.ancestors().nth(2).ok_or("QMAKE has no Qt folder")?;
    Ok(qt.join(PLUGINS))
}

/// Plants a folder in Qt's variables, pins a folder holding the platform plugin, and runs this binary again; the
/// child's verdict is the test's.
fn parent() -> Outcome {
    let planted = TempDir::new("planted").map_err(|e| e.to_string())?;
    let pinned = TempDir::new("pinned").map_err(|e| e.to_string())?;
    let [folder, plugin] = PLATFORM;
    std::fs::create_dir(pinned.path().join(folder)).map_err(|e| e.to_string())?;
    let from = qt_plugins()?.join(folder).join(plugin);
    std::fs::copy(from, pinned.path().join(folder).join(plugin)).map_err(|e| e.to_string())?;
    let mut run = Command::new(env::current_exe().map_err(|e| e.to_string())?);
    run.env(CHILD, planted.path()).env(PINNED, pinned.path());
    run.env(OUTSIDE.0, OUTSIDE.1).env(LATE.0, LATE.1);
    for name in PLANTED.into_iter().chain(ALIASES.map(|(alias, _)| alias)) {
        run.env(name, planted.path());
    }
    let status = run.status().map_err(|e| e.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("the child run failed: {status}"))
}

/// The items of `list`, with forward slashes.
fn items(list: &QStringList) -> Vec<String> {
    list.iter()
        .map(|s| s.to_string().replace(MAIN_SEPARATOR, "/"))
        .collect()
}

/// `dir` with forward slashes, in lower case.
fn slashed(dir: &Path) -> String {
    dir.to_string_lossy()
        .replace(MAIN_SEPARATOR, "/")
        .to_lowercase()
}

/// True when some item of `list` names `dir`.
fn names(list: &QStringList, dir: &Path) -> bool {
    let dir = slashed(dir);
    items(list).iter().any(|i| i.to_lowercase().contains(&dir))
}

/// Whether Qt sees each alias's narrow name set.
fn seen() -> Vec<(&'static str, bool)> {
    ALIASES
        .map(|(_, name)| (name, kx_ui::qt_sees(name)))
        .to_vec()
}

/// The planted variables Qt's text read finds, through the C runtime's wide copy.
fn wide() -> Vec<&'static str> {
    PLANTED.into_iter().filter(|v| kx_ui::qt_reads(v)).collect()
}

/// Clears Qt's variables, pins its plugins, starts Qt, and checks its plugin and import paths.
fn child(planted: &Path, pinned: &Path) -> Outcome {
    let (before, wide_before) = (seen(), wide());
    let cleared = kx_ui::harden_qt(Some(pinned));
    let after = seen();
    eprintln!("Qt's narrow view before harden_qt: {before:?}; after: {after:?}");
    if wide_before.len() != PLANTED.len() || !wide().is_empty() {
        return Err(format!(
            "Qt's wide read found {wide_before:?} before and {:?} after",
            wide()
        ));
    }
    if cleared < PLANTED.len() || PLANTED.iter().any(|v| env::var_os(v).is_some()) {
        return Err(format!(
            "{cleared} Qt variables were cleared, and some are still set"
        ));
    }
    if after.iter().any(|(_, set)| *set) {
        return Err(format!("Qt still sees an alias: {after:?}"));
    }
    if FrameLog::start().is_some() {
        return Err("the frame log started while Qt may draw off the GUI thread".into());
    }
    adapter()?;
    drawing()?;
    timing()?;
    let app = QGuiApplication::new();
    if FrameLog::start().is_none() {
        return Err("the frame log did not start once drawing was pinned".into());
    }
    let paths = app
        .as_ref()
        .map(QGuiApplication::library_paths)
        .unwrap_or_default();
    library(&paths, planted, pinned)?;
    let mut engine = QQmlApplicationEngine::new();
    let e = engine.as_mut().ok_or("the engine did not start")?;
    imports(e, planted)
}

/// After the clear, Qt reads the adapter the app pins, and only that.
fn adapter() -> Outcome {
    if !kx_ui::pin_adapter(ADAPTER) {
        return Err("the adapter was not pinned".into());
    }
    let read = env::var(kx_ui::ADAPTER_VAR).ok();
    if !kx_ui::qt_sees(kx_ui::ADAPTER_VAR) || read != Some(ADAPTER.to_string()) {
        return Err(format!("Qt's adapter is {read:?}, not {ADAPTER}"));
    }
    Ok(())
}

/// After the clear and the pins, Qt reads exactly the adapter and the drawing variables, with the app's values, so the
/// outside render loop is replaced.
fn drawing() -> Outcome {
    if !kx_ui::pin_drawing() {
        return Err("the drawing variables were not set".into());
    }
    let names = DRAW_VARS.map(|(name, _)| name);
    let mut want: Vec<String> = names
        .iter()
        .chain([&kx_ui::ADAPTER_VAR])
        .map(|n| n.to_uppercase())
        .collect();
    // Qt's text read matches only its own ASCII names; a planted non-ASCII alias stays in the wide copy, unread.
    let mut set: Vec<String> = env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.is_ascii() && is_qt(name.as_bytes()))
        .map(|name| name.to_uppercase())
        .collect();
    want.sort();
    set.sort();
    if set != want {
        return Err(format!("Qt reads {set:?}, not {want:?}"));
    }
    let wrong: Vec<&str> = DRAW_VARS
        .iter()
        .filter(|(name, value)| {
            env::var(name).ok().as_deref() != Some(*value) || !kx_ui::qt_sees(name)
        })
        .map(|(name, _)| *name)
        .collect();
    let replaced = DRAW_VARS
        .iter()
        .any(|(name, value)| *name == OUTSIDE.0 && *value != OUTSIDE.1);
    if !wrong.is_empty() || !replaced || !kx_ui::drawing_pinned() {
        return Err(format!(
            "{wrong:?} hold other values, {} was not replaced, or Qt's byte read differs",
            OUTSIDE.0
        ));
    }
    Ok(())
}

/// After the drawing pins, Qt reads the frame timing of the `[ui]` defaults, Qt's own values, in place of an outside
/// one, and every window asks for its frames at a screen refresh.
fn timing() -> Outcome {
    let s = UiSettings::from_defaults().map_err(|e| e.to_string())?;
    if !kx_ui::pin_timing(&s) {
        return Err("the frame timing was not set".into());
    }
    let vars = timing_vars(&s);
    let wrong: Vec<&str> = vars
        .iter()
        .filter(|(name, value)| {
            env::var(name).ok().as_deref() != Some(value.as_str()) || !kx_ui::qt_sees(name)
        })
        .map(|(name, _)| *name)
        .collect();
    let interval = kx_ui::swap_interval_set();
    let replaced = vars
        .iter()
        .any(|(name, value)| *name == LATE.0 && value != LATE.1);
    if !wrong.is_empty() || !replaced || interval != swap_interval(&s) {
        return Err(format!(
            "{wrong:?} hold other values, {} was not replaced, or the swap interval is {interval}",
            LATE.0
        ));
    }
    Ok(())
}

/// Qt's plugin folders hold the pinned folder, and neither the planted one nor Qt's own.
fn library(paths: &QStringList, planted: &Path, pinned: &Path) -> Outcome {
    if names(paths, planted) {
        return Err(format!("a planted plugin folder reached Qt: {paths}"));
    }
    if !names(paths, pinned) || names(paths, &qt_plugins()?) {
        return Err(format!("Qt's plugin folders are not pinned: {paths}"));
    }
    Ok(())
}

/// The planted folder reached no QML import path, and pinned imports hold only the resources and the app's `qml`.
fn imports(mut e: Pin<&mut QQmlApplicationEngine>, planted: &Path) -> Outcome {
    if names(&e.import_path_list(), planted) {
        return Err(format!(
            "a planted import folder reached QML: {}",
            e.import_path_list()
        ));
    }
    let dir = TempDir::new("app").map_err(|e| e.to_string())?;
    kx_ui::pin_imports(e.as_mut(), dir.path());
    let qml = dir.path().join(kx_ui::QML_DIR);
    let imports = items(&e.import_path_list());
    let want = slashed(&qml);
    if imports.len() != 2 || !imports[0].starts_with("qrc:") || imports[1].to_lowercase() != want {
        return Err(format!(
            "pinned imports are {imports:?}, not the resources and {want}"
        ));
    }
    let plugins = items(&e.plugin_path_list());
    if plugins != ["."] {
        return Err(format!("pinned plugin paths are {plugins:?}"));
    }
    Ok(())
}

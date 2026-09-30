//! The QML module loads: the root file finds `KeyXtend.Ui`, its `Ui` object starts over the hub and follows the
//! system look, and every icon the tokens name and every bundled font loads from the compiled-in resources. Qt wants
//! its app object on the main thread, so this binary runs without the test harness.

use cxx::UniquePtr;
use cxx_qt_lib::{QByteArray, QGuiApplication, QQmlApplicationEngine, QUrl};
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform::ui::SysLook;
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeWindowGuard};
use kx_theme::Themes;
use kx_ui::{ASSETS, Hub, MAIN_QML, UiPorts};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A hub over fresh fakes, as the app builds over the real adapters, with the fake look it follows.
fn hub() -> Option<(Arc<Hub>, Arc<FakeSystemLook>)> {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };
    let screen = Screen {
        rect,
        dpi: BASE_DPI,
        primary: true,
    };
    let look = Arc::new(FakeSystemLook::new());
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look: look.clone(),
        backdrop: Arc::new(FakeBackdrop::new()),
    };
    Hub::new(ports).ok().map(|(hub, _requests)| (hub, look))
}

/// True when the hub draws the dark look, as it does once it follows a dark system.
fn dark(hub: &Hub) -> bool {
    let look: Option<serde_json::Value> = serde_json::from_str(&hub.look_json()).ok();
    look.is_some_and(|v| v["dark"] == true)
}

/// Where the asset check says it is loaded from; it imports only Qt's own modules.
const CHECK_URL: &str = "qrc:/kx-ui-tests/assets.qml";

/// Loads QML into an engine the caller keeps, with `load`; `None` when Qt could not make its object.
fn load(
    load: impl FnOnce(std::pin::Pin<&mut QQmlApplicationEngine>),
) -> Option<UniquePtr<QQmlApplicationEngine>> {
    let mut engine = QQmlApplicationEngine::new();
    let failed = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&failed);
    let mut e = engine.as_mut()?;
    e.as_mut()
        .on_object_creation_failed(move |_, _| seen.store(true, Ordering::SeqCst))
        .release();
    load(e);
    (!failed.load(Ordering::SeqCst)).then_some(engine)
}

/// QML that loads every icon the tokens name and every font `Ui.assets()` lists, warns about each one that fails,
/// and exits with their count once the event loop runs.
fn asset_check() -> Option<String> {
    let themes = Themes::shipped().ok()?;
    let names = serde_json::to_string(&themes.icons.names()).ok()?;
    Some(format!(
        r#"import QtQuick
QtObject {{
    id: check
    readonly property var assets: {ASSETS}
    readonly property var names: {names}
    function loads(type, url) {{
        const o = Qt.createQmlObject("import QtQuick; " + type + " {{}}", check);
        o.source = url;
        return o.status === (type === "Image" ? Image.Ready : FontLoader.Ready);
    }}
    Component.onCompleted: {{
        const icons = names.map(n => assets.icons + "/" + n + "." + assets.ext);
        const bad = icons.filter(u => !loads("Image", u)).concat(assets.fonts.filter(u => !loads("FontLoader", u)));
        bad.forEach(u => console.warn("did not load: " + u));
        Qt.callLater(() => Qt.exit(assets.fonts.length > 0 ? bad.length : 1));
    }}
}}
"#
    ))
}

fn main() -> ExitCode {
    let Some((hub, look)) = hub() else {
        eprintln!("the hub did not start");
        return ExitCode::FAILURE;
    };
    if !kx_ui::install(Arc::clone(&hub)) {
        eprintln!("a hub was installed already");
        return ExitCode::FAILURE;
    }
    let mut app = QGuiApplication::new();
    let Some(_engine) = load(|e| e.load(&QUrl::from(MAIN_QML))) else {
        eprintln!("{MAIN_QML} did not load");
        return ExitCode::FAILURE;
    };
    look.set(SysLook {
        dark: true,
        ..SysLook::default()
    });
    if !dark(&hub) {
        eprintln!("the Ui object did not start following the system look");
        return ExitCode::FAILURE;
    }
    let Some(check) = asset_check() else {
        eprintln!("the asset check could not be written");
        return ExitCode::FAILURE;
    };
    let data = QByteArray::from(check.as_str());
    let Some(_check) = load(|e| e.load_data(&data, &QUrl::from(CHECK_URL))) else {
        eprintln!("the asset check did not load");
        return ExitCode::FAILURE;
    };
    if app.as_mut().map(|a| a.exec()) == Some(0) {
        return ExitCode::SUCCESS;
    }
    eprintln!("icons or fonts did not load from the resources, or no font is bundled");
    ExitCode::FAILURE
}

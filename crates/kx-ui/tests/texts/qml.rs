//! Writes what the Qt Quick tests read (settings, texts, the keyboard's golden board and states, the assets laid out
//! as the resources hold them); a change or a stray file fails it until it runs with `KX_BLESS=1`.

use crate::files::keys_of;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeTray, FakeWindowGuard};
use kx_test_support::golden::assert_golden;
use kx_theme::assets::{ICON_EXT, ICONS_AT, bundle};
use kx_ui::{Hub, I18n, UiPorts};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The fixtures folder, under this crate.
const DIR: &str = "qmltest/fixtures";
/// The keyboard's golden files, which pin the face's JSON.
const FACE: &str = "../kx-mod-keyboard/tests/golden";
/// The face's states the tests draw, by golden file name.
const STATES: [&str; 4] = ["en", "fr", "ar", "list"];
/// The UI languages the tests draw.
const LANGS: [&str; 3] = ["en", "fr", "ar"];
/// Where the bundled files are laid out, from this crate; the Qt Quick tests read them from there.
const MIRROR: &str = "../../target/tmp/qmltest-assets";
/// The repo's `assets/` folder, from this crate.
const ASSETS: &str = "../../assets";
/// The fake tray's picture side.
const TRAY_PX: u32 = 16;
/// The banner's counts the tests read, from none up to this many.
const MORE_UP_TO: usize = 12;
/// What starts each fixture, so QML imports it as a JavaScript library.
const HEAD: &str = ".pragma library\n// Written by kx-ui's texts test; do not edit.\n";

/// The fixtures folder.
fn dir() -> PathBuf {
    crate_path(DIR)
}

/// `name = value;` as JavaScript, spread over lines when `pretty`, else on one line.
fn var(name: &str, value: &Value, pretty: bool) -> String {
    let json = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    format!("var {name} = {};\n", json.expect("JSON writes"))
}

/// A hub whose system shows language `tag`, over fake ports.
fn hub(tag: &str) -> Arc<Hub> {
    let look = Arc::new(FakeSystemLook::new());
    look.set_language(tag);
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
    let ports = UiPorts {
        guard: Arc::new(FakeWindowGuard::new(0)),
        screens: Arc::new(FakeScreens::new(vec![screen], Point { x: 0, y: 0 })),
        look,
        backdrop: Arc::new(FakeBackdrop::new()),
        tray: Arc::new(FakeTray::new(TRAY_PX)),
    };
    Hub::new(ports).expect("the hub starts").0
}

/// Language `tag`'s settings and language JSON, every text by key, and the banner's counts by how many wait.
fn ui(tag: &str) -> String {
    let hub = hub(tag);
    let settings: Value = serde_json::from_str(&hub.settings_json()).expect("settings JSON");
    let lang: Value = serde_json::from_str(&hub.lang_json()).expect("language JSON");
    let t: Arc<I18n> = hub.i18n();
    let texts: Map<String, Value> = keys_of(t.lang().ftl)
        .into_iter()
        .map(|k| {
            let v = Value::String(t.text(&k));
            (k, v)
        })
        .collect();
    let texts = Value::Object(texts);
    let more: Vec<Value> = (0..=MORE_UP_TO).map(|n| Value::String(t.more(n))).collect();
    format!(
        "{HEAD}{}{}{}{}",
        var("settings", &settings, true),
        var("lang", &lang, true),
        var("texts", &texts, true),
        var("more", &Value::Array(more), true)
    )
}

/// Golden file `name` of the keyboard as JSON.
fn golden(name: &str) -> Value {
    let path = crate_path(FACE).join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).expect("a keyboard golden file");
    serde_json::from_str(&text).expect("golden JSON")
}

/// The board and every state the tests draw, one line each, as the keyboard's own golden files show them in full, and
/// the smallest and largest sizes the keyboard allows and its size step.
fn face() -> String {
    let states: Map<String, Value> = STATES
        .iter()
        .map(|s| ((*s).to_owned(), golden(&format!("state-{s}"))))
        .collect();
    let states = Value::Object(states);
    let size = |key: &str| {
        let v = crate::keyboard_defaults()[key].clone();
        serde_json::to_value(v).expect("a size")
    };
    format!(
        "{HEAD}{}{}{}{}{}",
        var("board", &golden("board"), false),
        var("states", &states, false),
        var("sizeMin", &size("size_min"), false),
        var("sizeMax", &size("size_max"), false),
        var("sizeStep", &size("size_step"), false)
    )
}

/// The bundled assets as `Ui.assets()` names them, relative to where they are laid out, and that place.
fn assets() -> String {
    let b = bundle(&crate_path(ASSETS)).expect("the bundle is found");
    let fonts: Vec<&str> = b.fonts.iter().map(|f| f.name.as_str()).collect();
    let assets = serde_json::json!({ "icons": ICONS_AT, "ext": ICON_EXT, "fonts": fonts });
    // The Qt Quick tests sit one folder below this crate.
    let root = Value::String(format!("../{MIRROR}"));
    format!(
        "{HEAD}{}{}",
        var("root", &root, false),
        var("assets", &assets, true)
    )
}

/// `rel` under this crate.
fn crate_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Checks fixture `name` against `text`, or writes it when blessing; returns the name.
fn fixture(name: String, text: &str) -> String {
    assert_golden(dir().join(&name), text);
    name
}

#[test]
fn the_qml_fixtures_match_the_hub_texts_and_face() {
    let mut written: Vec<String> = LANGS
        .iter()
        .map(|tag| fixture(format!("ui-{tag}.js"), &ui(tag)))
        .collect();
    written.push(fixture("face.js".into(), &face()));
    written.push(fixture("assets.js".into(), &assets()));
    for f in std::fs::read_dir(dir())
        .expect("the fixtures folder reads")
        .flatten()
    {
        let name = f.file_name().to_string_lossy().into_owned();
        assert!(
            written.contains(&name),
            "{DIR}/{name} is no longer written; delete it"
        );
    }
}

#[test]
fn the_bundle_is_laid_out_as_the_app_resources_hold_it() {
    let b = bundle(&crate_path(ASSETS)).expect("the bundle is found");
    let mirror = crate_path(MIRROR);
    if mirror.exists() {
        std::fs::remove_dir_all(&mirror).expect("the old layout goes");
    }
    for f in b.icons.iter().chain(&b.fonts) {
        let to = mirror.join(&f.name);
        std::fs::create_dir_all(to.parent().expect("a folder")).expect("the folder is made");
        std::fs::copy(&f.file, &to).expect("the file is copied");
    }
    let icon = b.icons.first().expect("an icon");
    assert!(mirror.join(&icon.name).is_file());
}

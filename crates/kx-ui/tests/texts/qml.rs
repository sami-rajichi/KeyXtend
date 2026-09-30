//! Writes what the Qt Quick tests read: the hub's settings and every text in English and Arabic, and the keyboard's
//! golden board and states. A change fails this test until it runs with `KX_BLESS=1`, and so does a stray file.

use crate::files::keys_of;
use kx_module_api::ports::{BASE_DPI, Point, Rect, Screen};
use kx_platform_fake::{FakeBackdrop, FakeScreens, FakeSystemLook, FakeWindowGuard};
use kx_test_support::golden::assert_golden;
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
const LANGS: [&str; 2] = ["en", "ar"];
/// What starts each fixture, so QML imports it as a JavaScript library.
const HEAD: &str = ".pragma library\n// Written by kx-ui's texts test; do not edit.\n";

/// The fixtures folder.
fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR)
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
    };
    Hub::new(ports).expect("the hub starts").0
}

/// Language `tag`'s settings JSON and every text by key.
fn ui(tag: &str) -> String {
    let hub = hub(tag);
    let settings: Value = serde_json::from_str(&hub.settings_json()).expect("settings JSON");
    let t: Arc<I18n> = hub.i18n();
    let texts: Map<String, Value> = keys_of(t.lang().ftl)
        .into_iter()
        .map(|k| {
            let v = Value::String(t.text(&k));
            (k, v)
        })
        .collect();
    let texts = Value::Object(texts);
    format!(
        "{HEAD}{}{}",
        var("settings", &settings, true),
        var("texts", &texts, true)
    )
}

/// Golden file `name` of the keyboard as JSON.
fn golden(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FACE)
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).expect("a keyboard golden file");
    serde_json::from_str(&text).expect("golden JSON")
}

/// The board and every state the tests draw, one line each, as the keyboard's own golden files show them in full.
fn face() -> String {
    let states: Map<String, Value> = STATES
        .iter()
        .map(|s| ((*s).to_owned(), golden(&format!("state-{s}"))))
        .collect();
    let states = Value::Object(states);
    format!(
        "{HEAD}{}{}",
        var("board", &golden("board"), false),
        var("states", &states, false)
    )
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

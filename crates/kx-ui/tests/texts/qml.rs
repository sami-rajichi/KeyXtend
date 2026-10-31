//! Writes what the Qt Quick tests read (settings, texts, filled texts, the keyboard's golden board and states, the chip
//! lanes and ask kinds, the assets laid out as the resources hold them with the icon plugin, and the picture folders);
//! a change or a stray file fails it until it runs with `KX_BLESS=1`.

use crate::files::keys_of;
use crate::pictures::{PICTURES_QML, hand_over};
use kx_module_api::ports::{
    BASE_DPI, FaceAction, FaceError, FaceReply, KeyboardFace, Point, Rect, Screen,
};
use kx_platform_fake::{
    FakeBackdrop, FakeMemory, FakeReveal, FakeScreens, FakeSpeaker, FakeSystemLook, FakeTray,
    FakeWindowGuard,
};
use kx_test_support::golden::assert_golden;
use kx_test_support::host::FakeHost;
use kx_test_support::tempdir::TempDir;
use kx_theme::assets::{ICON_EXT, SYSTEM_FONTS, bundle, icon_url};
use kx_ui::{Hub, I18n, MANIFEST, UiPorts, UiSettings};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The fixtures folder, under this crate.
const DIR: &str = "qmltest/fixtures";
/// The keyboard's golden files, which pin the face's JSON.
const FACE: &str = "../kx-mod-keyboard/tests/golden";
/// The face's states the tests draw, by golden file name: the usual side keys', then the shortcuts layer's.
const STATES: [&str; 9] = [
    "en", "fr", "ar", "ar-shift", "ar-altgr", "list", "en-caps", "layer-1", "layer-3",
];
/// The shortcuts layer's boards the tests draw, on its first and last pages, by golden file name after `board-`.
const LAYERS: [&str; 2] = ["layer-1", "layer-3"];
/// The UI languages the tests draw.
pub const LANGS: [&str; 3] = ["en", "fr", "ar"];
/// Where the bundled files and the icon plugin are laid out, from this crate; the Qt Quick tests read them from there.
pub const MIRROR: &str = "../../target/tmp/qmltest-assets";
/// The repo's `assets/` folder, from this crate.
const ASSETS: &str = "../../assets";
/// The fake tray's picture side.
const TRAY_PX: u32 = 16;
/// The banner's counts the tests read, from none up to this many.
const MORE_UP_TO: usize = 12;
/// What starts each fixture, so QML imports it as a JavaScript library.
pub const HEAD: &str = ".pragma library\n// Written by kx-ui's texts test; do not edit.\n";

/// The fixtures folder.
fn dir() -> PathBuf {
    crate_path(DIR)
}

/// `name = value;` as JavaScript, spread over lines when `pretty`, else on one line.
pub fn var(name: &str, value: &Value, pretty: bool) -> String {
    let json = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    format!("var {name} = {};\n", json.expect("JSON writes"))
}

/// A face that shows a layout in language `0`; the fixtures read only its language.
struct Speaks(String);

impl KeyboardFace for Speaks {
    fn board_json(&self) -> String {
        Value::Null.to_string()
    }

    fn state_json(&self, _: usize) -> String {
        Value::Null.to_string()
    }

    fn lang(&self) -> String {
        self.0.clone()
    }

    fn row_px(&self) -> f64 {
        f64::from(crate::board::row())
    }

    fn act(&self, _: FaceAction) -> Result<FaceReply, FaceError> {
        Ok(FaceReply::default())
    }
}

/// A hub whose system shows language `tag`, with a layout in `tag` in front, and the host behind its settings.
pub fn hub(tag: &str) -> (Arc<Hub>, FakeHost) {
    let hub = system(tag);
    let mut host = FakeHost::new(&MANIFEST);
    let settings = UiSettings::from_defaults().expect("the [ui] defaults");
    hub.attach(settings, Arc::new(Speaks(tag.to_owned())), host.handle());
    (hub, host)
}

/// A hub whose system shows language `tag`, over fake ports.
fn system(tag: &str) -> Arc<Hub> {
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
        memory: Arc::new(FakeMemory::new()),
        speaker: Arc::new(FakeSpeaker::new()),
        reveal: Arc::new(FakeReveal::new()),
    };
    Hub::new(ports).expect("the hub starts").0
}

/// Language `tag`'s settings and language JSON, every text by key, the banner's counts by how many wait, and the pointer
/// overlay's and scroll pad's states as the hub starts.
fn ui(tag: &str) -> String {
    let (hub, _host) = hub(tag);
    let settings: Value = serde_json::from_str(&hub.settings_json()).expect("settings JSON");
    let lang: Value = serde_json::from_str(&hub.lang_json()).expect("language JSON");
    let assist: Value = serde_json::from_str(&hub.assist_json()).expect("overlay JSON");
    let scroll: Value = serde_json::from_str(&hub.scroll_json()).expect("scroll JSON");
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
        "{HEAD}{}{}{}{}{}{}",
        var("settings", &settings, true),
        var("lang", &lang, true),
        var("texts", &texts, true),
        var("more", &Value::Array(more), true),
        var("assist", &assist, true),
        var("scroll", &scroll, true)
    )
}

/// The ask kinds QML sends and the chip lanes the tests draw, as the English hub writes them.
fn words() -> String {
    let (hub, _host) = hub("en");
    format!(
        "{HEAD}{}{}",
        var("asks", &crate::lanes::asks(), true),
        var("lanes", &crate::lanes::lanes(&hub), true)
    )
}

/// Golden file `name` of the keyboard as JSON.
fn golden(name: &str) -> Value {
    let path = crate_path(FACE).join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path).expect("a keyboard golden file");
    serde_json::from_str(&text).expect("golden JSON")
}

/// The golden files `{prefix}{name}` for each of `names`, by name.
fn goldens(prefix: &str, names: &[&str]) -> Value {
    let all: Map<String, Value> = names
        .iter()
        .map(|s| ((*s).to_owned(), golden(&format!("{prefix}{s}"))))
        .collect();
    Value::Object(all)
}

/// The board, the layer's boards and every state the tests draw, one line each, as the keyboard's own golden files show
/// them in full, and the smallest and largest sizes the keyboard allows and its size step.
fn face() -> String {
    let states = goldens("state-", &STATES);
    let size = |key: &str| {
        let v = crate::keyboard_defaults()[key].clone();
        serde_json::to_value(v).expect("a size")
    };
    format!(
        "{HEAD}{}{}{}{}{}{}",
        var("board", &golden("board"), false),
        var("layers", &goldens("board-", &LAYERS), false),
        var("states", &states, false),
        var("sizeMin", &size("size_min"), false),
        var("sizeMax", &size("size_max"), false),
        var("sizeStep", &size("size_step"), false)
    )
}

/// The parts of each space-bar text the face's states make, once, with the first state that makes it.
pub fn spaces() -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for s in STATES {
        let parts = golden(&format!("state-{s}"))["lang"]["space"].clone();
        let parts: Vec<String> = serde_json::from_value(parts).expect("space-bar parts");
        if !out.iter().any(|(_, p)| *p == parts) {
            out.push(((*s).to_owned(), parts));
        }
    }
    out
}

/// The bundled assets as `Ui.assets()` names them, the fonts relative to where they are laid out, that place, and the
/// font families Windows ships, which the tests' font rule may skip on a system without them.
fn assets() -> String {
    let b = bundle(&crate_path(ASSETS)).expect("the bundle is found");
    let fonts: Vec<&str> = b.fonts.iter().map(|f| f.name.as_str()).collect();
    let assets = serde_json::json!({ "icons": icon_url(), "ext": ICON_EXT, "fonts": fonts });
    // The Qt Quick tests sit one folder below this crate.
    let root = Value::String(format!("../{MIRROR}"));
    format!(
        "{HEAD}{}{}{}{}",
        var("root", &root, false),
        var("assets", &assets, true),
        var("system", &serde_json::json!(SYSTEM_FONTS), false),
        var("pictures", &Value::String(PICTURES_QML.into()), false)
    )
}

/// `rel` under this crate.
fn crate_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Checks fixture `name` against `text`, or writes it when blessing; returns the name.
pub fn fixture(name: String, text: &str) -> String {
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
    written.push(fixture("words.js".into(), &words()));
    let order = var("texts", &crate::filled::order(), true);
    written.push(fixture("order.js".into(), &format!("{HEAD}{order}")));
    // The clipboard views are checked by their own test, so blessing them touches no other fixture.
    written.push(crate::clipview::FILE.into());
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
    let fresh = TempDir::new("qmltest-assets").expect("a folder of its own");
    for f in b.icons.iter().chain(&b.fonts) {
        let to = fresh.path().join(&f.name);
        std::fs::create_dir_all(to.parent().expect("a folder")).expect("the folder is made");
        std::fs::copy(&f.file, &to).expect("the file is copied");
    }
    crate::plugin::lay_out(fresh.path());
    hand_over(fresh.path());
    let mirror = crate_path(MIRROR);
    crate::sync::sync(fresh.path(), &mirror);
    let icon = b.icons.first().expect("an icon");
    assert!(mirror.join(&icon.name).is_file());
    assert!(mirror.join(PICTURES_QML).is_file());
}

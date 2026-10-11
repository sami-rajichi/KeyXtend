//! A colour change steps an icon's tint once a frame; each step's texture goes when the next arrives, so a fade leaves
//! one texture alive, not one per step.

use crate::Outcome;
use cxx_qt_lib::{QByteArray, QQmlApplicationEngine, QUrl};
use kx_theme::assets::{ICON_EXT, icon_url};

/// The tint steps the check makes, about a 150 ms fade's frames for three icons.
const STEPS: u64 = 40;
/// Where the check's QML says it is loaded from.
const FADE_URL: &str = "qrc:/kx-ui-tests/fade.qml";

/// One app icon, named `name`, at `side` pixels, whose tint steps `STEPS` times once it is made.
fn fade(name: &str, side: i32) -> String {
    format!(
        r#"import QtQuick
import {uri}
Icon {{
    icon: "{name}"
    base: "{base}"
    ext: "{ICON_EXT}"
    px: {side}
    tint: "black"
    Component.onCompleted: {{
        for (let k = 1; k <= {STEPS}; k++)
            tint = Qt.rgba(k / {STEPS}, 0, 0, 1);
    }}
}}
"#,
        uri = kx_ui::QML_URI,
        base = icon_url(),
    )
}

/// After `STEPS` tint steps, the provider drew each step and only the last one's texture is alive.
pub(crate) fn a_fade_keeps_one_texture(name: &str, side: i32) -> Outcome {
    let (made, live) = kx_ui::icon_textures();
    let mut engine = QQmlApplicationEngine::new();
    let mut e = engine.as_mut().ok_or("Qt made no QML engine")?;
    kx_ui::serve_icons(e.as_mut());
    e.load_data(
        &QByteArray::from(fade(name, side).as_str()),
        &QUrl::from(FADE_URL),
    );
    let (now_made, now_live) = kx_ui::icon_textures();
    let (drawn, kept) = (now_made.saturating_sub(made), now_live.saturating_sub(live));
    if drawn < STEPS || kept > 1 {
        return Err(format!(
            "a fade of {STEPS} steps drew {drawn} textures and kept {kept} alive"
        ));
    }
    Ok(())
}
